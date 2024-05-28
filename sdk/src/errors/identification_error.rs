// src/errors/identification_error.rs

use rocket_okapi::okapi::schemars;
use thiserror::Error;

pub type IdentityResult<T> = core::result::Result<T, IdentityError>;

#[derive(Debug, Error, schemars::JsonSchema)]
pub enum IdentityError {
    #[error("No password in vault")]
    NoStrongholdSecret,
    #[error("An Identity IdentityError has occurred: {0}")]
    IdentityError(String),

    #[error("An Identity Core IdentityError has occurred: {0}")]
    IdentityCoreError(String),

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

    #[error("Mnemonic could not be generated from provided keypair")]
    StrongholdMnemonicError,

    #[error("Not enough balance found with address {0}")]
    InsufficientBalance(String),

    #[error("Identity document id could not be found within stronghold")]
    MissingIdentityDoc,

    #[error("Identity document does not contain a method for fragment {0}")]
    MissingIdentityMethod(String),
}
