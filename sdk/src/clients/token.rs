use crate::{
    clients::SecretManager,
    errors::{SecretError, SecretResult},
    models::{TokenType, TokenWrap},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenManager {
    #[serde(skip_serializing, skip_deserializing)]
    vault_token: Option<TokenWrap>,
    #[serde(skip_serializing, skip_deserializing)]
    aws_token: Option<TokenWrap>,

    #[serde(skip_serializing, skip_deserializing, default = "crate::clients::default_secret")]
    secret_manager: Box<dyn SecretManager>,
}

impl Default for TokenManager {
    fn default() -> Self {
        Self {
            vault_token: Default::default(),
            aws_token: Default::default(),
            secret_manager: crate::clients::default_secret(),
        }
    }
}

#[async_trait::async_trait]
impl SecretManager for TokenManager {
    async fn get_token(&mut self, token_type: &TokenType, username: &str, password: &str) -> SecretResult<TokenWrap> {
        match token_type {
            TokenType::AWS => {
                if let Some(token) = &self.aws_token {
                    if !token.is_expired() {
                        return Ok(token.clone());
                    }
                }
            }
            TokenType::VAULT => {
                if let Some(token) = &self.vault_token {
                    if !token.is_expired() {
                        return Ok(token.clone());
                    }
                }
            }
        };

        let token = self.secret_manager.get_token(token_type, username, password).await?;

        match token_type {
            TokenType::AWS => self.set_aws_token(token.clone()),
            TokenType::VAULT => self.set_vault_token(token.clone()),
        }

        Ok(token)
    }

    async fn refresh_token(&mut self) -> SecretResult<TokenWrap> {
        let new_token = self.secret_manager.refresh_token().await?;
        self.set_vault_token(new_token.clone());

        Ok(new_token)
    }
}

impl TokenManager {
    pub fn new(secret_manager: Box<impl SecretManager + 'static>) -> Self {
        Self {
            secret_manager,
            vault_token: None,
            aws_token: None,
        }
    }

    pub fn get_status(&self, token_type: TokenType) -> bool {
        match token_type {
            TokenType::AWS => self.aws_token.is_some() && !self.aws_token.as_ref().unwrap().is_expired(),
            TokenType::VAULT => self.vault_token.is_some() && !self.vault_token.as_ref().unwrap().is_expired(),
        }
    }

    pub async fn refresh(&mut self) -> SecretResult<TokenWrap> {
        let _token = self.vault_token()?;
        self.refresh_token().await
    }

    pub fn set_vault_token(&mut self, vault_token: TokenWrap) {
        self.vault_token = Some(vault_token);
    }

    pub fn vault_token(&self) -> SecretResult<&TokenWrap> {
        match self.vault_token.as_ref() {
            Some(token) => Ok(token),
            None => Err(SecretError::TokenNotFound(TokenType::VAULT.to_string()).into()),
        }
    }

    pub fn set_aws_token(&mut self, aws_token: TokenWrap) {
        self.aws_token = Some(aws_token);
    }

    pub fn aws_token(&self) -> SecretResult<&TokenWrap> {
        match self.aws_token.as_ref() {
            Some(token) => Ok(token),
            None => Err(SecretError::TokenNotFound(TokenType::AWS.to_string()).into()),
        }
    }
}
