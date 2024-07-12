use std::{collections::HashMap, fmt::Debug, time::SystemTime};

use aws_config::{BehaviorVersion, ConfigLoader};
use aws_credential_types::Credentials;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_sts::Client as StsClient;
use tokio::io::AsyncReadExt;

use super::{FileInfo, FileMetadata, Storage, StorageInfo};
use crate::{
    errors::{StorageError, StorageResult},
    models::TokenWrap,
};

#[derive(Clone)]
pub struct AwsClient {
    s3_client: S3Client,
    pub sub: String,
}

impl Debug for AwsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AwsClient").field("sub", &self.sub).finish()
    }
}

impl AwsClient {
    pub async fn new(jwt_token: TokenWrap) -> StorageResult<Self> {
        let sub = jwt_token.get_sub().unwrap();

        let c = assume_role(jwt_token.raw(), &sub).await?;
        let creds = Credentials::new(
            c.access_key_id,
            c.secret_access_key,
            Some(c.session_token),
            Some(SystemTime::try_from(c.expiration).unwrap()),
            "sts",
        );

        let config = ConfigLoader::default()
            .credentials_provider(creds)
            .behavior_version(BehaviorVersion::latest())
            .region("us-east-1")
            .load()
            .await;

        let s3_client = S3Client::new(&config);

        Ok(Self { s3_client, sub })
    }
}

#[async_trait::async_trait]
impl Storage for AwsClient {
    async fn list_objects(&self, info: StorageInfo<'_>) -> StorageResult<Vec<FileInfo>> {
        let objects = self
            .s3_client
            .list_objects_v2()
            .bucket(info.bucket.to_string())
            .prefix(info.url)
            .delimiter("/".to_string())
            .send()
            .await
            .map_err(StorageError::from)?;

        Ok(objects
            .contents
            .unwrap_or_default()
            .iter()
            .map(|f| FileInfo {
                name: f.key.clone().unwrap_or_default(),
                owner: f.owner.clone().and_then(|o| o.id).unwrap_or_default(),
                last_modified: f.last_modified.map(|c| c.to_string()).unwrap_or_default(),
                metadata: None,
            })
            .collect())
    }

    async fn delete(&self, info: StorageInfo<'_>) -> StorageResult<()> {
        let _objects = self
            .s3_client
            .delete_object()
            .bucket(info.bucket.to_string())
            .key(info.url)
            .send()
            .await
            .map_err(StorageError::from)?;
        Ok(())
    }
    async fn get_metadata(&self, info: StorageInfo<'_>) -> StorageResult<FileMetadata> {
        self.s3_client
            .head_object()
            .bucket(info.bucket.to_string())
            .key(info.url)
            .send()
            .await
            .map(|m| FileMetadata {
                size: m.content_length.map(|b| b.to_string()).unwrap_or("0".to_string()),
                r#type: m.content_type.unwrap_or_default(),
                custom: m.metadata.unwrap_or_default(),
            })
            .map_err(StorageError::from)
    }

    async fn set_metadata(&self, _info: StorageInfo<'_>, _metadata: HashMap<String, String>) -> StorageResult<()> {
        todo!();
    }

    async fn upload(&self, info: StorageInfo<'_>) -> StorageResult<()> {
        self.s3_client
            .put_object()
            .bucket(info.bucket.to_string())
            .key(info.url)
            .body(info.data.unwrap().into())
            .send()
            .await
            .map_err(StorageError::from)?;
        Ok(())
    }

    async fn download(&self, info: StorageInfo<'_>) -> StorageResult<Vec<u8>> {
        match self
            .s3_client
            .get_object()
            .bucket(info.bucket.to_string())
            .key(info.url)
            .send()
            .await
        {
            Ok(object) => {
                let mut data_read = object.body.into_async_read();
                let mut data = Vec::new();
                data_read
                    .read_to_end(&mut data)
                    .await
                    .expect("Should be able to read from stream");

                Ok(data)
            }
            Err(e) => Err(e.into()),
        }
    }
}

async fn assume_role(token: &str, sub: &str) -> StorageResult<aws_sdk_sts::types::Credentials> {
    let config = ConfigLoader::default()
        .no_credentials()
        .behavior_version(BehaviorVersion::latest())
        .region("us-east-1")
        .load()
        .await;

    let client = StsClient::new(&config);
    match client
        .assume_role_with_web_identity()
        .set_role_session_name(Some(format!("session-{}", sub)))
        .role_arn("arn:aws:iam::071771013126:role/KeycloakAccess")
        .web_identity_token(token)
        .send()
        .await
    {
        Ok(e) => e.credentials().cloned().ok_or(StorageError::Credentials),
        Err(e) => Err(e.into()),
    }
}
