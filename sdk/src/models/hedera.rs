use chrono::{DateTime, Utc};
use log::info;
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::errors::{Error, SdkResult as Result};

pub const GUARDIAN_BASE: &str = "http://guardian.demia-nodes.net/api/v1";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GuardianClient {
    #[serde(skip_serializing, skip_deserializing)]
    pub client: reqwest::Client,
    pub access_token: Option<String>,
    pub policy_id: String,
    pub send_block: String,
    pub trust_chain_block: String,
    pub ref_block: String,
}

impl GuardianClient {
    pub fn new(policy_id: String, send_block: String, ref_block: String) -> Self {
        GuardianClient {
            policy_id,
            send_block,
            ref_block,
            ..Default::default()
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        self.log_in("Installer", "test").await?;
        info!("Logged in");
        self.get_policy().await?;
        info!("Got policy");
        Ok(())
    }

    pub async fn send_report(&mut self, report: GuardianReport, username: &str, password: &str) -> Result<()> {
        let url = format!("{}/policies/{}/blocks", GUARDIAN_BASE, self.policy_id);
        info!("Connecting to guardian url: {}", url);
        self.log_in(username, password).await?;
        self.get_policy().await?;
        let ref_blocks = self.get_ref_block().await?;
        let ref_blocks = ref_blocks
            .get("data")
            .ok_or_else(|| Error::Hedera("Json get data".to_string()))?
            .as_array()
            .ok_or_else(|| Error::Hedera("Json get data as array".to_string()))?;
        let ref_block = ref_blocks
            .iter()
            .find(|data| {
                data.get("type")
                    .and_then(|v| v.as_str())
                    .map(|str| str == "project_sr")
                    .unwrap_or(false)
            })
            .ok_or_else(|| Error::Hedera("Json get type".to_string()))?;

        let report_block = self.get_report_block().await?;
        let data = json!({
            "document": report,
            "ref": ref_block,
        });
        info!("data: {:#}", data);
        let res = self
            .client
            .post(format!("{}/{}", url, report_block))
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.clone().unwrap()),
            )
            .json(&data)
            .send()
            .await?;
        info!("Response: {:#?}", res.json::<Value>().await?);
        Ok(())
    }

    async fn log_in(&mut self, username: &str, password: &str) -> Result<()> {
        let url = format!("{}/accounts/login", GUARDIAN_BASE);
        let body = json!( {
            "username": username,
            "password": password
        });
        let res = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        let refresh_token = get_str(res, "refreshToken")?;

        let url = format!("{}/accounts/access-token", GUARDIAN_BASE);
        let access_token = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json!({"refreshToken": refresh_token}))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        info!("Login response: {}", access_token);
        self.access_token = Some(get_str(access_token, "accessToken")?);
        info!("Access token: {:?}", self.access_token);
        Ok(())
    }

    pub async fn get_policy(&self) -> Result<Value> {
        let url = format!("{}/policies/{}", GUARDIAN_BASE, self.policy_id);
        info!("Getting policy: {}", url);
        self.bearer(&url).await
    }

    pub async fn get_report_block(&self) -> Result<String> {
        let url = format!("{}/policies/{}/tag/{}", GUARDIAN_BASE, self.policy_id, self.send_block);
        let res = self
            .client
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.access_token.clone().unwrap()),
            )
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        Ok(res.get("id").unwrap().as_str().unwrap().to_string())
    }

    pub async fn get_ref_block(&self) -> Result<Value> {
        let url = format!("{}/policies/{}/tag/{}", GUARDIAN_BASE, self.policy_id, self.ref_block);
        info!("Requesting ref block {}", url);
        let res = self
            .client
            .get(&url)
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    self.access_token
                        .as_ref()
                        .ok_or_else(|| Error::Hedera("Missing access token".to_string()))?
                ),
            )
            .send()
            .await?
            .json::<Value>()
            .await?;
        info!("Ref block id: {:#?}", res);
        let block_id = get_str(res, "id")?;
        self.get_block(&block_id).await
    }

    pub async fn get_block(&self, id: &str) -> Result<Value> {
        let url = format!("{}/policies/{}/blocks/{}", GUARDIAN_BASE, self.policy_id, id);
        self.bearer(&url).await
    }

    async fn bearer(&self, url: &str) -> Result<Value> {
        Ok(self
            .client
            .get(url)
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    self.access_token
                        .as_ref()
                        .ok_or_else(|| Error::Hedera("Missing access token".to_string()))?
                ),
            )
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?)
    }
}

