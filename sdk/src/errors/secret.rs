use rocket_okapi::okapi::schemars;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type SecretResult<T> = core::result::Result<T, SecretError>;

#[derive(Clone, Debug, Error, schemars::JsonSchema, Serialize, Deserialize)]
pub enum SecretError {
    #[error("AWS error: {0}")]
    Aws(String),
    #[error("Reqwest client error: {0}")]
    ReqwestError(String),

    #[error("No token found of type {0}")]
    TokenNotFound(String),

    #[error("JWT error {0}")]
    Jwt(String),
}

impl From<google_cloud_storage::http::Error> for SecretError {
    fn from(error: google_cloud_storage::http::Error) -> Self {
        Self::Aws(format!("{}", error))
    }
}

impl From<reqwest::Error> for SecretError {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error.to_string())
    }
}
