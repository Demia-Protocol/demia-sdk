mod analytics;
mod asset;
mod data;
mod hedera;
mod identity;
mod json_scheme_wrap;
mod notification;
mod parameter;
mod reading;
mod record;
mod sensor;
mod site;
mod token;
mod user_metadata;
mod valueset;
mod vault;

use std::collections::HashSet;

pub use analytics::*;
pub use asset::*;
pub use data::*;
pub use hedera::*;
pub use identity::*;
pub use json_scheme_wrap::*;
pub use notification::*;
pub use parameter::*;
pub use reading::*;
pub use record::*;
use rocket_okapi::okapi::schemars;
pub use sensor::*;
pub use site::*;
pub use token::*;
pub use user_metadata::*;
pub use valueset::*;
pub use vault::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StreamsAddresses(pub HashSet<String>);

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct Card {
    pub title: String,
    pub content: String,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ProjectInfo {
    #[serde(rename = "type")]
    pub project_type: String,
    #[serde(rename = "projectId")]
    pub id: String,
    pub methodology: String,
    #[serde(rename = "projectDev")]
    pub developer: String,
}

impl ProjectInfo {
    pub fn new(project_id: String, project_type: String, methodology: String, developer: String) -> Self {
        ProjectInfo {
            project_type,
            id: project_id,
            methodology,
            developer,
        }
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct GHGInfo {
    pub value: String,
    pub data: Vec<String>,
    pub unit: String,
    pub label: String,
}

impl GHGInfo {
    pub fn new(value: &str, unit: &str, label: &str) -> Self {
        GHGInfo {
            value: value.to_string(),
            unit: unit.to_string(),
            data: vec![value.to_string()],
            label: label.to_string(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, schemars::JsonSchema, Debug)]
pub struct LoginCredentials {
    /// Username for the user
    username: String,
    /// Password for the user instance
    password: String,
}

#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema, Debug)]
pub struct LoginResponse {
    pub message: String,
    pub token: String,
}
