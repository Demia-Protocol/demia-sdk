// src/errors/identification_error.rs

use rocket_okapi::okapi::schemars;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type IdentityResult<T> = core::result::Result<T, IdentityError>;

#[derive(Clone, Debug, Error, schemars::JsonSchema, Serialize, Deserialize)]
pub enum IdentityError {
    #[error("No password in vault")]
    NoStrongholdSecret,
    #[error("An Identity IdentityError has occurred: {0}")]
    IdentityError(String),

    #[error("An Identity Core IdentityError has occurred: {0}")]
    IdentityCoreError(String),

    #[error("An Identity credentials error has occurred: {0}")]
    IdentityCredentialError(String),

    #[error("An Identity Verification IdentityError has occurred: {0}")]
    IdentityVerificationError(String),

    #[error("An Identity DID IdentityError has occurred: {0}")]
    IdentityDIDError(String),

    #[error("A Stronghold IdentityError has occurred: {0}")]
    StrongholdError(String),

    #[error("A Stronghold Client IdentityError has occurred: {0}")]
    StrongholdClientError(String),

    #[error("Stronghold type is unknown")]
    StrongholdTypeUnknown,

    #[error("Mnemonic could not be generated")]
    StrongholdMnemonicError,

    #[error("Not enough balance found with address {0}")]
    InsufficientBalance(String),

    #[error("Identity document id could not be found within stronghold")]
    MissingIdentityDoc,

    #[error("Identity document does not contain a method for fragment {0}")]
    MissingIdentityMethod(String),
}

impl IdentityError {
    pub fn is_missing_identity(&self) -> bool {
        match self {
            IdentityError::IdentityDIDError(err) => err.eq("Failed to fetch doc"),
            IdentityError::StrongholdError(err) => {
                err.eq("stronghold client error: error loading client data; no data present")
            }
            _ => false,
        }
    }
}

impl From<identity_demia::demia::Error> for IdentityError {
    fn from(error: identity_demia::demia::Error) -> Self {
        log::debug!("Error: {}", error);
        Self::IdentityError(error.to_string())
    }
}

impl From<identity_demia::core::Error> for IdentityError {
    fn from(error: identity_demia::core::Error) -> Self {
        log::debug!("Error: {}", error);
        Self::IdentityCoreError(error.to_string())
    }
}

impl From<identity_demia::verification::Error> for IdentityError {
    fn from(error: identity_demia::verification::Error) -> Self {
        log::debug!("Error: {}", error);
        Self::IdentityVerificationError(error.to_string())
    }
}

impl From<identity_demia::did::Error> for IdentityError {
    fn from(error: identity_demia::did::Error) -> Self {
        log::debug!("Error: {}", error);
        Self::IdentityDIDError(error.to_string())
    }
}

impl From<iota_sdk::client::stronghold::Error> for IdentityError {
    fn from(error: iota_sdk::client::stronghold::Error) -> Self {
        log::debug!("Error: {}", error);
        Self::StrongholdError(error.to_string())
    }
}

impl From<iota_stronghold::ClientError> for IdentityError {
    fn from(error: iota_stronghold::ClientError) -> Self {
        log::debug!("Error: {}", error);
        Self::StrongholdClientError(error.to_string())
    }
}
