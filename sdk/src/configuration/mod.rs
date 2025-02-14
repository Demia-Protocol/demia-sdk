mod config;

use std::fs::File;

pub use config::*;
use serde::de::DeserializeOwned;

use crate::errors::Error;

/// Builds a configuration object from either an environment variable or a fallback file.
///
/// 1. If the CONFIG_JSON environment variable is set, it tries to parse the configuration from it.
/// 2. Otherwise, it checks the CONFIG environment variable for a file path.
/// 3. If CONFIG is not set, it uses the provided fallback_path.
/// 4. It reads and deserializes the JSON configuration from the determined file.
/// 5. If the LOG_CONFIG environment variable is set, it logs the configuration details.
pub fn build_config<T: DeserializeOwned + std::fmt::Debug>(fallback_path: String) -> Result<T, Error> {
    let config = match std::env::var("CONFIG_JSON") {
        Ok(json_str) => serde_json::from_str(&json_str).expect("Failed to parse CONFIG_JSON environment variable"),
        Err(_) => {
            let config_file = std::env::var("CONFIG").unwrap_or(fallback_path);

            // Open the JSON file
            let config_file = File::open(config_file).map_err(|e| Error::Configuration(e.to_string()))?;

            // Parse the JSON file
            serde_json::from_reader(config_file).map_err(|e| Error::Configuration(e.to_string()))?
        }
    };

    if std::env::var("LOG_CONFIG").is_ok() {
        log::info!("Configuration: {:?}", config);
    }

    Ok(config)
}
