use std::sync::Arc;

use alvarium_annotator::SignProvider;
use alvarium_sdk_rust::{config::SignatureInfo, errors::Error};
use iota_sdk::{client::secret::SecretManager, crypto::signatures::ed25519::Signature};
use iota_stronghold::Location;
use streams::id::did::STREAMS_VAULT;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error, schemars::JsonSchema)]
pub enum StrongholdProviderError {
    /// Crypto.rs error
    #[error("{0}")]
    Stronghold(String),

    #[error("Unsupported adapter")]
    StrongholdAdapterError,

    #[error("Array is not the correct size: {0}/{1}")]
    IncorrectKeySize(usize, usize),
}

impl From<iota_sdk::client::stronghold::Error> for StrongholdProviderError {
    fn from(error: iota_sdk::client::stronghold::Error) -> Self {
        log::warn!("Error: {}", error);
        Self::Stronghold(error.to_string())
    }
}

pub struct StrongholdProvider {
    config: SignatureInfo,
    // pub_stronghold_password: String,
    // pub_stronghold_signature_key_path: String,
    // private_stronghold_password: String,
    // private_stronghold_signature_key_path: String,
    manager: Arc<RwLock<SecretManager>>,
}

impl StrongholdProvider {
    pub fn new(
        config: &SignatureInfo,
        // pub_stronghold_password: String,
        // pub_stronghold_signature_key_path: String,
        // private_stronghold_password: String,
        // private_stronghold_signature_key_path: String,
        manager: Arc<RwLock<SecretManager>>,
    ) -> Result<Self, StrongholdProviderError> {
        Ok(StrongholdProvider {
            config: config.clone(),
            // pub_stronghold_password,
            // pub_stronghold_signature_key_path,
            // private_stronghold_password,
            // private_stronghold_signature_key_path,
            manager,
        })
    }
}

// TODO replace with StrongholdAdapter
#[async_trait::async_trait]
impl SignProvider for StrongholdProvider {
    type Error = Error;

    async fn sign(&self, content: &[u8]) -> Result<String, Self::Error> {
        // Sign using the key stored in Stronghold
        if let SecretManager::Stronghold(adapter) = &*self.manager.read().await {
            println!(
                "StrongholdProvider.sign() adapter: {}",
                self.config.private_key_stronghold.path.clone()
            );
            let location = Location::generic(STREAMS_VAULT, self.config.private_key_stronghold.path.clone());
            let signature = adapter
                .ed25519_sign(location, content)
                .await
                .map_err(|e| Self::Error::External(Box::new(e)))?;
            return Ok(hex::encode(signature.to_bytes()));
        }
        unreachable!()
    }

    async fn verify(&self, content: &[u8], signed: &[u8]) -> Result<bool, Self::Error> {
        let sig = get_signature(signed)?;
        // Fetch public key from Stronghold and verify the signature
        if let SecretManager::Stronghold(adapter) = &*self.manager.read().await {
            let location = Location::generic(STREAMS_VAULT, self.config.private_key_stronghold.path.clone());
            let pub_key = adapter
                .ed25519_public_key(location)
                .await
                .map_err(|e| Self::Error::External(Box::new(e)))?;
            return Ok(pub_key.verify(&sig, content));
        }
        unreachable!()
    }
}

pub(crate) fn get_signature(signature: &[u8]) -> Result<Signature, Error> {
    match <[u8; Signature::LENGTH]>::try_from(signature) {
        Ok(resized) => Ok(Signature::from_bytes(resized)),
        Err(_) => Err(Error::IncorrectKeySize(signature.len(), Signature::LENGTH)),
    }
}
