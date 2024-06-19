use std::{collections::HashMap, fmt::Debug};

use log::{debug, info, warn};
use rusoto_core::{credential::StaticProvider, Region};
use rusoto_s3::{
    DeleteObjectRequest, GetObjectOutput, GetObjectRequest, ListObjectsV2Request, Object, PutObjectRequest, S3Client,
    S3,
};
use rusoto_sts::{AssumeRoleWithWebIdentityRequest, Credentials, Sts, StsClient};
use tokio::io::AsyncReadExt;

use super::*;
use crate::{errors::StorageResult, models::TokenWrap};

#[derive(Clone)]
pub struct AwsClient {
    s3_client: S3Client,
    pub sub: String,
    messages: HashMap<String, Object>,
}

impl Debug for AwsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AwsClient")
            .field("sub", &self.sub)
            .field("messages", &self.messages)
            .finish()
    }
}

#[async_trait::async_trait]
impl Storage for AwsClient {
    type FileInfo = GetObjectOutput;
    type File = Object;

    async fn list_objects(&self, info: StorageInfo<'_>) -> StorageResult<Vec<FileInfo>> {
        let get_object_request = ListObjectsV2Request {
            bucket: info.bucket.to_string(),
            prefix: Some(info.url),
            ..Default::default()
        };
        let objects = self
            .s3_client
            .list_objects_v2(get_object_request)
            .await
            .map_err(StorageError::from)?;

        Ok(objects
            .contents
            .unwrap_or_default()
            .iter()
            .map(|f| FileInfo {
                name: f.key.clone().unwrap_or_default(),
                owner: f.owner.clone().and_then(|o| o.id).unwrap_or_default(),
                last_modified: f.last_modified.clone().unwrap_or_default(),
                metadata: None,
            })
            .collect())
    }

    async fn delete(&self, info: StorageInfo<'_>) -> StorageResult<()> {
        let request = DeleteObjectRequest {
            bucket: info.bucket.to_string(),
            key: info.url,
            ..Default::default()
        };
        let _objects = self
            .s3_client
            .delete_object(request)
            .await
            .map_err(StorageError::from)?;
        Ok(())
    }

    async fn upload(&self, info: StorageInfo<'_>) -> StorageResult<()> {
        self.s3_client
            .put_object(PutObjectRequest {
                bucket: info.bucket.to_owned(),
                key: info.url,
                body: Some(info.data.unwrap().into()),
                ..Default::default()
            })
            .await
            .map_err(StorageError::from)?;
        Ok(())
    }

    async fn download(&self, info: StorageInfo<'_>) -> StorageResult<Vec<u8>> {
        let get_object_request = GetObjectRequest {
            bucket: info.bucket.to_string(),
            key: info.url.clone(),
            ..Default::default()
        };

        match self.s3_client.get_object(get_object_request).await {
            Ok(object) => {
                let mut data_read = object
                    .body
                    .expect("There should be content to the requested data")
                    .into_async_read();
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

impl AwsClient {
    pub async fn new(jwt_token: TokenWrap) -> StorageResult<Self> {
        let sub = jwt_token.get_sub().unwrap();

        let credentials = assume_role(jwt_token.raw(), &sub)
            .await
            .expect("Should be able to assume role");

        let credentials_provider = StaticProvider::new(
            credentials.access_key_id.clone(),
            credentials.secret_access_key.clone(),
            Some(credentials.session_token.clone()),
            None,
        );

        let s3_client = S3Client::new_with(
            rusoto_core::HttpClient::new().unwrap(),
            credentials_provider,
            Region::UsEast1,
        );

        Ok(AwsClient {
            s3_client,
            sub,
            messages: HashMap::new(),
        })
    }

    pub async fn upload_message(&self, key: String, data: Vec<u8>) -> StorageResult<bool> {
        let mut uploaded = false;
        if !self.messages.contains_key(&key) {
            let _put_object_request = PutObjectRequest {
                bucket: MESSAGE_PATH.to_string(),
                key,
                body: Some(data.into()),
                ..Default::default()
            };

            info!("Self messages: {:?}", self.messages.len());

            // info!("Client uploading message");
            // self.s3_client.put_object(put_object_request).await.expect("failed to put object");
            uploaded = true;
        }
        Ok(uploaded)
    }

    pub async fn get_messages(&mut self) -> StorageResult<Vec<Object>> {
        let get_object_request = ListObjectsV2Request {
            bucket: MESSAGE_PATH.to_string(),
            ..Default::default()
        };
        let mut retrieved: Vec<Object> = Vec::new();
        match self.s3_client.list_objects_v2(get_object_request).await {
            Ok(messages) => {
                if let Some(object_list) = messages.contents {
                    for object in object_list.iter() {
                        let key = object.key.clone().unwrap();
                        self.messages.insert(key.clone(), object.clone());
                        retrieved.push(object.clone());
                    }
                }
            }
            Err(e) => {
                info!("Error with s3 client: {}", e);
            }
        }
        info!("Self messages: {:?}", self.messages.len());
        Ok(retrieved)
    }
}

async fn assume_role(token: &str, sub: &str) -> StorageResult<Credentials> {
    let creds = StaticProvider::new_minimal("dummy".to_string(), "dummy".to_string());
    let sts_client = StsClient::new_with(
        rusoto_core::HttpClient::new().expect("Failed to create HTTP client"),
        creds,
        Region::UsEast1,
    );
    // println!("Token: {}", token);
    debug!("Sub: {}", sub);
    let assume_role_request = AssumeRoleWithWebIdentityRequest {
        role_arn: "arn:aws:iam::071771013126:role/KeycloakAccess".to_string(),
        web_identity_token: token.to_string(),
        role_session_name: format!("session-{}", sub),
        ..Default::default()
    };
    debug!("Assuming role with openid");
    let assume_role_result = sts_client
        .assume_role_with_web_identity(assume_role_request)
        .await
        .map_err(|e| {
            warn!("Error with assuming role: {}", e);
            e
        })
        .expect("assuming role failed");
    // println!("Assumed the role: {:?}", assume_role_result.assumed_role_user);
    Ok(assume_role_result.credentials.unwrap())
}
