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
