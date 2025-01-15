use std::{collections::HashSet, sync::Arc};

use isocountry::CountryCode;
use log::{debug, info, warn};
use tokio::sync::RwLock;

use super::TokenWrap;
use crate::{
    clients::ApiClient,
    configuration::{IdentityConfiguration, StrongholdConfiguration},
    errors::{Error as SdkError, IdentityError, IdentityResult as Result, NodeError, SdkResult},
    identity_demia::{
        core::{BaseEncoding, FromJson, ToJson},
        demia::{DemiaDID, DemiaDocument, IotaClientExt, IotaIdentityClientExt},
        verification::{MethodData, MethodScope, MethodType, VerificationMethod},
    },
    identity_did::DID,
    iota_sdk::{
        client::{
            Client as IdentityClient,
            api::GetAddressesOptions,
            node_api::indexer::query_parameters::QueryParameter,
            secret::{SecretManager, stronghold::StrongholdSecretManager},
            storage::StorageAdapter,
        },
        crypto::{
            keys::{bip39, x25519},
            signatures::ed25519::{PublicKey, SecretKey},
        },
        types::block::{address::Address, output::AliasOutputBuilder},
    },
    iota_stronghold::Location,
    models::{StreamsAddresses, VAULT_DOC_ID, VAULT_STREAMS_ADDRESSES},
    streams::id::did::STREAMS_VAULT,
};

pub struct UserIdentity {
    doc_id: DemiaDID,
    config: StrongholdConfiguration,
    client: IdentityClient,
    pub(crate) stronghold: Arc<RwLock<SecretManager>>,
}

impl std::fmt::Debug for UserIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("UserIdentity");
        d.field("doc_id", &self.doc_id).finish()
    }
}

impl UserIdentity {
    pub async fn new(
        api: &ApiClient,
        auth0_token: &TokenWrap,
        identity_config: &IdentityConfiguration,
        stronghold_config: &StrongholdConfiguration,
        client: IdentityClient,
        stronghold_adapter: StrongholdSecretManager,
    ) -> SdkResult<UserIdentity> {
        // if exists: ensure it is accessible (i.e. get pub key)
        // if not exists: create new stronghold
        // TODO: Check if snapshot can be accessed via keycloak vault

        info!("Checking if there is an existing vault in stronghold");
        // create identity instance storing stronghold dir and adapter

        let vault_doc_id = stronghold_adapter.get_bytes(VAULT_DOC_ID).await.map_err(|e| {
            warn!("Could not get vault doc id: {}", e);
            e
        })?;
        let identity = match vault_doc_id {
            Some(doc_id) => {
                info!("There is one, gonna try and return it");

                // TODO: Error
                let doc_id_str = String::from_utf8(doc_id)
                    .map_err(|_| IdentityError::IdentityError("invalid utf8 doc id".to_string()))?;
                let doc_id = DemiaDID::parse(doc_id_str);
                if let Ok(doc_id) = doc_id {
                    UserIdentity {
                        stronghold: Arc::new(RwLock::new(SecretManager::Stronghold(stronghold_adapter))),
                        config: stronghold_config.clone(),
                        doc_id,
                        client: client.clone(),
                    }
                } else {
                    // Most likely from a different network, or it was pruned.
                    // Throw error for now
                    return Err(IdentityError::IdentityDIDError("Failed to fetch doc".to_string()).into());
                }
            }
            None => {
                warn!("Could not find existing Identity doc id");
                // Try to create new one
                info!("Creating new identity");

                // Generate keys for doc and mnemonic for storage in stronghold
                let stronghold_private = SecretKey::generate().map_err(|_| IdentityError::StrongholdMnemonicError)?;
                let mnemonic = bip39::wordlist::encode(stronghold_private.as_slice(), &bip39::wordlist::ENGLISH)
                    .map_err(|_| IdentityError::StrongholdMnemonicError)?;
                debug!("Mnemonic for stronghold wallet [store somewhere safe]: {:?}", mnemonic);
                if let Err(e) = stronghold_adapter.store_mnemonic(mnemonic.clone()).await {
                    warn!("Could not store mnemonic in stronghold: {}", e);
                }

                // Check that balances exist for addresses otherwise error out (temp faucet running for tests)
                debug!("Getting Addresses");
                let mut stronghold = SecretManager::Stronghold(stronghold_adapter);
                let (balance_address, enough) = check_balance(api, auth0_token, &client, &mut stronghold).await?;
                if !enough {
                    return Err(SdkError::Identity(IdentityError::InsufficientBalance(
                        balance_address.to_string(),
                    )));
                }

                // Generate a new doc and store streams keys in it
                debug!("Creating doc");
                let doc_private = SecretKey::generate().map_err(|_| IdentityError::StrongholdMnemonicError)?;
                let mut doc = new_doc(
                    identity_config,
                    stronghold_config,
                    &client,
                    &mut stronghold,
                    &doc_private.public_key(),
                    balance_address,
                )
                .await?;
                let doc_id = doc.id().clone();
                debug!("Generating keys");
                generate_streams_keys(stronghold_config, &mut stronghold, &mut doc, &doc_private).await?;
                info!("Publishing new doc: {}", doc_id);
                let doc = publish_identity_doc(&client, doc, &mut stronghold, &identity_config.country).await?;

                // Return a UserIdentity object
                UserIdentity {
                    stronghold: Arc::new(RwLock::new(stronghold)),
                    config: stronghold_config.clone(),
                    doc_id: doc.id().clone(),
                    client: client.clone(),
                }
            }
        };

        // fetch identity via client
        let iota_doc = identity.doc().await;
        if let Err(_e) = iota_doc {
            // Most likely from a different network, or it was pruned.
            // Throw error for now
            return Err(IdentityError::IdentityDIDError("Failed to fetch doc".to_string()).into());
        }
        let iota_doc = iota_doc?;

        // check that all vars are retrievable
        let signing_key = &stronghold_config.key_locations.signature_keys;
        let ke_key = &stronghold_config.key_locations.key_exchange_keys;

        match iota_doc.resolve_method(signing_key, None) {
            Some(_) => {}
            None => return Err(IdentityError::MissingIdentityMethod(signing_key.clone()))?,
        }

        match iota_doc.resolve_method(ke_key, None) {
            Some(_) => {}
            None => return Err(IdentityError::MissingIdentityMethod(ke_key.clone()))?,
        }

        info!("IDENTITY: {}", identity.doc_id);
        // confirm keycloak identity
        Ok(identity)
    }

