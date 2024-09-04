use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    clients::SecretManager,
    errors::{SecretError, SecretResult},
    models::{TokenType, TokenWrap},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenManager {
    #[serde(skip_serializing, skip_deserializing)]
    tokens: Arc<RwLock<HashMap<TokenType, TokenWrap>>>,

    #[serde(skip_serializing, skip_deserializing, default = "crate::clients::default_secret")]
    secret_manager: Box<dyn SecretManager>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenSecret<'a> {
    pub password: Option<&'a str>,
    pub secret: Option<&'a str>,
}

impl Default for TokenManager {
    fn default() -> Self {
        Self {
            tokens: Default::default(),
            secret_manager: crate::clients::default_secret(),
        }
    }
}

#[async_trait::async_trait]
impl SecretManager for TokenManager {
    async fn get_token(&mut self, token_type: &TokenType, username: &str, password: &str) -> SecretResult<TokenWrap> {
        {
            let lock = self.tokens.read().await;
            if let Some(token) = lock.get(token_type) {
                if !token.is_expired() {
                    return Ok(token.clone());
                }
            }
        }

        let token = self.secret_manager.get_token(token_type, username, password).await?;
        self.tokens.write().await.insert(token_type.clone(), token.clone());

        Ok(token)
    }

    async fn get_token_with_secret(&mut self, token_type: &TokenType, client_secret: &str) -> SecretResult<TokenWrap> {
        {
            let lock = self.tokens.read().await;
            if let Some(token) = lock.get(token_type) {
                if !token.is_expired() {
                    return Ok(token.clone());
                }
            }
        }

        let token = self
            .secret_manager
            .get_token_with_secret(token_type, client_secret)
            .await?;
        self.tokens.write().await.insert(token_type.clone(), token.clone());

        Ok(token)
    }

    /// Refreshes the "refresh" token. Doesn't update tokens held by the tokenmanager.
    /// That operation is called refresh_token_type() or refresh()
    async fn refresh_token(&mut self) -> SecretResult<TokenWrap> {
        self.secret_manager.refresh_token().await
    }

    /// Creates a TokenWrap for a raw id token string and stores the token locally. This is for API
    /// based functionality and won't contain the refresh token
    async fn token_from_raw(&mut self, token_type: &TokenType, token: &str) -> SecretResult<TokenWrap> {
        self.secret_manager.token_from_raw(token_type, token).await
    }
}

impl TokenManager {
    pub fn new(secret_manager: Box<impl SecretManager + 'static>) -> Self {
        Self {
            tokens: Default::default(),
            secret_manager,
        }
    }

    /// Refreshes the refresh token and all active tokens held by the token manager.
    pub async fn refresh(&mut self) -> SecretResult<()> {
        let new_token = self.refresh_token().await?;
        let mut new_tokens: HashMap<TokenType, TokenWrap> = HashMap::default();
        for el in self.tokens.clone().read().await.keys() {
            new_tokens.insert(el.clone(), new_token.clone());
        }
        self.tokens.write().await.extend(new_tokens);
        Ok(())
    }

    /// Refreshes the specific token regardless wether its expired or not
    async fn _refresh_token_type(&mut self, token_type: TokenType, username: &str, password: &str) -> SecretResult<()> {
        let token = self.secret_manager.get_token(&token_type, username, password).await?;
        self.tokens.write().await.insert(token_type.clone(), token.clone());
        Ok(())
    }

    // Checks if the token exists and is unexpired(true), otherwise false
    pub async fn get_status(&self, token_type: TokenType) -> bool {
        let lock = self.tokens.read().await;
        lock.get(&token_type).map(|t| !t.is_expired()).unwrap_or(false)
    }

    pub async fn set_vault_token(&mut self, vault_token: TokenWrap) {
        self.tokens.write().await.insert(TokenType::VAULT, vault_token);
    }

    pub async fn vault_token(&self) -> SecretResult<TokenWrap> {
        match self.tokens.read().await.get(&TokenType::VAULT) {
            Some(token) => Ok(token.clone()),
            None => Err(SecretError::TokenNotFound(TokenType::VAULT.to_string())),
        }
    }

    pub async fn set_aws_token(&mut self, vault_token: TokenWrap) {
        self.tokens.write().await.insert(TokenType::AWS, vault_token);
    }

    pub async fn aws_token(&self) -> SecretResult<TokenWrap> {
        match self.tokens.read().await.get(&TokenType::AWS) {
            Some(token) => Ok(token.clone()),
            None => Err(SecretError::TokenNotFound(TokenType::AWS.to_string())),
        }
    }

    pub async fn set_auth0_token(&mut self, token: TokenWrap) {
        self.tokens.write().await.insert(TokenType::AUTH0, token);
    }

    pub async fn auth0_token(&self) -> SecretResult<TokenWrap> {
        match self.tokens.read().await.get(&TokenType::AUTH0) {
            Some(token) => Ok(token.clone()),
            None => Err(SecretError::TokenNotFound(TokenType::AUTH0.to_string())),
        }
    }
}
