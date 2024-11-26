use rocket_okapi::okapi::schemars;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    errors::{ApiError as Error, ApiResult as Result},
    models::{GuardianAccessTokenResponse, GuardianLoginResponse, GuardianProfileResponse, GuardianReport},
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GuardianClient {
    pub client: GuardianApiClient,
    pub access_token: Option<String>,
    pub policy_id: String,
    pub send_block: String,
    pub trust_chain_block: String,
    pub ref_block: String,
}

impl GuardianClient {
    pub fn new(client: GuardianApiClient, policy_id: String, send_block: String, ref_block: String) -> Self {
        GuardianClient {
            client,
            policy_id,
            send_block,
            ref_block,
            ..Default::default()
        }
    }

    //
    pub async fn connect(&mut self) -> Result<()> {
        self.log_in("Installer", "test").await?;
        self.get_policy().await?;
        Ok(())
    }

    pub fn access_token(&self) -> Result<&str> {
        self.access_token
            .as_deref()
            .ok_or_else(|| Error::Guardian("No Access token".to_string()))
    }

    pub async fn send_report(&mut self, report: GuardianReport, username: &str, password: &str) -> Result<()> {
        self.log_in(username, password).await?;
        self.get_policy().await?;

        let ref_blocks = self.get_ref_block().await?;
        let ref_blocks = ref_blocks
            .get("data")
            .ok_or_else(|| Error::Guardian("Json get data".to_string()))?
            .as_array()
            .ok_or_else(|| Error::Guardian("Json get data as array".to_string()))?;
        let ref_block = ref_blocks
            .iter()
            .find(|data| {
                data.get("type")
                    .and_then(|v| v.as_str())
                    .map(|str| str == "project_sr")
                    .unwrap_or(false)
            })
            .ok_or_else(|| Error::Guardian("Json get type".to_string()))?;

        let report_block = self.get_report_block_id().await?;
        let data = json!({
            "document": report,
            "ref": ref_block,
        });
        log::debug!("data: {:#}", data);
        let res = self
            .client
            .post_block(self.access_token()?, &self.policy_id, &report_block, &data)
            .await?;
        log::debug!("Response: {:#?}", res);
        Ok(())
    }

    async fn log_in(&mut self, username: &str, password: &str) -> Result<()> {
        let res = self.client.login(username, password).await?;
        let access_token = self.client.access_token(&res.refresh_token).await?;
        log::debug!("Login response: {:?}", access_token);

        self.access_token = Some(access_token.access_token);
        log::debug!("Access token: {:?}", self.access_token);
        Ok(())
    }

    pub async fn get_policy(&self) -> Result<Value> {
        self.client.policy(self.access_token()?, &self.policy_id).await
    }

    pub async fn get_report_block_id(&self) -> Result<String> {
        let res = self
            .client
            .ref_block(self.access_token()?, &self.policy_id, &self.send_block)
            .await?;
        Ok(get_str(res, "id")?)
    }

    pub async fn get_ref_block(&self) -> Result<Value> {
        let block_id = self.get_report_block_id().await?;
        self.get_block(&block_id).await
    }

    pub async fn get_block(&self, block_id: &str) -> Result<Value> {
        self.client
            .get_block(self.access_token()?, &self.policy_id, block_id)
            .await
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GuardianApiClient {
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) client: reqwest::Client,
    pub(crate) url: String,
}

impl GuardianApiClient {
    pub fn new(url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
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

    pub async fn profile(&self, username: &str, access_token: &str) -> Result<GuardianProfileResponse> {
        let profile_url = format!("{}/profiles/{}", self.url, username);

        Ok(self
            .client
            .get(profile_url)
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<GuardianProfileResponse>()
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

    pub async fn policy(&self, access_token: &str, policy_id: &str) -> Result<Value> {
        let policy_url = format!("{}/policies/{}", self.url, policy_id);

        Ok(self
            .client
            .get(&policy_url)
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<Value>()
            .await?)
    }
    pub async fn ref_block(&self, access_token: &str, policy_id: &str, ref_block: &str) -> Result<Value> {
        let tag_url = format!("{}/policies/{}/tag/{}", self.url, policy_id, ref_block);

        Ok(self
            .client
            .get(&tag_url)
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<Value>()
            .await?)
    }

    pub async fn blocks(&self, access_token: &str, policy_id: &str) -> Result<Value> {
        let policy_url = format!("{}/policies/{}/blocks", self.url, policy_id);

        Ok(self
            .client
            .get(&policy_url)
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<Value>()
            .await?)
    }

    pub async fn get_block(&self, access_token: &str, policy_id: &str, block_id: &str) -> Result<Value> {
        let get_block_url = format!("{}/policies/{}/blocks/{}", self.url, policy_id, block_id);

        Ok(self
            .client
            .get(&get_block_url)
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<Value>()
            .await?)
    }

    pub async fn post_block(&self, access_token: &str, policy_id: &str, block_id: &str, data: &Value) -> Result<Value> {
        let post_block_url = format!("{}/policies/{}/blocks/{}", self.url, policy_id, block_id);

        Ok(self
            .client
            .post(&post_block_url)
            .bearer_auth(access_token)
            .json(&data)
            .send()
            .await?
            .json::<Value>()
            .await?)
    }
}

// Extracts string from serde Value, thros Guardian error if it fails
pub(crate) fn get_str(value: Value, name: &str) -> Result<String> {
    let val = value
        .get(name)
        .ok_or_else(|| Error::Guardian(format!("Missing {}", name).to_string()))?;
    val.as_str()
        .ok_or_else(|| Error::Guardian(format!("{} not a string", name).to_string()))
        .map(|s| s.to_owned())
}
