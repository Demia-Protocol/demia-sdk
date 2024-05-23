use std::collections::HashSet;

use identity_demia::{
    core::ToJson,
    crypto::{KeyPair, KeyType},
    demia::{DemiaDID, IotaClientExt, IotaDocument, IotaIdentityClientExt},
    verification::{MethodScope, VerificationMethod},
};
use iota_client::{
    block::{
        address::Address,
        output::{AliasOutputBuilder, Output},
    },
    crypto::keys::bip39,
    node_api::indexer::query_parameters::QueryParameter,
    secret::{stronghold::StrongholdSecretManager, SecretManager},
    storage::StorageProvider,
    Client as IdentityClient,
};
use log::{debug, info, warn};
use streams::id::did::{Location, STREAMS_VAULT};

use crate::{
    clients::ApiClient,
    configuration::{Configuration, StrongholdConfiguration},
    errors::{IdentityError, NodeError, UserError, UserResult as Result},
    models::{StreamsAddresses, VAULT_DOC_ID, VAULT_STREAMS_ADDRESSES},
    User,
};

pub struct UserIdentity {
    doc_id: DemiaDID,
    config: StrongholdConfiguration,
    client: IdentityClient,
    pub stronghold: SecretManager,
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
        user: &User,
        config: &Configuration,
        client: IdentityClient,
        mut stronghold_adapter: StrongholdSecretManager,
    ) -> Result<UserIdentity> {
        // if exists: ensure it is accessible (i.e. get pub key)
        // if not exists: create new stronghold
        // TODO: Check if snapshot can be accessed via keycloak vault

        info!("Checking if there is an existing vault in stronghold");
        // create identity instance storing stronghold dir and adapter
        let identity = match stronghold_adapter.get(VAULT_DOC_ID.as_bytes()).await? {
            Some(doc_id) => {
                info!("There is one, gonna try and return it");

                // TODO: Error
                let doc_id_str = String::from_utf8(doc_id).unwrap();
                let doc_id = DemiaDID::parse(doc_id_str)?;

                let stronghold = SecretManager::Stronghold(stronghold_adapter);
                UserIdentity {
                    stronghold,
                    config: config.stronghold.clone(),
                    doc_id,
                    client: client.clone(),
                }
            }
            None => {
                warn!("Could not find existing Identity doc id");
                // Try to create new one
                info!("Creating new identity");

                // Generate keys for doc and mnemonic for storage in stronghold
                let stronghold_keypair: KeyPair = KeyPair::new(KeyType::Ed25519)?;
                let mnemonic =
                    bip39::wordlist::encode(stronghold_keypair.private().as_ref(), &bip39::wordlist::ENGLISH)
                        .map_err(|_| IdentityError::StrongholdMnemonicError)?;
                debug!("Mnemonic for stronghold wallet [store somewhere safe]: {}", mnemonic);
                if let Err(e) = stronghold_adapter.store_mnemonic(mnemonic.clone()).await {
                    warn!("Could not store mnemonic in stronghold: {}", e);
                }

                // Check that balances exist for addresses otherwise error out (temp faucet running for tests)
                debug!("Getting Addresses");
                let mut stronghold = SecretManager::Stronghold(stronghold_adapter);
                let balance_address = check_balance(api, user, &client, &mut stronghold).await?;

                // Generate a new doc and store streams keys in it
                debug!("Creating doc");
                let doc_keypair = KeyPair::new(KeyType::Ed25519)?;
                let mut doc = new_doc(config, &client, &mut stronghold, &doc_keypair, balance_address).await?;
                let doc_id = doc.id().clone();
                debug!("Generating keys");
                generate_streams_keys(config, &mut stronghold, &mut doc, &doc_keypair).await?;
                info!("Publishing new doc: {}", doc_id);
                let id = publish_identity_doc(&client, doc, &mut stronghold, &config.identity.country).await?;

                // Return a UserIdentity object
                UserIdentity {
                    stronghold,
                    config: config.stronghold.clone(),
                    doc_id: id,
                    client: client.clone(),
                }
            }
        };

        // fetch identity via client
        let iota_doc = identity.doc().await?;

        // check that all vars are retrievable
        let signing_key = &config.stronghold.key_locations.signature_keys;
        let ke_key = &config.stronghold.key_locations.key_exchange_keys;

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

    pub async fn doc(&self) -> Result<IotaDocument> {
        Ok(self.client.resolve_did(&self.doc_id).await?)
    }

    pub fn take_adapter(self) -> Result<StrongholdSecretManager> {
        match self.stronghold {
            SecretManager::Stronghold(stronghold) => Ok(stronghold),
            _ => Err(IdentityError::StrongholdTypeUnknown)?,
        }
    }

    pub async fn vaulted_doc_id(&mut self) -> Result<DemiaDID> {
        if let SecretManager::Stronghold(stronghold) = &mut self.stronghold {
            return match stronghold.get(VAULT_DOC_ID.as_bytes()).await? {
                Some(id) => {
                    // TODO: Error
                    let id = String::from_utf8(id).unwrap();
                    let iota_did = DemiaDID::parse(id)?;
                    Ok(iota_did)
                }
                None => Err(IdentityError::MissingIdentityDoc.into()),
            };
        }
        Err(IdentityError::StrongholdTypeUnknown.into())
    }

    pub async fn store_streams_address(&mut self, address: streams::Address) -> Result<()> {
        info!("Storing streams addresses");
        let mut addresses = self.vaulted_streams_addresses().await?;
        addresses.0.insert(address.to_string());

        if let SecretManager::Stronghold(stronghold) = &mut self.stronghold {
            stronghold
                .insert(VAULT_STREAMS_ADDRESSES.as_bytes(), &addresses.to_json_vec().unwrap())
                .await?;
        }
        info!("Stored streams addresses");

        Ok(())
    }
    pub async fn vaulted_streams_addresses(&mut self) -> Result<StreamsAddresses> {
        if let SecretManager::Stronghold(stronghold) = &mut self.stronghold {
            return match stronghold.get(VAULT_STREAMS_ADDRESSES.as_bytes()).await? {
                Some(addresses) => {
                    // TODO: Error
                    match serde_json::from_slice(&addresses) {
                        Ok(addresses) => Ok(addresses),
                        Err(_) => Ok(StreamsAddresses(HashSet::new())),
                    }
                }
                None => Ok(StreamsAddresses(HashSet::new())),
            };
        }
        Err(IdentityError::StrongholdTypeUnknown.into())
    }
}

