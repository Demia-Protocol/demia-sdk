use rocket_okapi::okapi::schemars;
#[cfg(feature = "aws_rusoto")]
use rusoto_core::RusotoError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type StorageResult<T> = core::result::Result<T, StorageError>;

#[derive(Clone, Debug, Error, schemars::JsonSchema, Serialize, Deserialize)]
pub enum StorageError {
    #[cfg(any(feature = "aws_rusoto", feature = "aws"))]
    #[error("AWS Client error: {0}")]
    AwsClientError(String),
    #[cfg(feature = "google_cloud")]
    #[error("GC error: {0}")]
    GoogleCloud(String),
    #[error("File error: {0}")]
    File(String),
    #[error("Should be able to assume role")]
    Credentials,
    #[error("Download file request was denied due to no update needed")]
    NotModified,
    #[error("Invalid name for file \"{0}\"")]
    InvalidName(String),
}

#[cfg(feature = "google_cloud")]
impl From<google_cloud_storage::http::Error> for StorageError {
    fn from(error: google_cloud_storage::http::Error) -> Self {
        Self::GoogleCloud(format!("{}", error))
    }
}

#[cfg(feature = "aws")]
impl<T: std::error::Error + 'static, R: std::fmt::Debug> From<aws_sdk_s3::error::SdkError<T, R>> for StorageError {
    fn from(value: aws_sdk_s3::error::SdkError<T, R>) -> Self {
        // Note: If it says no permission means there is no file, as we dont have listFiles permission.
        // And it will check for listfiles when it doesnt exist, so it doesnt leak permissions
        // as in error 403 vs 404
        log::debug!("{}", aws_sdk_s3::error::DisplayErrorContext(&value));
        Self::AwsClientError(format!("AWS SDK error: {}", value))
    }
}

#[cfg(feature = "aws_rusoto")]
impl From<RusotoError<rusoto_s3::HeadObjectError>> for StorageError {
    fn from(error: RusotoError<rusoto_s3::HeadObjectError>) -> Self {
        Self::AwsClientError(format!("Metadata Object: {}", error))
    }
}

#[cfg(feature = "aws_rusoto")]
impl From<RusotoError<rusoto_s3::DeleteObjectError>> for StorageError {
    fn from(error: RusotoError<rusoto_s3::DeleteObjectError>) -> Self {
        Self::AwsClientError(format!("Delete Object: {}", error))
    }
}

#[cfg(feature = "aws_rusoto")]
impl From<RusotoError<rusoto_s3::ListObjectsV2Error>> for StorageError {
    fn from(error: RusotoError<rusoto_s3::ListObjectsV2Error>) -> Self {
        Self::AwsClientError(format!("List Object: {}", error))
    }
}

#[cfg(feature = "aws_rusoto")]
impl From<RusotoError<rusoto_s3::PutObjectError>> for StorageError {
    fn from(error: RusotoError<rusoto_s3::PutObjectError>) -> Self {
        Self::AwsClientError(format!("Put Object: {}", error))
    }
}

#[cfg(feature = "aws_rusoto")]
impl From<RusotoError<rusoto_s3::GetObjectError>> for StorageError {
    fn from(error: RusotoError<rusoto_s3::GetObjectError>) -> Self {
        Self::AwsClientError(format!("Get Object: {}", error))
    }
}