    pub fn config(&self) -> &StrongholdConfiguration {
        &self.config
    }

    pub fn doc_id(&self) -> &DemiaDID {
        &self.doc_id
    }

    pub async fn doc(&self) -> Result<DemiaDocument> {
        Ok(self.client.resolve_did(&self.doc_id).await?)
    }

    pub fn clone_stronghold(&self) -> Arc<RwLock<SecretManager>> {
        self.stronghold.clone()
    }

    pub fn set_stronghold(&mut self, stronghold: StrongholdSecretManager) {
        self.stronghold = Arc::new(RwLock::new(SecretManager::Stronghold(stronghold)))
    }

    pub async fn read_stronghold(&self) -> tokio::sync::RwLockReadGuard<SecretManager> {
        self.stronghold.read().await
    }

    pub async fn write_stronghold(&self) -> tokio::sync::RwLockWriteGuard<SecretManager> {
        self.stronghold.write().await
    }

    pub async fn set_stronghold_bytes(&self, key: &str, record: &[u8]) -> Result<()> {
        match &*self.write_stronghold().await {
            SecretManager::Stronghold(adapter) => {
                adapter.set_bytes(key, record).await?;
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    pub async fn get_stronghold_bytes<T: From<Vec<u8>>>(&self, key: &str) -> Result<Option<T>> {
        match &*self.read_stronghold().await {
            SecretManager::Stronghold(adapter) => match adapter.get_bytes(key).await? {
                Some(id) => Ok(Some(id.into())),
                None => Ok(None),
            },
            _ => unreachable!(),
        }
    }

    pub async fn get_stronghold_string(&self, key: &str) -> Result<Option<String>> {
        match &*self.read_stronghold().await {
            SecretManager::Stronghold(adapter) => match adapter.get::<String>(key).await? {
                Some(id) => Ok(Some(id)),
                None => Ok(None),
            },
            _ => unreachable!(),
        }
    }

    pub async fn vaulted_doc_id(&self) -> Result<DemiaDID> {
        match self.get_stronghold_bytes(VAULT_DOC_ID).await? {
            Some(bytes) => {
                let id = String::from_utf8(bytes).unwrap();
                info!("Found vault doc id: {}", id);
                let iota_did = DemiaDID::parse(id)?;
                Ok(iota_did)
            }
            None => Err(IdentityError::MissingIdentityDoc),
        }
    }

    pub async fn store_streams_address(&self, address: streams::Address) -> Result<()> {
        let mut addresses = self.vaulted_streams_addresses().await?;
        if addresses.0.contains(&address.to_string()) {
            return Ok(());
        };

        addresses.0.insert(address.to_string());

        self.set_stronghold_bytes(VAULT_STREAMS_ADDRESSES, &addresses.to_json_vec().unwrap())
            .await
    }

    pub async fn remove_streams_address(&self, address: &streams::Address) -> Result<()> {
        let mut addresses = self.vaulted_streams_addresses().await?;
        if !addresses.0.contains(&address.to_string()) {
            return Ok(());
        };

        addresses.0.remove(&address.to_string());

        self.set_stronghold_bytes(VAULT_STREAMS_ADDRESSES, &addresses.to_json_vec()?)
            .await
    }

    pub async fn vaulted_streams_addresses(&self) -> Result<StreamsAddresses> {
        match self.get_stronghold_bytes::<Vec<u8>>(VAULT_STREAMS_ADDRESSES).await? {
            Some(addresses) => match StreamsAddresses::from_json_slice(&addresses) {
                Ok(addresses) => Ok(addresses),
                Err(_) => Ok(StreamsAddresses(HashSet::new())),
            },
            None => Ok(StreamsAddresses(HashSet::new())),
        }
    }

    /// Publish the DID document using the client and stronghold attached to this user.
    /// Will fail if the doc id does not match this user
    pub async fn publish_doc(&mut self, doc: DemiaDocument) -> SdkResult<DemiaDocument> {
        if !doc.id().eq(&self.doc_id) {
            return Err(IdentityError::StrongholdTypeUnknown.into());
        }

        let doc_id = self.doc_id().clone();
        // Safe unwrap as this cannot be created without the check first
        publish_identity_doc(
            &self.client,
            doc,
            &mut *self.stronghold.write().await,
            &CountryCode::for_alpha3_caseless(doc_id.country_str()).unwrap(),
        )
        .await
    }
}

pub async fn check_balance(
    api: &ApiClient,
    auth0_token: &TokenWrap,
    client: &IdentityClient,
    stronghold: &mut SecretManager,
) -> SdkResult<(Address, bool)> {
    let addresses: Vec<identity_demia::demia::block::address::Bech32Address> = stronghold
        .generate_ed25519_addresses(
            GetAddressesOptions::from_client(client)
                .await
                .map_err(NodeError::from)?
                .with_range(0..1),
        )
        .await
        .map_err(|e| NodeError::NodeClientError(e.to_string()))?;

    debug!("Getting outputs");
    let b32_address = addresses[0];
    let output_ids = client
        .basic_output_ids(vec![QueryParameter::Address(b32_address)])
        .await
        .map_err(|e| {
            warn!("Could not get output ids: {}", e);
            NodeError::NodeClientError(e.to_string())
        })?;

    let mut total_amount = 0;
    let outputs = client
        .get_outputs(&output_ids.items)
        .await
        .map_err(|e| {
            warn!("could not get outputs: {}", e);
            e
        })
        .map_err(NodeError::from)?;
    for output in outputs {
        total_amount += output.output().amount();
    }

    if total_amount < required_funds() {
        warn!(
            "Not enough balance, trying to request funds from faucet for {}",
            addresses[0]
        );
        // Request and return, user can press again manually
        api.request_balance(auth0_token.raw(), &addresses[0])
            .await
            .map_err(|e| {
                warn!("Could not request balance: {}", e);
                e
            })?;
        return Ok((addresses[0].into_inner(), false));
    }

    info!("Enough balance exists for identity operations");
    Ok((addresses[0].into_inner(), true))
}

fn required_funds() -> u64 {
    // TODO: Calculate minimum required
    1000000
}

// Create a new IOTA DID document and publish it
async fn new_doc(
    identity_config: &IdentityConfiguration,
    stronghold_config: &StrongholdConfiguration,
    did_client: &IdentityClient,
    stronghold: &mut SecretManager,
    doc_public: &PublicKey,
    output_address: Address,
) -> SdkResult<DemiaDocument> {
    // Create a new document with a base method
    let network_name = did_client.get_network_name().await.map_err(NodeError::from)?;
    let country = &identity_config.country;
    let mut doc = DemiaDocument::new(country, &network_name.try_into()?);

    let method = VerificationMethod::builder(Default::default())
        .id(doc
            .id()
            .to_url()
            .join(format!("#{}", &stronghold_config.key_locations.doc_keys))?)
        .controller(doc.id().to_url().did().clone())
        .type_(MethodType::ED25519_VERIFICATION_KEY_2018)
        .data(MethodData::PublicKeyMultibase(BaseEncoding::encode_multibase(
            doc_public.as_slice(),
            None,
        )))
        .build()?;
    doc.insert_method(method, MethodScope::VerificationMethod)?;

    // Create new alias output and publish it
    let output = did_client.new_did_output(output_address, doc, None).await?;
    Ok(did_client.publish_did_output(stronghold, output, country).await?)
}

// Create the Ed25519 and X25519 keys that will be used by the streams user and store them into the
// stronghold vaults
async fn generate_streams_keys(
    config: &StrongholdConfiguration,
    stronghold: &mut SecretManager,
    doc: &mut DemiaDocument,
    doc_secret: &SecretKey,
) -> Result<()> {
    let method = doc
        .resolve_method(&format!("#{}", config.key_locations.doc_keys), None)
        .expect("Should be able to fetch method from newly made doc");

    let doc_key_location = Location::generic(STREAMS_VAULT, method.id().to_string());

    match stronghold {
        SecretManager::Stronghold(adapter) => {
            // Store keys in vault
            let vault = adapter.vault_client(STREAMS_VAULT).await?;
            vault.write_secret(doc_key_location, doc_secret.as_slice().to_vec().into())?;

            // insert new methods
            let signing_private = SecretKey::generate().map_err(|_| IdentityError::StrongholdMnemonicError)?;
            let exchange_private = x25519::SecretKey::generate().map_err(|_| IdentityError::StrongholdMnemonicError)?;

            let signing_method = VerificationMethod::builder(Default::default())
                .id(doc
                    .id()
                    .to_url()
                    .join(format!("#{}", &config.key_locations.signature_keys))?)
                .controller(doc.id().to_url().did().clone())
                .type_(MethodType::ED25519_VERIFICATION_KEY_2018)
                .data(MethodData::PublicKeyMultibase(BaseEncoding::encode_multibase(
                    &signing_private.public_key(),
                    None,
                )))
                .build()?;

            let exchange_method = VerificationMethod::builder(Default::default())
                .id(doc
                    .id()
                    .to_url()
                    .join(format!("#{}", &config.key_locations.key_exchange_keys))?)
                .controller(doc.id().to_url().did().clone())
                .type_(MethodType::X25519_KEY_AGREEMENT_KEY_2019)
                .data(MethodData::PublicKeyMultibase(BaseEncoding::encode_multibase(
                    &exchange_private.public_key(),
                    None,
                )))
                .build()?;

            let signing_key_location = Location::generic(STREAMS_VAULT, signing_method.id().to_string());
            let exchange_key_location = Location::generic(STREAMS_VAULT, exchange_method.id().to_string());

            // Store new methods in vault
            vault.write_secret(signing_key_location, signing_private.as_slice().to_vec().into())?;
            vault.write_secret(exchange_key_location, exchange_private.to_bytes().to_vec().into())?;

            // Insert methods into document
            doc.insert_method(signing_method, MethodScope::VerificationMethod)?;
            doc.insert_method(exchange_method, MethodScope::VerificationMethod)?;

            adapter.write_stronghold_snapshot(None).await?;
            Ok(())
        }
        _ => Err(IdentityError::StrongholdTypeUnknown)?,
    }
}

async fn publish_identity_doc(
    client: &IdentityClient,
    doc: DemiaDocument,
    stronghold: &mut SecretManager,
    country_code: &isocountry::CountryCode,
) -> SdkResult<DemiaDocument> {
    // Resolve the latest output and update it with the given document, updating the storage deposit for
    // the new rent structure
    let alias_output = client.update_did_output(doc.clone()).await?;
    let rent_structure = client.get_rent_structure().await?;
    let alias_output = AliasOutputBuilder::from(&alias_output)
        .with_minimum_storage_deposit(rent_structure)
        .finish()
        .map_err(NodeError::from)?;

    // Publish the updated Alias Output.
    let updated = client
        .publish_did_output(stronghold, alias_output, country_code)
        .await?;

    if let SecretManager::Stronghold(adapter) = stronghold {
        adapter
            .set_bytes(VAULT_DOC_ID, updated.id().to_string().as_bytes())
            .await?;
        adapter.write_stronghold_snapshot(None).await?;
    }

    Ok(updated)
}