pub async fn check_balance(
    api: &ApiClient,
    user: &User,
    client: &IdentityClient,
    stronghold: &mut SecretManager,
) -> Result<Address> {
    let token_supply = client.get_token_supply().await?;
    let mut total_amount = 0;

    let addresses = client
        .get_addresses(stronghold)
        .with_range(0..1)
        .get_raw()
        .await
        .map_err(|e| NodeError::NodeClientError(e.to_string()))?;

    debug!("Getting outputs");
    let b32_address = addresses[0].to_bech32(client.get_bech32_hrp().await?);
    let output_ids = client
        .basic_output_ids(vec![QueryParameter::Address(b32_address.clone())])
        .await
        .map_err(|e| NodeError::NodeClientError(e.to_string()))?;

    let outputs = client.get_outputs(output_ids.items).await?;
    for output in outputs {
        let output = Output::try_from_dto(&output.output, token_supply)?;
        total_amount += output.amount();
    }

    if total_amount < required_funds() {
        warn!("Not enough balance, trying to request funds from faucet");
        // Request and return, user can press again manually
        api.request_balance(user, &addresses[0]).await?;
        return Err(IdentityError::InsufficientBalance(b32_address.to_string()).into());
    }

    info!("Enough balance exists for identity operations");
    Ok(addresses[0])
}

fn required_funds() -> u64 {
    // TODO: Calculate minimum required
    1000000
}

