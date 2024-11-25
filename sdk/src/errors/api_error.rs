// src/errors/api_error.rs

use log::error;
use rocket_okapi::okapi::schemars;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type ApiResult<T> = core::result::Result<T, ApiError>;

#[derive(Clone, Debug, Error, schemars::JsonSchema, Serialize, Deserialize)]
pub enum ApiError {
    #[error("Reqwest Error")]
    ReqwestError(String),

    #[error("Bad Request")]
    BadRequest,
    #[error("Internal Server Error")]
    InternalServerError,

    #[error("Invalid API url {0}")]
    NotFound(String),

    #[error("ResponseError for url {url}, Code={code}, text={text}")]
    ResponseError { code: u16, text: String, url: String }, // ... other API-related error variants ...
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error.to_string())
    }
}
