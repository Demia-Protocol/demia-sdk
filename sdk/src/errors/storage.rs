use rocket_okapi::okapi::schemars;
use rusoto_core::RusotoError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type StorageResult<T> = core::result::Result<T, StorageError>;

#[derive(Clone, Debug, Error, schemars::JsonSchema, Serialize, Deserialize)]
pub enum StorageError {
    #[error("AWS Client error: {0}")]
    AwsClientError(String),
    #[error("GC error: {0}")]
    GoogleCloud(String),
    #[error("File error: {0}")]
    File(String),
    #[error("Should be able to assume role")]
    Credentials,
    #[error("Download file request was denied due to no update needed")]
    NotModified,
}

impl From<google_cloud_storage::http::Error> for StorageError {
    fn from(error: google_cloud_storage::http::Error) -> Self {
        Self::GoogleCloud(format!("{}", error))
    }
}

impl<T, R> From<aws_sdk_s3::error::SdkError<T, R>> for StorageError {
    fn from(value: aws_sdk_s3::error::SdkError<T, R>) -> Self {
        Self::AwsClientError(format!("AWS SDK error: {}", value))
    }
}

impl From<RusotoError<rusoto_s3::HeadObjectError>> for StorageError {
    fn from(error: RusotoError<rusoto_s3::HeadObjectError>) -> Self {
        Self::AwsClientError(format!("Metadata Object: {}", error))
    }
}

impl From<RusotoError<rusoto_s3::DeleteObjectError>> for StorageError {
    fn from(error: RusotoError<rusoto_s3::DeleteObjectError>) -> Self {
        Self::AwsClientError(format!("Delete Object: {}", error))
    }
}

impl From<RusotoError<rusoto_s3::ListObjectsV2Error>> for StorageError {
    fn from(error: RusotoError<rusoto_s3::ListObjectsV2Error>) -> Self {
        Self::AwsClientError(format!("List Object: {}", error))
    }
}

impl From<RusotoError<rusoto_s3::PutObjectError>> for StorageError {
    fn from(error: RusotoError<rusoto_s3::PutObjectError>) -> Self {
        Self::AwsClientError(format!("Put Object: {}", error))
    }
}

impl From<RusotoError<rusoto_s3::GetObjectError>> for StorageError {
    fn from(error: RusotoError<rusoto_s3::GetObjectError>) -> Self {
        Self::AwsClientError(format!("Get Object: {}", error))
    }
}
