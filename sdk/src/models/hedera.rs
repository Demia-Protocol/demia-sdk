use chrono::{DateTime, Utc};
use rocket_okapi::okapi::schemars;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Deserialize, Serialize, Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct GuardianRegistryForm {
    #[serde(default = "String::new")]
    pub operator_id: String,
    #[serde(default = "String::new")]
    pub operator_key: String,
    #[serde(default = "String::new")]
    pub policy: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct GuardianLoginResponse {
    pub username: String,
    #[serde(default)]
    pub did: String,
    pub role: String,
    pub refresh_token: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct GuardianAccessTokenResponse {
    pub access_token: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct GuardianProfileResponse {
    pub username: String,
    pub role: String,
    pub did: String,
    pub parent: String,
    pub hedera_account_id: String,
    pub confirmed: bool,
    pub failed: bool,
    pub hedera_account_key: Option<String>,
    pub topic_id: String,
    pub parent_topic_id: String,
    pub did_document: GuardianDidDocument,
    pub vc_document: Option<serde_json::Value>, // VcDocument
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct GuardianDidDocument {
    pub create_date: DateTime<Utc>,
    pub did: String,
    pub document: Map<String, Value>, // CoreDocument "https://www.w3.org/ns/did/v1"
    pub id: String,
    pub message_id: String,
    pub status: String,
    pub topic_id: String,
    pub update_date: DateTime<Utc>,
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
