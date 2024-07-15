use std::collections::HashMap as Map;

use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        download::Range,
        get::GetObjectRequest,
        upload::{Media, UploadObjectRequest, UploadType},
    },
};

use crate::{
    clients::{FileInfo, FileMetadata, Storage, StorageInfo},
    errors::{StorageError, StorageResult},
};
use crate::models::TokenWrap;

#[derive(Clone)]
pub struct GoogleCloud {
    client: Client,
}

impl GoogleCloud {
    pub fn new(config: ClientConfig) -> StorageResult<Self> {
        Ok(Self {
            client: Client::new(config),
        })
    }
}

#[async_trait::async_trait]
impl Storage for GoogleCloud {
    // type FileInfo = UploadObjectRequest;
    // type File = Object;

    async fn list_objects(&self, _info: StorageInfo<'_>) -> StorageResult<Vec<FileInfo>> {
        todo!()
    }

    async fn get_metadata(&self, _info: StorageInfo<'_>) -> StorageResult<FileMetadata> {
        todo!()
    }

    async fn set_metadata(&self, _info: StorageInfo<'_>, _metadata: Map<String, String>) -> StorageResult<()> {
        todo!()
    }

    async fn delete(&self, _info: StorageInfo<'_>) -> StorageResult<()> {
        todo!()
    }

    async fn upload(&self, info: StorageInfo<'_>) -> StorageResult<()> {
        let upload_type = UploadType::Simple(Media::new(info.url.clone()));
        self.client
            .upload_object(
                &UploadObjectRequest {
                    bucket: info.bucket.to_owned(),
                    ..Default::default()
                },
                info.data.unwrap_or_default(),
                &upload_type,
            )
            .await
            .map_err(StorageError::from)?;
        Ok(())
    }

    async fn download(&self, data: StorageInfo<'_>) -> StorageResult<Vec<u8>> {
        self.client
            .download_object(
                &GetObjectRequest {
                    bucket: data.bucket.to_string(),
                    object: data.url,
                    ..Default::default()
                },
                &Range::default(),
            )
            .await
            .map_err(Into::into)
    }

    async fn update_credentials(&mut self, _token: TokenWrap) -> StorageResult<()> {
        todo!()
    }
}
