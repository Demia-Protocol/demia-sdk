use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::TokenData;
use rocket_okapi::okapi::schemars;
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub id_token: String,
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
        Some(sub.to_string().replace('"', "").replace("auth0|", ""))
    }

    pub fn get_email(&self) -> Option<String> {
        let email = self.token.claims.get("email").expect("Should be able to pull email");
        Some(email.to_string().replace('"', ""))
    }

    pub fn get_name(&self) -> Option<String> {
        let name = self
            .token
            .claims
            .get("nickname")
            .expect("Should be able to pull nickname");
        Some(name.to_string().replace('"', ""))
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn is_expired(&self) -> bool {
        let time_elapsed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.get_expiration().unwrap() <= time_elapsed
    }

    pub fn get_expiration(&self) -> Option<u64> {
        self.token
            .claims
            .get("exp")
            .expect("Should be able to pull sub")
            .as_u64()
    }

    pub fn token_data(&self) -> &TokenData<Value> {
        &self.token
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Eq, Hash, PartialEq)]
pub enum TokenType {
    AWS,
    AUTH0,
    Auth0Admin,
    VAULT,
}

impl TokenType {
    pub fn client_id(&self) -> &'static str {
        match self {
            Self::AWS => "aws-token-issuer",
            Self::AUTH0 => "KJO1MMQW7ae5aQykrpbNKZnyUJb7dsyZ",
            Self::Auth0Admin => "TOF8oMvj577kvq2tVq6dofRDDEAfdAwn",
            Self::VAULT => "vault-client-public",
        }
    }

    pub fn name(&self) -> &'static str {
        match &self {
            Self::AWS => "aws",
            Self::AUTH0 => "auth0",
            Self::Auth0Admin => "auth0-admin",
            Self::VAULT => "vault",
        }
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
