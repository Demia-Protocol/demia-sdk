use rocket_okapi::okapi::schemars;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type NodeResult<T> = core::result::Result<T, NodeError>;

#[derive(Clone, Debug, Error, schemars::JsonSchema, Serialize, Deserialize)]
pub enum NodeError {
    #[error("An error has occurred while generating identity client. Cause: {0}")]
    NodeClientError(String),

    #[error("An error occurred with the identity client block formation. Cause: {0}")]
    NodeClientBlockError(String),

    #[error("An error occurred with DTO response conversion. Cause: {0}")]
    NodeClientDTOError(String),

    #[error("Node is not synced for connection")]
    NodeSyncIncomplete,
}

impl From<iota_sdk::client::Error> for NodeError {
    fn from(error: iota_sdk::client::Error) -> Self {
        log::warn!("Error: {}", error);
        NodeError::NodeClientError(error.to_string())
    }
}

impl From<iota_sdk::types::block::Error> for NodeError {
    fn from(error: iota_sdk::types::block::Error) -> Self {
        log::warn!("Error: {}", error);
        NodeError::NodeClientBlockError(error.to_string())
    }
}
