use std::collections::HashMap;

use identity_demia::{credential::Credential, document::CoreDocument};
use serde::{Deserialize, Serialize};

use crate::utils::constants::*;

fn base_url() -> String {
    "http://localhost:14265".to_string()
}

fn stronghold_doc_keys() -> String {
    STRONGHOLD_DOC_KEYS.to_string()
}

fn stronghold_sig_keys() -> String {
    STRONGHOLD_SIG_KEYS.to_string()
}

fn stronghold_ke_keys() -> String {
    STRONGHOLD_KE_KEYS.to_string()
}

fn streams_backup() -> String {
    "user.bin".to_string()
}

fn level_info() -> String {
    "info".to_string()
}

fn debug_location() -> String {
    "log.out".to_string()
}

fn local_api() -> String {
    LOCAL_API.to_string()
}

fn retriever_api() -> String {
    RETRIEVER_API.to_string()
}

fn guardian_api() -> String {
    GUARDIAN_API.to_string()
}

fn secrets_api() -> String {
    SECRETS_API.to_string()
}

fn public_bucket_path() -> String {
    PUBLIC_BUCKET_PATH.to_string()
}

fn protected_bucket_path() -> String {
    PROTECTED_BUCKET_PATH.to_string()
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BaseConfiguration {
    #[serde(default)]
    pub logging: LoggingConfiguration,
    #[serde(default)]
    pub application: ApplicationConfiguration,
    #[serde(default)]
    pub streams: StreamsConfiguration,
    #[serde(default)]
    pub stronghold: StrongholdConfiguration,
    #[serde(default)]
    pub identity: IdentityConfiguration,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoggingConfiguration {
    #[serde(default = "level_info")]
    pub level: String,
    #[serde(default = "debug_location")]
    pub debug_location: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ApplicationConfiguration {
    pub username: String,
    #[serde(default = "local_api")]
    pub amazon_api: String,
    #[serde(default = "local_api")]
    pub local_api: String,
    pub use_local_api: bool,
    #[serde(default = "secrets_api")]
    pub secrets_api: String,
    #[serde(default = "retriever_api")]
    pub retriever_api: String,
    #[serde(default = "guardian_api")]
    pub guardian_api: String,

    #[serde(default = "public_bucket_path")]
    pub public_bucket_path: String,
    #[serde(default = "protected_bucket_path")]
    pub protected_bucket_path: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StreamsConfiguration {
    #[serde(default)]
    pub client: ClientConfiguration,
    #[serde(default = "streams_backup")]
    pub backup_path: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StrongholdConfiguration {
    #[serde(default)]
    pub key_locations: StrongholdKeyFragments,
    #[serde(default)]
    pub path: String,
    // This should be env locked for security
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfiguration {
    #[serde(default)]
    pub client: ClientConfiguration,
    #[serde(default)]
    pub faucet: ClientConfiguration,
    pub country: isocountry::CountryCode,
}

impl Default for IdentityConfiguration {
    fn default() -> Self {
        Self {
            client: Default::default(),
            faucet: Default::default(),
            country: isocountry::CountryCode::USA,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClientConfiguration {
    #[serde(default = "base_url")]
    pub url: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StrongholdKeyFragments {
    #[serde(default = "stronghold_doc_keys")]
    pub doc_keys: String,
    #[serde(default = "stronghold_sig_keys")]
    pub signature_keys: String,
    #[serde(default = "stronghold_ke_keys")]
    pub key_exchange_keys: String,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuardianConfigs {
    pub operator_id: String,
    pub username: String,
    pub did_doc: Option<CoreDocument>,
    pub policy: String,
    pub vcs: HashMap<String, (String, Credential)>,
}
