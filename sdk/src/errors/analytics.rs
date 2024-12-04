// src/errors/user_error.rs

use log::error;
use rocket_okapi::okapi::schemars;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type AnalyticsResult<T> = core::result::Result<T, AnalyticsError>;

#[derive(Clone, Debug, Error, schemars::JsonSchema, Serialize, Deserialize)]
pub enum AnalyticsError {
    #[error("No vault client stored in user instance")]
    NoVaultClient,
    #[error("No profile found by the name of {0}")]
    NoProfileFound(String),
}
