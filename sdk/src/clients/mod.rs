mod api;
mod aws;
mod gc;
mod http_client;
mod keycloak;
mod token;

use core::fmt::Debug;
use std::{
    fs::File,
    io::{Read, Write},
};

pub use api::ApiClient;
pub use aws::AwsClient;
pub use gc::GoogleCloud;
pub(crate) use http_client::*;
pub use keycloak::Keycloak;
pub use token::TokenManager;

use crate::{
    errors::{SecretResult, StorageError, StorageResult},
    models::{TokenType, TokenWrap},
};

pub const BUCKET_PATH: &str = "stronghold-snapshots";
pub const STRONGHOLD_PATH: &str = "stronghold";
pub const IDENTITY_METADATA: &str = "metadata";
pub const STREAMS_PATH: &str = "stream";
pub const MESSAGE_PATH: &str = "demia-messages";
pub const DOCUMENTS_PATH: &str = "documents";

pub enum StorageDataType<'a> {
    StreamsSnapshot(&'a str),
    StrongholdSnapshot(&'a str),
    IdentityMetadata(&'a str),
    Document(&'a str),
}

/// Storage info
#[derive(Default)]
pub struct StorageInfo<'a> {
    /// Name of the bucket
    bucket: &'a str,
    /// Name of the file
    url: String,
    /// Content/body
    data: Option<Vec<u8>>,
}

pub(crate) fn get_paths<'a>(data: &'a StorageDataType) -> (&'a str, String) {
    match *data {
        StorageDataType::StreamsSnapshot(path) => (path, STREAMS_PATH.to_owned()),
        StorageDataType::StrongholdSnapshot(path) => (path, STRONGHOLD_PATH.to_owned()),
        StorageDataType::IdentityMetadata(path) => (path, IDENTITY_METADATA.to_owned()),
        StorageDataType::Document(path) => (path, format!("{}/{}", DOCUMENTS_PATH, path)),
    }
}

#[async_trait::async_trait]
pub trait Storage {
    type FileInfo;
    type File;

    async fn upload(&self, file: StorageInfo<'_>) -> StorageResult<()>;

    async fn download(&self, info: StorageInfo<'_>) -> StorageResult<Vec<u8>>;

    async fn delete(&self, info: StorageInfo<'_>) -> StorageResult<()>;

    async fn list_objects(&self, file: String) -> StorageResult<Vec<String>>;
}

#[async_trait::async_trait]
pub trait SecretManager: Debug + Send + Sync {
    async fn get_token(&mut self, token_type: &TokenType, username: &str, password: &str) -> SecretResult<TokenWrap>;

    async fn refresh_token(&mut self) -> SecretResult<TokenWrap>;
}

pub(crate) fn default_secret() -> Box<impl SecretManager> {
    Box::new(Keycloak::default())
}

pub enum Clients {
    AWS(AwsClient),
    GC(GoogleCloud),
}

#[derive(Debug, Clone)]
pub struct StorageClient<T: Storage> {
    storage: T,
    pub sub: String,
}

impl<T: Storage> StorageClient<T> {
    pub async fn new(jwt_token: TokenWrap, storage: T) -> StorageResult<Self> {
        let sub = jwt_token.get_sub().unwrap();
        Ok(Self { storage, sub })
    }

    pub async fn upload(&self, info: StorageInfo<'_>) -> StorageResult<()> {
        self.storage.upload(info).await
    }

    pub async fn upload_data(&self, data: StorageDataType<'_>) -> StorageResult<()> {
        let (file_path, storage_path) = get_paths(&data);
        let key = format!("users/{}/{}", self.sub, storage_path);

        let file = File::open(file_path).expect("File not found");
        let mut data = Vec::new();
        file.take(10 * 1024 * 1024)
            .read_to_end(&mut data)
            .expect("Failed to read file");

        // Upload the file
        self.storage
            .upload(StorageInfo {
                url: key,
                bucket: BUCKET_PATH,
                data: Some(data),
            })
            .await
    }

    pub async fn upload_metadata<S: serde::Serialize + Send>(&self, metadata: S) -> StorageResult<()> {
        let (_, storage_path) = get_paths(&StorageDataType::IdentityMetadata(""));
        let key = format!("users/{}/{}", self.sub, storage_path);

        log::debug!("Metadata key: {}", key);
        // Upload the file
        self.storage
            .upload(StorageInfo {
                url: key,
                bucket: BUCKET_PATH,
                data: Some(serde_json::to_vec(&metadata).expect("Metadata is serializable, should not fail")),
            })
            .await
    }

    pub async fn download_data(&self, storage_type: StorageDataType<'_>) -> StorageResult<Vec<u8>> {
        let (file_path, storage_path) = get_paths(&storage_type);
        let key = format!("users/{}/{}", self.sub, storage_path);

        let info = StorageInfo {
            url: key,
            bucket: BUCKET_PATH,
            ..Default::default()
        };

        let raw = self.storage.download(info).await;
        match storage_type {
            StorageDataType::IdentityMetadata(_) | StorageDataType::Document(_) => match raw {
                Ok(object) => Ok(object),
                Err(_) => Ok(vec![]),
            },
            _ => {
                let data = raw?;

                let mut file = File::options()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(file_path)
                    .expect("Should be able to write to provided file path");
                file.write_all(&data).expect("should be able to write to that file...");

                Ok(data)
            }
        }
    }
}