// Create a new IOTA DID document and publish it
async fn new_doc(
    config: &Configuration,
    did_client: &IdentityClient,
    stronghold: &mut SecretManager,
    doc_keypair: &KeyPair,
    output_address: Address,
) -> Result<IotaDocument> {
    // Create a new document with a base method
    let network_name = did_client.get_network_name().await?;
    let country = &config.identity.country;
    let mut doc = IotaDocument::new(country, &network_name.try_into()?);
    let method = VerificationMethod::new(
        doc.id().clone(),
        KeyType::Ed25519,
        doc_keypair.public(),
        &config.stronghold.key_locations.doc_keys,
    )?;
    doc.insert_method(method, MethodScope::VerificationMethod)?;

    // Create new alias output and publish it
    let output = did_client.new_did_output(output_address, doc, None).await?;
    Ok(did_client.publish_did_output(stronghold, output, country).await?)
}

// Create the Ed25519 and X25519 keys that will be used by the streams user and store them into the
// stronghold vaults
async fn generate_streams_keys(
    config: &Configuration,
    stronghold: &mut SecretManager,
    doc: &mut IotaDocument,
    doc_keypair: &KeyPair,
) -> Result<()> {
    let method = doc
        .resolve_method(&format!("#{}", config.stronghold.key_locations.doc_keys), None)
        .expect("Should be able to fetch method from newly made doc");

    let doc_key_location = Location::generic(STREAMS_VAULT, method.id().to_string());

    match stronghold {
        SecretManager::Stronghold(adapter) => {
            // Store keys in vault
            let vault = adapter.vault_client(STREAMS_VAULT).await?;
            vault
                .write_secret(doc_key_location, doc_keypair.private().as_ref().to_vec())
                .map_err(|e| {
                    IdentityError::StrongholdClientError(iota_client::Error::StrongholdClient(e).to_string())
                })?;

            // insert new methods
            let signing_kp = KeyPair::new(KeyType::Ed25519)?;
            let exchange_kp = KeyPair::new(KeyType::X25519)?;

            let signing_method = VerificationMethod::new(
                doc.id().clone(),
                KeyType::Ed25519,
                signing_kp.public(),
                &config.stronghold.key_locations.signature_keys,
            )?;
            let exchange_method = VerificationMethod::new(
                doc.id().clone(),
                KeyType::X25519,
                exchange_kp.public(),
                &config.stronghold.key_locations.key_exchange_keys,
            )?;

            let signing_key_location = Location::generic(STREAMS_VAULT, signing_method.id().to_string());
            let exchange_key_location = Location::generic(STREAMS_VAULT, exchange_method.id().to_string());

            // Store new methods in vault
            vault
                .write_secret(signing_key_location, signing_kp.private().as_ref().to_vec())
                .map_err(|e| {
                    IdentityError::StrongholdClientError(iota_client::Error::StrongholdClient(e).to_string())
                })?;
            vault
                .write_secret(exchange_key_location, exchange_kp.private().as_ref().to_vec())
                .map_err(|e| {
                    IdentityError::StrongholdClientError(iota_client::Error::StrongholdClient(e).to_string())
                })?;

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
    doc: IotaDocument,
    stronghold: &mut SecretManager,
    country_code: &isocountry::CountryCode,
) -> Result<DemiaDID> {
    // Resolve the latest output and update it with the given document, updating the storage deposit for
    // the new rent structure
    let alias_output = client.update_did_output(doc.clone()).await?;
    let rent_structure = client.get_rent_structure().await?;
    let alias_output = AliasOutputBuilder::from(&alias_output)
        .with_minimum_storage_deposit(rent_structure)
        .finish(client.get_token_supply().await?)?;

    // Publish the updated Alias Output.
    let updated = client
        .publish_did_output(stronghold, alias_output, country_code)
        .await?;

    if let SecretManager::Stronghold(adapter) = stronghold {
        adapter
            .insert(VAULT_DOC_ID.as_bytes(), updated.id().to_string().as_bytes())
            .await?;
        adapter.write_stronghold_snapshot(None).await?;
    }

    Ok(updated.id().clone())
}
