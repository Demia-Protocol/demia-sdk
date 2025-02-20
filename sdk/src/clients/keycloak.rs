use std::fmt::Debug;

use base64::Engine;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use reqwest::Response;
use serde_json::{Value, json};

use crate::{
    clients::SecretManager,
    configuration::ApplicationConfiguration,
    errors::SecretResult,
    models::{TokenResponse, TokenType, TokenWrap},
};

#[derive(Default)]
pub struct Keycloak {
    client: reqwest::Client,
    url: String,
    session_refresh: Option<String>,
}

impl Keycloak {
    pub fn new(config: &ApplicationConfiguration) -> Self {
        Self {
            client: reqwest::Client::new(),
            url: config.secrets_api.clone(),
            session_refresh: None,
        }
    }

    async fn get_token_data(&self, token: &TokenResponse) -> SecretResult<jsonwebtoken::TokenData<Value>> {
        let jwks_url = format!("{}/protocol/openid-connect/certs", self.url);
        let jwks_json: Value = reqwest::get(jwks_url)
            .await
            .expect("couldn't query jwks")
            .json()
            .await
            .expect("couldn't convert to json");
        let jwk = jwks_json["keys"][0]["x5c"][0]
            .as_str()
            .expect("Failed to extract public key");

        // println!("Jwk: {}", jwk);
        // The public key is Base64-encoded in the JWKS, so decode it
        let engine = base64::engine::general_purpose::STANDARD;
        let public_key_der = engine.decode(jwk).expect("coudln't decode base64");

        let public_key_pem = format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
            engine.encode(public_key_der)
        );

        // println!("Public key pem: {}", public_key_pem);

        let decoding_key =
            DecodingKey::from_rsa_pem(public_key_pem.as_bytes()).expect("Failed to turn key into decodingkey");
        let mut validator = Validation::new(Algorithm::RS256);

        validator.set_audience(&[TokenType::VAULT.client_id(), TokenType::AWS.client_id()]);
        Ok(
            jsonwebtoken::decode::<Value>(&token.access_token, &decoding_key, &validator)
                .expect("Could not decode jwt"),
        )
    }

    async fn token_from_response(&mut self, token_type: TokenType, response: Response) -> SecretResult<TokenWrap> {
        log::debug!("Response: {:?}", response);
        let token: TokenResponse = response.json().await.expect("Should be a token response");
        self.session_refresh.replace(token.refresh_token.clone());
        let token_data = self.get_token_data(&token).await?;

        Ok(TokenWrap::new(
            token_type.clone(),
            token_data,
            token.access_token.clone(),
        ))
    }
}

impl Debug for Keycloak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keycloak").finish()
    }
}

#[async_trait::async_trait]
impl SecretManager for Keycloak {
    async fn get_token(&mut self, token_type: &TokenType, username: &str, password: &str) -> SecretResult<TokenWrap> {
        let client_id = TokenType::AUTH0.client_id();
        log::debug!("Refreshing token: {}", client_id);

        let url = format!("{}/protocol/openid-connect/token", self.url);
        let params = json!({
            "grant_type": "password",
            "client_id": client_id,
            "username": username,
            "password": password
        });

        let response = self
            .client
            .post(url)
            .form(&params)
            .send()
            .await
            .expect("Expect a response at least");

        self.token_from_response(token_type.clone(), response).await
    }

    async fn get_token_with_secret(&mut self, token_type: &TokenType, client_secret: &str) -> SecretResult<TokenWrap> {
        let client_id = TokenType::AUTH0.client_id();
        log::debug!("Refreshing token: {}", client_id);

        let url = format!("{}/protocol/openid-connect/token", self.url);
        let params = json!({
            "grant_type": "client_credentials",
            "client_id": client_id,
            "client_secret": client_secret,
        });

        let response = self
            .client
            .post(url)
            .form(&params)
            .send()
            .await
            .expect("Expect a response at least");

        self.token_from_response(token_type.clone(), response).await
    }

    async fn refresh_token(&mut self) -> SecretResult<TokenWrap> {
        let url = format!("{}/protocol/openid-connect/token", self.url);
        let params = serde_json::json!({
            "grant_type": "refresh_token",
            "refresh_token": &self.session_refresh,
            "client_id": "vault-client-public",
        });

        let response = self.client.post(url).form(&params).send().await?;
        let token: TokenResponse = response.json().await?;
        self.session_refresh.replace(token.refresh_token.clone());
        let token_data = self.get_token_data(&token).await?;

        Ok(TokenWrap::new(TokenType::VAULT, token_data, token.access_token.clone()))
    }

    async fn token_from_raw(&self, token_type: &TokenType, token: &str) -> SecretResult<TokenWrap> {
        let client_id = token_type.client_id();
        log::debug!("Refreshing token: {}", client_id);

        let token_data = self
            .get_token_data(&TokenResponse {
                access_token: token.to_string(),
                id_token: "".to_string(),
                refresh_token: "".to_string(),
            })
            .await?;

        Ok(TokenWrap::new(token_type.clone(), token_data, token.to_string()))
    }
}
