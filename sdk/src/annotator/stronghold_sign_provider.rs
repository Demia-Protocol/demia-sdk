use std::sync::Arc;

use alvarium_annotator::SignProvider;
use alvarium_sdk_rust::config::SignatureInfo;
use iota_sdk::{
    client::secret::{SecretManager, stronghold::StrongholdSecretManager},
    crypto::signatures::ed25519::Signature,
};
use iota_stronghold::Location;
use streams::id::did::STREAMS_VAULT;
use tokio::sync::RwLock;

use crate::models::UserIdentity;

#[derive(Debug, thiserror::Error)]
pub enum StrongholdProviderError {
    /// Crypto.rs error
    #[error("{0}")]
    Stronghold(#[from] iota_sdk::client::stronghold::Error),

    #[error("Unsupported adapter")]
    StrongholdAdapterError,

    #[error("Array is not the correct size: {0}/{1}")]
    IncorrectKeySize(usize, usize),
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

    fn get_adapter(&self) -> Result<SecretManager, StrongholdProviderError> {
        println!(
            "GetAdapter: {}, {}",
            self.config.private_key_info.path, self.config.private_key_stronghold.password
        );
        let stronghold_adapter = StrongholdSecretManager::builder()
            .password(self.config.private_key_stronghold.password.clone())
            .build(self.config.private_key_info.path.clone())?;
        Ok(SecretManager::Stronghold(stronghold_adapter))
    }
}

// TODO replace with StrongholdAdapter
#[async_trait::async_trait]
impl SignProvider for StrongholdProvider {
    type Error = crate::errors::Error;

    async fn sign(&self, content: &[u8]) -> Result<String, Self::Error> {
        // Sign using the key stored in Stronghold
        if let SecretManager::Stronghold(adapter) = &*self.manager.read().await {
            println!(
                "StrongholdProvider.sign() adapter: {}",
                self.config.private_key_stronghold.path.clone()
            );
            let location = Location::generic(STREAMS_VAULT, self.config.private_key_stronghold.path.clone());
            let signature = adapter.ed25519_sign(location, content).await?;
            return Ok(hex::encode(signature.to_bytes()));
        }
        Err(Self::Error::StrongholdAdapterError)
    }

    async fn verify(&self, content: &[u8], signed: &[u8]) -> Result<bool, Self::Error> {
        let sig = get_signature(signed)?;
        // Fetch public key from Stronghold and verify the signature
        if let SecretManager::Stronghold(adapter) = &*self.manager.read().await {
            let location = Location::generic(STREAMS_VAULT, self.config.private_key_stronghold.path.clone());
            let pub_key = adapter.ed25519_public_key(location).await?;
            return Ok(pub_key.verify(&sig, content));
        }
        Err(Self::Error::StrongholdAdapterError)
    }
}

pub(crate) fn get_signature(signature: &[u8]) -> Result<Signature, StrongholdProviderError> {
    match <[u8; Signature::LENGTH]>::try_from(signature) {
        Ok(resized) => Ok(Signature::from_bytes(resized)),
        Err(_) => Err(StrongholdProviderError::IncorrectKeySize(
            signature.len(),
            Signature::LENGTH,
        )),
    }
}
