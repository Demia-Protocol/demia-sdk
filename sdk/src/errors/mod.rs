// src/errors/configuration

mod analytics;
mod api_error;
mod identification_error;
mod node;
mod secret;
mod storage;
mod user_error;

pub use analytics::{AnalyticsError, AnalyticsResult};
pub use api_error::{ApiError, ApiResult};
pub use identification_error::{IdentityError, IdentityResult};
use log::warn;
pub use node::{NodeError, NodeResult};
pub use secret::{SecretError, SecretResult};
pub use storage::{StorageError, StorageResult};
pub use streams::Error as StreamsError;
use thiserror::Error;
pub use user_error::{UserError, UserResult};
use serde::{Deserialize, Serialize};

pub type SdkResult<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, Error, schemars::JsonSchema, Serialize, Deserialize)]
pub enum Error {
    #[error("Node Service Error: {0}")]
    Node(#[from] NodeError),

    // Dont think we want user error here
    //#[error("User Service Error: {0}")]
    // User(#[from] UserError),
    #[error("Identity Service Error: {0}")]
    Identity(#[from] IdentityError),

    #[error("API Service Error: {0}")]
    Api(#[from] ApiError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Secret error: {0}")]
    Secret(#[from] SecretError),

    #[error("Streams error: {0}")]
    Streams(String),

    #[error("Alvarium annotator Error: {0}")]
    AlvariumAnnotator(String),

    #[error("Alvarium SDK Error: {0}")]
    AlvariumSdk(String),

    #[error("Hedera Error: {0}")]
    Hedera(String),
}

impl From<streams::Error> for Error {
    fn from(error: streams::Error) -> Self {
        warn!("Error: {}", error);
        Error::Streams(error.to_string())
    }
}

impl From<identity_demia::demia::Error> for Error {
    fn from(error: identity_demia::demia::Error) -> Self {
        warn!("Error: {}", error);
        Error::Identity(IdentityError::IdentityError(error.to_string()))
    }
}

impl From<identity_demia::core::Error> for Error {
    fn from(error: identity_demia::core::Error) -> Self {
        warn!("Error: {}", error);
        Error::Identity(IdentityError::IdentityCoreError(error.to_string()))
    }
}

impl From<identity_demia::verification::Error> for Error {
    fn from(error: identity_demia::verification::Error) -> Self {
        warn!("Error: {}", error);
        Error::Identity(IdentityError::IdentityVerificationError(error.to_string()))
    }
}

impl From<identity_demia::did::Error> for Error {
    fn from(error: identity_demia::did::Error) -> Self {
        warn!("Error: {}", error);
        Error::Identity(IdentityError::IdentityDIDError(error.to_string()))
    }
}

impl From<iota_sdk::client::stronghold::Error> for Error {
    fn from(error: iota_sdk::client::stronghold::Error) -> Self {
        warn!("Error: {}", error);
        Error::Identity(IdentityError::StrongholdError(error.to_string()))
    }
}

impl From<crate::iota_stronghold::ClientError> for Error {
    fn from(error: crate::iota_stronghold::ClientError) -> Self {
        warn!("Error: {}", error);
        Error::Identity(IdentityError::StrongholdClientError(error.to_string()))
    }
}

impl From<iota_sdk::client::Error> for Error {
    fn from(error: iota_sdk::client::Error) -> Self {
        warn!("Error: {}", error);
        Error::Node(NodeError::NodeClientError(error.to_string()))
    }
}

impl From<iota_sdk::types::block::Error> for Error {
    fn from(error: iota_sdk::types::block::Error) -> Self {
        warn!("Error: {}", error);
        Error::Node(NodeError::NodeClientBlockError(error.to_string()))
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        warn!("Error: {}", error);
        Error::Api(ApiError::ReqwestError(error.to_string()))
    }
}

impl From<alvarium_annotator::Error> for Error {
    fn from(error: alvarium_annotator::Error) -> Self {
        warn!("Error: {}", error);
        Error::AlvariumAnnotator(error.to_string())
    }
}
impl From<alvarium_sdk_rust::errors::Error> for Error {
    fn from(error: alvarium_sdk_rust::errors::Error) -> Self {
        warn!("Error: {}", error);
        Error::AlvariumSdk(error.to_string())
    }
}
