pub mod serde;
pub use serde::*;

pub mod constants;
use base64::Engine;
pub use constants::*;
use iota_sdk::crypto::signatures::ed25519::SecretKey;

pub fn new_stronghold_key() -> String {
    let key = SecretKey::generate().expect("Shouldn't be a problem to generate a new key");
    let str = base64::engine::general_purpose::STANDARD.encode(key.as_slice());
    str
}

pub fn make_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// prints the context around json that failed to parse
pub fn print_error_context(json_slice: &[u8], error_message: &str) {
    log::info!("Error with metadata: {}", error_message);
    // Extract column number from the error message
    let column_prefix = "column ";
    if let Some(start) = error_message.find(column_prefix) {
        if let Some(column_str) = error_message[start + column_prefix.len()..].split_whitespace().next() {
            if let Ok(column) = column_str.parse::<usize>() {
                // Convert JSON slice to a string
                if let Ok(json_string) = String::from_utf8(json_slice.to_vec()) {
                    // Calculate start and end of the context slice
                    let start = column.saturating_sub(20);
                    let end = usize::min(column + 20, json_string.len());

                    // Print the 15-character buffer around the column
                    log::info!("Error context around column {}: {}", column, &json_string[start..end]);
                } else {
                    eprintln!("Failed to convert JSON slice to a valid UTF-8 string");
                }
            } else {
                eprintln!("Failed to parse column number from error message");
            }
        } else {
            eprintln!("Failed to extract column number from error message");
        }
    } else {
        eprintln!("Error message does not contain column information");
    }
}
