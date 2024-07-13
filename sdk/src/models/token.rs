use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::TokenData;
use rocket_okapi::okapi::schemars;
use serde_json::Value;

use crate::errors::StorageResult as Result;

#[derive(Debug, serde::Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone)]
pub struct TokenWrap {
    pub token_type: TokenType,
    token: TokenData<Value>,
    refresh_token: String,
    raw: String,
}

impl TokenWrap {
    pub fn new(token_type: TokenType, token: TokenData<Value>, raw: String, refresh_token: String) -> Self {
        TokenWrap {
            token_type,
            refresh_token,
            token,
            raw,
        }
    }

    pub fn get_sub(&self) -> Result<String> {
        let sub = self.token.claims.get("sub").expect("Should be able to pull sub");
        log::info!("\nSub: {}", sub);
        Ok(sub.to_string().replace("\"", ""))
    }

    pub fn is_expired(&self) -> bool {
        let time_elapsed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.get_expiration().unwrap() <= time_elapsed
    }

    fn get_expiration(&self) -> Result<u64> {
        let exp = self
            .token
            .claims
            .get("exp")
            .expect("Should be able to pull sub")
            .as_u64()
            .expect("Should not exede u64 size for exp");
        Ok(exp)
    }

    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Eq, Hash, PartialEq)]
pub enum TokenType {
    AWS,
    AUTH0,
    VAULT,
}

impl TokenType {
    pub fn client_id(&self) -> &'static str {
        match self {
            Self::AWS => "aws-token-issuer",
            Self::AUTH0 => "KJO1MMQW7ae5aQykrpbNKZnyUJb7dsyZ",
            Self::VAULT => "vault-client-public",
        }
    }

    pub fn name(&self) -> &'static str {
        match &self {
            Self::AWS => "aws",
            Self::AUTH0 => "auth0",
            Self::VAULT => "vault",
        }
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
