use serde::{Deserialize, Serialize};

fn base_url() -> String {
    "http://localhost:14265".to_string()
}

fn stronghold_doc_keys() -> String {
    "streams_doc_keys".to_string()
}

fn stronghold_sig_keys() -> String {
    "streams_sig_keys".to_string()
}

fn stronghold_ke_keys() -> String {
    "streams_ke_keys".to_string()
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
    "http://localhost:1111".to_string()
}

fn secrets_api() -> String {
    "https://auth.demia-testing-domain.com/realms/DemiaTest".to_string()
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
    pub country: isocountry::CountryCode,
}

impl Default for IdentityConfiguration {
    fn default() -> Self {
        Self {
            client: Default::default(),
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
