use std::{
    fmt::Debug,
    time::{SystemTime, UNIX_EPOCH},
};

use log::{debug, info};
use serde_json::Value;
use vaultrs::{
    auth::oidc,
    client::{Client as _, VaultClient as Client},
};
use vaultrs::api::AuthInfo;

use crate::{
    configuration::StrongholdConfiguration,
    errors::{IdentityError, IdentityResult as Result},
    models::{TokenType, TokenWrap},
    utils::new_stronghold_key,
};

pub const VAULT_DOC_ID: &str = "streams_doc_id";
pub const VAULT_STREAMS_ADDRESSES: &str = "streams_addresses";

pub struct VaultClient {
    config: StrongholdConfiguration,
    token: TokenWrap,
    vault_client: Client,
    exp: u64,
    password: Option<String>,
}

impl Debug for VaultClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultClient")
            .field("config", &self.config)
            .field("exp", &self.exp)
            .field("password", &self.password)
            .finish()
    }
}

impl VaultClient {
    pub async fn new(config: StrongholdConfiguration, token: TokenWrap) -> Result<VaultClient> {
        let vault_config = vaultrs::client::VaultClientSettingsBuilder::default()
            .address("http://35.230.109.16:8200")
            .build()
            .expect("Should be able to build the vault configuration");

        let mut vault_client = Client::new(vault_config).expect("Should be able to use vault client");

        let auth_info = Self::set_client_token(&mut vault_client, token.clone()).await?;

        let exp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + auth_info.lease_duration;

        Ok(VaultClient {
            config,
            token,
            vault_client,
            exp,
            password: None,
        })
    }

    pub async fn config(&self) -> &StrongholdConfiguration {
        &self.config
    }

    async fn check_token(&mut self) -> Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let (mount, role) = match self.token.token_type() {
            TokenType::VAULT => ("jwt", Some("default".to_string())),
            _ => ("jwt2", None),
        };
        if self.exp <= now {
            let auth_info = oidc::login(&self.vault_client, mount, self.token.raw(), role)
                .await
                .expect("Should be able to login");

            self.vault_client.set_token(&auth_info.client_token);
            self.exp = self.token.get_expiration().unwrap_or(now);
        }
        Ok(())
    }

    pub async fn store_password(&mut self, password: String) -> Result<()> {
        self.check_token().await?;
        let key = format!("users/{}/stronghold", self.token.get_sub().unwrap());

        let data = serde_json::json!({
            "data": {
                "password": password
            }
        });

        debug!("Storing password in {}", key);
        vaultrs::kv2::set(&self.vault_client, "stronghold", &key, &data)
            .await
            .expect("Should be able to store the secret");
        Ok(())
    }

    pub async fn retrieve_password(&mut self) -> Result<String> {
        self.check_token().await?;
        let key = format!("users/{}/stronghold", self.token.get_sub().unwrap());
        match &self.password {
            None => {
                debug!("No password in vault client");
                if let Ok(secret) = vaultrs::kv2::read::<Value>(&self.vault_client, "stronghold", &key).await {
                    if let Some(data) = secret.get("data") {
                        if let Some(p) = data.get("password") {
                            if let Some(password) = p.as_str() {
                                self.password = Some(password.to_string());
                                return Ok(password.to_string());
                            }
                        }
                    }
                }

                info!("No stronghold key found, generating new one");
                let password = new_stronghold_key();
                self.password = Some(password.clone());
                self.store_password(password).await?;

                Err(IdentityError::NoStrongholdSecret)?
            }
            Some(password) => Ok(password.clone()),
        }
    }

    pub async fn update_client_token(&mut self, token: TokenWrap) -> Result<()> {
        Self::set_client_token(&mut self.vault_client, token).await?;
        Ok(())
    }

    async fn set_client_token(vault_client: &mut vaultrs::client::VaultClient, token: TokenWrap) -> Result<AuthInfo> {
        let (mount, role) = match token.token_type() {
            TokenType::VAULT => ("jwt", Some("default".to_string())),
            _ => ("jwt2", None),
        };
        let auth_info = oidc::login(vault_client, mount, token.raw(), role)
            .await
            .expect("Should be able to login");
        vault_client.set_token(&auth_info.client_token);
        Ok(auth_info)
    }
}
