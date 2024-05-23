use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::TokenData;
use serde_json::Value;

#[derive(Debug, serde::Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone)]
pub struct TokenWrap {
    token_type: TokenType,
    token: TokenData<Value>,
    raw: String,
}

impl TokenWrap {
    pub fn new(token_type: TokenType, token: TokenData<Value>, raw: String) -> Self {
        TokenWrap { token_type, token, raw }
    }

    pub fn get_sub(&self) -> Option<String> {
        let sub = self.token.claims.get("sub").expect("Should be able to pull sub");
        Some(sub.to_string().replace('"', ""))
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn is_expired(&self) -> bool {
        let time_elapsed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.get_expiration().unwrap() <= time_elapsed
    }

    fn get_expiration(&self) -> Option<u64> {
        self.token
            .claims
            .get("exp")
            .expect("Should be able to pull sub")
            .as_u64()
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }
}

#[derive(Debug, Clone)]
pub enum TokenType {
    AWS,
    VAULT,
}

impl TokenType {
    pub fn client_id(&self) -> &'static str {
        match self {
            Self::AWS => "aws-token-issuer",
            Self::VAULT => "vault-client-public",
        }
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
