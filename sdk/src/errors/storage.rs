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
    #[error("File error: {0}")]
    File(String),
}

impl From<google_cloud_storage::http::Error> for StorageError {
    fn from(error: google_cloud_storage::http::Error) -> Self {
        Self::GoogleCloud(format!("{}", error))
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
