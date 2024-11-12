use alvarium_annotator::SignProvider;
use alvarium_sdk_rust::config::SignatureInfo;
use iota_sdk::{
    client::secret::{SecretManager, stronghold::StrongholdSecretManager},
    crypto::signatures::ed25519::Signature,
};
use iota_stronghold::Location;
use streams::id::did::STREAMS_VAULT;

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
    stronghold_password: String,
    stronghold_signature_key_path: String,
}

impl StrongholdProvider {
    pub fn new(
        config: &SignatureInfo,
        stronghold_password: String,
        stronghold_signature_key_path: String,
    ) -> Result<Self, StrongholdProviderError> {
        Ok(StrongholdProvider {
            config: config.clone(),
            stronghold_password,
            stronghold_signature_key_path,
        })
    }

    fn get_adapter(&self) -> Result<SecretManager, StrongholdProviderError> {
        println!(
            "GetAdapter: {}, {}",
            self.stronghold_password, self.config.private_key_info.path
        );
        let stronghold_adapter = StrongholdSecretManager::builder()
            .password(self.stronghold_password.clone())
            .build(self.config.private_key_info.path.clone())?;
        Ok(SecretManager::Stronghold(stronghold_adapter))
    }
}

#[async_trait::async_trait]
impl SignProvider for StrongholdProvider {
    type Error = StrongholdProviderError;

    async fn sign(&self, content: &[u8]) -> Result<String, Self::Error> {
        // Sign using the key stored in Stronghold
        if let SecretManager::Stronghold(adapter) = self.get_adapter()? {
            println!(
                "StrongholdProvider.sign() adapter: {}",
                self.stronghold_signature_key_path.clone()
            );
            let location = Location::generic(STREAMS_VAULT, self.stronghold_signature_key_path.clone());
            let signature = adapter.ed25519_sign(location, content).await?;
            return Ok(hex::encode(signature.to_bytes()));
        }
        Err(Self::Error::StrongholdAdapterError)
    }

    async fn verify(&self, content: &[u8], signed: &[u8]) -> Result<bool, Self::Error> {
        let sig = get_signature(signed)?;
        // Fetch public key from Stronghold and verify the signature
        if let SecretManager::Stronghold(adapter) = self.get_adapter()? {
            let location = Location::generic(STREAMS_VAULT, self.stronghold_signature_key_path.clone());
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
