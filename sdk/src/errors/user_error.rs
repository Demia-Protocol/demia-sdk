// src/errors/user_error.rs

use log::error;
use rocket_okapi::okapi::schemars;
use thiserror::Error;

pub type UserResult<T> = core::result::Result<T, UserError>;

#[derive(Debug, Error, schemars::JsonSchema)]
pub enum UserError {
    #[error("Streams node api client failed to start")]
    StreamsAPIClientError,

    #[error("A Streams error has occurred: {0}")]
    StreamsError(String),

    #[error("No streams user client stored in user instance")]
    NoStreamsUserClient,

    #[error("No identity stored in user instance")]
    NoUserIdentity,

    #[error("No storage client stored in user instance")]
    NoStorageClient,

    #[error("No vault client stored in user instance")]
    NoVaultClient,
}
