use rocket_okapi::okapi::schemars;
use rusoto_core::RusotoError;
use thiserror::Error;

pub type StorageResult<T> = core::result::Result<T, StorageError>;

#[derive(Debug, Error, schemars::JsonSchema)]
pub enum StorageError {
    #[error("AWS Client error: {0}")]
    AwsClientError(String),
    #[error("GC error: {0}")]
    GoogleCloud(String),
}

impl From<google_cloud_storage::http::Error> for StorageError {
    fn from(error: google_cloud_storage::http::Error) -> Self {
        Self::GoogleCloud(format!("{}", error))
    }
}

impl From<RusotoError<rusoto_s3::ListObjectsError>> for StorageError {
    fn from(error: RusotoError<rusoto_s3::ListObjectsError>) -> Self {
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