// Extracts string from serde Value, thros Hedera error if it fails
pub(crate) fn get_str(value: Value, name: &str) -> Result<String> {
    let val = value
        .get(name)
        .ok_or_else(|| Error::Hedera(format!("Missing {}", name).to_string()))?;
    val.as_str()
        .ok_or_else(|| Error::Hedera(format!("{} not a string", name).to_string()))
        .map(|s| s.to_owned())
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, JsonSchema)]
pub struct GuardianReport {
    #[serde(rename = "field0")]
    pub report_id: String,
    #[serde(rename = "field1")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "field2")]
    pub confidence_score: f32,
    #[serde(rename = "field3")]
    pub calculation_label: String,
    #[serde(rename = "field4")]
    pub value: f64,
    #[serde(rename = "field5")]
    pub message_ids: Vec<String>,
    #[serde(rename = "field6")]
    pub stream_address: String,
    #[serde(rename = "field7")]
    pub start_time: DateTime<Utc>,
    #[serde(rename = "field8")]
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct HederaLoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct GuardianRegistryForm {
    #[serde(rename = "operatorId", default = "String::new")]
    pub operator_id: String,
    #[serde(rename = "operatorKey", default = "String::new")]
    pub operator_key: String,
    #[serde(default = "String::new")]
    pub policy: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuardianLoginResponse {
    pub username: String,
    #[serde(default)]
    pub did: String,
    pub role: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuardianAccessTokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GuardianProfileResponse {
    pub username: String,
    pub role: String,
    pub did: String,
    pub parent: String,
    #[serde(rename = "hederaAccountId")]
    pub hedera_account_id: String,
    pub confirmed: bool,
    pub failed: bool,
    #[serde(rename = "hederaAccountKey")]
    pub hedera_account_key: Option<String>,
    #[serde(rename = "topicId")]
    pub topic_id: String,
    #[serde(rename = "parentTopicId")]
    pub parent_topic_id: String,
    #[serde(rename = "didDocument")]
    pub did_document: GuardianDidDocument,
    #[serde(rename = "vcDocument")]
    pub vc_document: Option<serde_json::Value>, // VcDocument
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GuardianDidDocument {
    #[serde(rename = "createDate")]
    pub create_date: DateTime<Utc>,
    pub did: String,
    pub document: Map<String, Value>, // CoreDocument "https://www.w3.org/ns/did/v1"
    pub id: String,
    #[serde(rename = "messageId")]
    pub message_id: String,
    pub status: String,
    #[serde(rename = "topicId")]
    pub topic_id: String,
    #[serde(rename = "updateDate")]
    pub update_date: DateTime<Utc>,
    #[serde(rename = "verificationMethods")]
    pub verification_methods: serde_json::Value, // map of [methd : did]
    pub _id: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct VerifiableCredential {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    pub r#type: Vec<String>,
    #[serde(rename = "credentialSubject")]
    pub credential_subject: Vec<CredentialSubject>,
    pub issuer: String,
    #[serde(rename = "issuanceDate")]
    pub issuance_date: String,
    pub proof: Proof,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CredentialSubject {
    pub id: String,
    #[serde(rename = "field0")]
    pub report_id: String,
    #[serde(rename = "field1")]
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "field2")]
    pub confidence_score: f32,
    #[serde(rename = "field3")]
    pub calculation_label: String,
    #[serde(rename = "field4")]
    pub value: f64,
    #[serde(rename = "field5")]
    pub message_ids: Vec<String>,
    #[serde(rename = "field6")]
    pub stream_address: String,
    #[serde(rename = "field7")]
    pub start_time: DateTime<Utc>,
    #[serde(rename = "field8")]
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Proof {
    r#type: String,
    #[serde(rename = "verificationMethod")]
    verification_method: String,
    jws: String,
    created: String,
    #[serde(rename = "proofPurpose")]
    proof_purpose: String,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DataResponse {
    pub address: String,
    pub content: Value,
    pub identifier: String,
}

#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ReportParams {
    pub site_id: String,
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}
