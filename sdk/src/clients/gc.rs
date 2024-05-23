use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        download::Range,
        get::GetObjectRequest,
        upload::{Media, UploadObjectRequest, UploadType},
        Object,
    },
};

use crate::{
    clients::{Storage, StorageInfo},
    errors::{StorageError, StorageResult},
};

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
    type FileInfo = UploadObjectRequest;
    type File = Object;

    async fn list_objects(&self, bucket: String) -> StorageResult<Vec<Self::FileInfo>> {
        // TODO: Vec<GetObjectOutput> -> into type File
        let _ = bucket.split(':');
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
}