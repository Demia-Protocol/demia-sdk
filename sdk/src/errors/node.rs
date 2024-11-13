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
