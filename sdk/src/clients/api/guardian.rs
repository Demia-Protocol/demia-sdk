use crate::errors::ApiResult as Result;
use crate::models::{GuardianAccessTokenResponse, GuardianLoginResponse};
use reqwest::Client;
use serde_json::{json, Value};

pub struct GuardianApiClient {
    pub(crate) client: Client,
    pub(crate) url: String,
}

impl GuardianApiClient {
    pub fn new(url: String) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<GuardianLoginResponse> {
        let login_url = format!("{}{}", self.url, "/accounts/login");
        Ok(self
            .client
            .post(&login_url)
            .json(&json!({"username": username, "password": password}))
            .send()
            .await?
            .json::<GuardianLoginResponse>()
            .await?)
    }

    pub async fn access_token(&self, refresh_token: &str) -> Result<GuardianAccessTokenResponse> {
        let token_url = format!("{}{}", self.url, "/accounts/access-token");

        Ok(self
            .client
            .post(token_url)
            .json(&json!({"refreshToken": refresh_token}))
            .send()
            .await?
            .json::<GuardianAccessTokenResponse>()
            .await?)
    }

    pub async fn profile(&self, username: &str, access_token: &str) -> Result<Value> {
        let profile_url = format!("{}/profiles/{}", self.url, username);

        Ok(self
            .client
            .get(profile_url)
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<Value>()
            .await?)
    }

    pub async fn policies(&self, access_token: &str) -> Result<Value> {
        let policies_url = format!("{}{}", self.url, "/policies");

        Ok(self
            .client
            .get(&policies_url)
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<Value>()
            .await?)
    }
}
