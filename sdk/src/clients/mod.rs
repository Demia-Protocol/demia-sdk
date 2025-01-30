mod auth0;
mod http;

#[cfg(feature = "aws")]
mod aws;

#[cfg(feature = "aws_rusoto")]
mod aws_rusoto;

#[cfg(feature = "google_cloud")]
mod gc;

mod keycloak;
mod token;

use core::fmt::Debug;
use std::{
    collections::HashMap as Map,
    fs::File,
    io::{Read, Write},
};

pub use auth0::Auth0Client;
#[cfg(feature = "aws")]
pub use aws::AwsClient;
#[cfg(feature = "aws_rusoto")]
pub use aws_rusoto::AwsRusotoClient;
use chrono::{DateTime, Utc};
#[cfg(feature = "google_cloud")]
pub use gc::GoogleCloud;
pub use http::*;
pub use keycloak::Keycloak;
use rocket_okapi::okapi::schemars;
pub use token::TokenManager;

use crate::{
    errors::{SecretResult, StorageResult},
    models::{Asset, TokenType, TokenWrap},
};

pub const STRONGHOLD_PATH: &str = "stronghold";
pub const IDENTITY_METADATA: &str = "metadata";
pub const STREAMS_PATH: &str = "stream";
pub const MESSAGE_PATH: &str = "demia-messages";

pub const USERS_PATH: &str = "users";
pub const SITES_PATH: &str = "sites";
pub const ASSETS_PATH: &str = "assets";

pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum StorageDataType<'a> {
    StreamsSnapshot(&'a str),
    StrongholdSnapshot(&'a str),
    IdentityMetadata(&'a str),
    Document(&'a str, &'a str), // Site, filename
    Asset(&'a str, Asset),
}

impl<'a> StorageDataType<'a> {
    pub fn get_paths(&self, sub: &'a str) -> (&'a str, String) {
        match self {
            Self::StreamsSnapshot(path) => (path, format!("{}/{}/{}", USERS_PATH, sub, path)),
            Self::StrongholdSnapshot(path) => (path, format!("{}/{}/{}", USERS_PATH, sub, STRONGHOLD_PATH.to_owned())),
            Self::IdentityMetadata(path) => (path, format!("{}/{}/{}", USERS_PATH, sub, IDENTITY_METADATA.to_owned())),
            Self::Document(site, file) => (file, format!("{}/{}/{}/{}", SITES_PATH, site, sub, file)),
            Self::Asset(site, asset) => (site, format!("{}/{}/{}", ASSETS_PATH, site, asset.storage_path())),
        }
    }

    pub fn is_public(&self) -> bool {
        matches!(self, Self::Asset(_, _))
    }
}

/// Storage info
#[derive(Debug, Default, schemars::JsonSchema)]
pub struct StorageInfo<'a> {
    /// Name of the bucket
    bucket: &'a str,
    /// Name of the file
    url: String,
    /// Content/body
    data: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct FileInfo {
    pub name: String,
    pub owner: String,
    #[serde(rename = "lastModified")]
    pub last_modified: String,
    pub metadata: Option<FileMetadata>,
}

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct FileMetadata {
    pub size: String,
    pub r#type: String,
    pub custom: Map<String, String>,
}

#[async_trait::async_trait]
pub trait Storage {
    // Unused
    // type FileInfo = FileInfo;
    // type File = Vec<u8>;

    /// Upload an object
    async fn upload(&self, file: StorageInfo<'_>) -> StorageResult<()>;

    /// Download an object
    /// If last_modified is specified, only downloads if the remote version is newer
    async fn download(&self, info: StorageInfo<'_>, last_modified: Option<DateTime<Utc>>) -> StorageResult<Vec<u8>>;

    /// marks an object for deletion from the storage. Does not guarantee immediate deletion.
    async fn delete(&self, info: StorageInfo<'_>) -> StorageResult<()>;

    /// List all objects from the path, does not get metadata.
    async fn list_objects(&self, file: StorageInfo<'_>) -> StorageResult<Vec<FileInfo>>;

    /// Get object metadata
    async fn get_metadata(&self, file: StorageInfo<'_>) -> StorageResult<FileMetadata>;

    /// Assign object metadata, set/update is dependant on trait implementation
    async fn set_metadata(&self, file: StorageInfo<'_>, metadata: Map<String, String>) -> StorageResult<()>;

    /// Refresh credentials for storage provider
    async fn update_credentials(&mut self, token: TokenWrap) -> StorageResult<()>;
}

#[async_trait::async_trait]
pub trait SecretManager: Debug + Send + Sync {
    /// Gets the specific token from the manager using the refresh token
    async fn get_token(&mut self, token_type: &TokenType, username: &str, password: &str) -> SecretResult<TokenWrap>;
    /// Gets a token using a token secret
    async fn get_token_with_secret(&mut self, token_type: &TokenType, client_secret: &str) -> SecretResult<TokenWrap>;
    /// Updates the refresh token used to connect to the manager
    async fn refresh_token(&mut self) -> SecretResult<TokenWrap>;
    /// Get token data from raw token response
    async fn token_from_raw(&self, token_type: &TokenType, token: &str) -> SecretResult<TokenWrap>;
}

pub(crate) fn default_secret() -> Box<impl SecretManager> {
    Box::<Keycloak>::default()
}

pub enum Clients {
    AWS(AwsClient),
    #[cfg(feature = "aws_rusoto")]
    AWSRusoto(AwsRusotoClient),
    #[cfg(feature = "google_cloud")]
    GC(GoogleCloud),
}

#[derive(Debug, Clone)]
pub struct StorageClient<T: Storage> {
    storage: T,
    pub sub: String,
    pub private_bucket_path: String,
    pub public_bucket_path: String,
}

impl<T: Storage> StorageClient<T> {
    pub async fn new(
        public_bucket_path: String,
        private_bucket_path: String,
        jwt_token: TokenWrap,
        storage: T,
    ) -> StorageResult<Self> {
        let sub = jwt_token.get_sub().unwrap();
        Ok(Self {
            public_bucket_path,
            private_bucket_path,
            storage,
            sub,
        })
    }

    fn get_bucket(&self, data: &StorageDataType<'_>) -> &str {
        self.get_bucket_path(data.is_public())
    }

    fn get_bucket_path(&self, public: bool) -> &str {
        match public {
            true => &self.public_bucket_path,
            false => &self.private_bucket_path,
        }
    }

    /// Uploads the data from the optional parameter if it exists.
    /// Otherwise, upload from file system.
    pub async fn upload(&self, data: StorageDataType<'_>, content: Option<Vec<u8>>) -> StorageResult<()> {
        let (file_path, storage_path) = data.get_paths(&self.sub);
        let bucket = self.get_bucket(&data);
        let data = content.unwrap_or_else(|| {
            let file = File::open(file_path).expect("File not found");
            let mut data = Vec::new();
            file.take(MAX_FILE_SIZE)
                .read_to_end(&mut data)
                .expect("Failed to read file");
            data
        });

        self.storage
            .upload(StorageInfo {
                url: storage_path,
                bucket,
                data: Some(data),
            })
            .await
    }

    /// Uploads the data from a file on the system
    pub async fn upload_data(&self, data: StorageDataType<'_>) -> StorageResult<()> {
        self.upload(data, None).await
    }

    pub async fn list_objects(&self, path: String, get_metadata: bool, public: bool) -> StorageResult<Vec<FileInfo>> {
        let mut objs = self
            .storage
            .list_objects(StorageInfo {
                url: path,
                bucket: self.get_bucket_path(public),
                data: None,
            })
            .await?;

        match get_metadata {
            false => Ok(objs),
            true => {
                for obj in &mut objs {
                    let meta = self.get_metadata_raw(obj.name.clone()).await?;
                    obj.metadata = Some(meta);
                }
                Ok(objs)
            }
        }
    }

    /// Function expects raw path of file for the storage provider.
    pub async fn get_metadata_raw(&self, file: String) -> StorageResult<FileMetadata> {
        self.storage
            .get_metadata(StorageInfo {
                url: file,
                bucket: &self.private_bucket_path,
                data: None,
            })
            .await
    }

    pub async fn delete(&self, data: StorageDataType<'_>) -> StorageResult<()> {
        let (_, storage_path) = data.get_paths(&self.sub);
        self.storage
            .delete(StorageInfo {
                url: storage_path,
                bucket: &self.private_bucket_path,
                data: None,
            })
            .await
    }

    pub async fn upload_metadata<S: serde::Serialize + Send>(&self, metadata: &S) -> StorageResult<()> {
        let (_, storage_path) = StorageDataType::IdentityMetadata("").get_paths(&self.sub);
        self.storage
            .upload(StorageInfo {
                url: storage_path,
                bucket: &self.private_bucket_path,
                data: Some(serde_json::to_vec(metadata).expect("Metadata is serializable, should not fail")),
            })
            .await
    }

    pub async fn download_data(
        &self,
        storage_type: StorageDataType<'_>,
        last_modified: Option<DateTime<Utc>>,
    ) -> StorageResult<Vec<u8>> {
        let (file_path, storage_path) = storage_type.get_paths(&self.sub);
        let info = StorageInfo {
            url: storage_path,
            bucket: &self.private_bucket_path,
            ..Default::default()
        };

        let raw = self.storage.download(info, last_modified).await;
        match storage_type {
            StorageDataType::IdentityMetadata(_) | StorageDataType::Document(_, _) => match raw {
                Ok(object) => Ok(object),
                Err(_) => Ok(vec![]),
            },
            _ => {
                let data = raw?;

                // Stronghold will crash with a completely empty file
                if data.len() > 0 {
                    let mut file = File::options()
                        .create(true)
                        .truncate(true)
                        .write(true)
                        .read(true)
                        .open(file_path)
                        .expect("Should be able to write to provided file path");
                    file.write_all(&data).expect("should be able to write to that file...");
                }

                Ok(data)
            }
        }
    }

    pub async fn update_credentials(&mut self, token: TokenWrap) -> StorageResult<()> {
        self.storage.update_credentials(token).await
    }
}
