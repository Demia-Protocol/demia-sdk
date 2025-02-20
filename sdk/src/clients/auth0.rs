use std::fmt::Debug;

use base64::Engine;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde_json::{Value, json};

use crate::{
    clients::SecretManager,
    configuration::ApplicationConfiguration,
    errors::{SecretError, SecretResult},
    models::{TokenResponse, TokenType, TokenWrap},
};

#[derive(Default, Clone)]
pub struct Auth0Client {
    client: reqwest::Client,
    url: String,
    session_refresh: Option<String>,
}

impl Auth0Client {
    pub fn new(config: &ApplicationConfiguration) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: config.secrets_api.clone(),
            session_refresh: None,
        }
    }

    async fn get_token_data(&self, token: &TokenResponse) -> SecretResult<jsonwebtoken::TokenData<Value>> {
        let jwks_url = format!("{}/.well-known/jwks.json", self.url);
        let jwks_json: Value = reqwest::get(jwks_url)
            .await
            .map_err(|_| SecretError::Jwt("couldn't query jwks".to_string()))?
            .json()
            .await
            .map_err(|_| SecretError::Jwt("couldn't convert jwk response to json".to_string()))?;
        let jwk = jwks_json["keys"][0]["x5c"][0]
            .as_str()
            .ok_or_else(|| SecretError::Jwt("Failed to extract jwk public key".to_string()))?;

        // println!("Jwk: {}", jwk);
        // The public key is Base64-encoded in the JWKS, so decode it
        let engine = base64::engine::general_purpose::STANDARD_NO_PAD;
        let public_key_der = engine
            .decode(jwk)
            .map_err(|_| SecretError::Jwt("coudln't decode jwk base64".to_string()))?;

        let public_key_pem = format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
            engine.encode(public_key_der)
        );

        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(|_| SecretError::Jwt("Failed to turn key into decodingkey".to_string()))?;
        let mut validator = Validation::new(Algorithm::RS256);
        validator.set_audience(&["KJO1MMQW7ae5aQykrpbNKZnyUJb7dsyZ"]);

        jsonwebtoken::decode::<Value>(&token.id_token, &decoding_key, &validator)
            .map_err(|e| SecretError::Jwt(e.to_string()))
    }

    async fn token_from_response(
        &mut self,
        token_type: TokenType,
        response: reqwest::Response,
    ) -> SecretResult<TokenWrap> {
        let token: TokenResponse = response
            .json()
            .await
            .map_err(|_| SecretError::Jwt("Should be a token response".to_string()))?;
        self.session_refresh.replace(token.refresh_token.clone());
        let token_data = self.get_token_data(&token).await?;

        Ok(TokenWrap::new(token_type, token_data, token.id_token.clone()))
    }
}

impl Debug for Auth0Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Auth0").finish()
    }
}

#[async_trait::async_trait]
impl SecretManager for Auth0Client {
    async fn get_token(&mut self, token_type: &TokenType, username: &str, password: &str) -> SecretResult<TokenWrap> {
        let client_id = token_type.client_id();
        log::debug!("Refreshing token: {}", client_id);

        let url = format!("{}/oauth/token", self.url);
        let params = json!({
            "grant_type": "password",
            "client_id": client_id,
            "username": username,
            "password": password,
            "scope": "openid profile email offline_access"
        });

        let response = self
            .client
            .post(url.clone())
            .form(&params)
            .send()
            .await
            .map_err(|_| SecretError::Jwt("Failed to receive reponse from Auth0 client".to_string()))?;

        log::debug!("Response: {:?}", response);

        self.token_from_response(token_type.clone(), response).await
    }

    async fn get_token_with_secret(&mut self, token_type: &TokenType, client_secret: &str) -> SecretResult<TokenWrap> {
        let client_id = token_type.client_id();
        log::debug!("Refreshing token: {}", client_id);

        let url = format!("{}/oauth/token", self.url);
        let params = json!({
            "grant_type": "client_secret",
            "client_id": client_id,
            "client_secret": client_secret,
            "scope": "openid profile email offline_access"
        });

        let response = self
            .client
            .post(url.clone())
            .form(&params)
            .send()
            .await
            .map_err(|_| SecretError::Jwt("Failed to receive reponse from Auth0 client".to_string()))?;

        log::debug!("Response: {:?}", response);
        self.token_from_response(token_type.clone(), response).await
    }

    async fn refresh_token(&mut self) -> SecretResult<TokenWrap> {
        let url = format!("{}/oauth/token", self.url);
        let params = serde_json::json!({
            "grant_type": "refresh_token",
            "refresh_token": &self.session_refresh,
            "client_id": "KJO1MMQW7ae5aQykrpbNKZnyUJb7dsyZ",
        });

        let response = self.client.post(url).form(&params).send().await?;
        let token: TokenResponse = response.json().await?;
        self.session_refresh.replace(token.refresh_token.clone());
        let token_data = self.get_token_data(&token).await?;

        Ok(TokenWrap::new(TokenType::AUTH0, token_data, token.id_token.clone()))
    }

    async fn token_from_raw(&self, token_type: &TokenType, token: &str) -> SecretResult<TokenWrap> {
        let client_id = token_type.client_id();
        log::debug!("Refreshing token: {}", client_id);

        let token_data = self
            .get_token_data(&TokenResponse {
                access_token: "".to_string(),
                id_token: token.to_string(),
                refresh_token: "".to_string(),
            })
            .await?;

        Ok(TokenWrap::new(token_type.clone(), token_data, token.to_string()))
    }
}
