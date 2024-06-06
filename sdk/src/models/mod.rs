mod reading;
mod sensor;
mod site;
mod token;
mod vault;

use std::collections::HashSet;

pub use reading::*;
pub use sensor::*;
pub use site::*;
pub use token::*;
pub use vault::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StreamsAddresses(pub HashSet<String>);

#[derive(serde::Serialize)]
pub struct Card {
    pub title: String,
    pub content: String,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValueSet {
    pub title: String,
    pub label: String,
    pub values: Vec<f64>,
    pub timestamps: Vec<String>,
    pub total: f64,
    pub avg: f64,
}

impl ValueSet {
    pub fn new(mut values: Vec<f64>, timestamps: Vec<String>, title: String, label: String) -> ValueSet {
        if values.len() == 1 {
            (1..timestamps.len()).for_each(|_| values.push(values[0]))
        }
        let total = values.iter().sum();
        let avg = total / values.len() as f64;
        ValueSet {
            values,
            timestamps,
            total,
            avg,
            title,
            label,
        }
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification {}
