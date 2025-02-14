use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type GuardianResult<T> = Result<T, GuardianError>;

#[derive(Clone, Debug, Error, schemars::JsonSchema, Serialize, Deserialize)]
pub enum GuardianError {
    #[error("No Guardian Configurations registered for site {0}")]
    NoGuardianConfig(String),

    #[error("DID document not found in guardian configurations in site {0}")]
    NoGuardianDoc(String),

    #[error("Provided key does not appear to own the DID linked to the account.")]
    NoMatchingGuardianKey,
}