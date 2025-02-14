// src/errors/api_error.rs

use log::error;
use rocket_okapi::okapi::schemars;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::errors::UserError;

pub type ApiResult<T> = core::result::Result<T, ApiError>;

#[derive(Clone, Debug, Error, schemars::JsonSchema, Serialize, Deserialize)]
pub enum ApiError {
    #[error("Reqwest Error")]
    ReqwestError(String),

    #[error("Bad Request")]
    BadRequest,
    #[error("Internal Server Error")]
    InternalServerError,

    #[error("Internal Server User Error: {0}")]
    InternalUserError(#[from] UserError),

    #[error("Invalid API url {0}")]
    NotFound(String),

    #[error("ResponseError for url {url}, Code={code}, text={text}")]
    ResponseError { code: u16, text: String, url: String },

    #[error("Guardian {0}")]
    Guardian(String),

    #[error("Serde {0}")]
    Serde(String),
}

impl From<url::ParseError> for ApiError {
    fn from(error: url::ParseError) -> Self {
        Self::NotFound(error.to_string())
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serde(error.to_string())
    }
}
