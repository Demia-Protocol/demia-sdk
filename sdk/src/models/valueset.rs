use std::collections::HashMap;
use chrono::{DateTime, Utc};

use super::Parameter;
use crate::utils::deserialize_null_default;
use serde::Deserialize;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ValueSetsWrap {
    site_id: String,
    value_sets: Vec<ValueSet>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ValueSet {
    #[serde(default)]
    pub inputs: HashMap<String, Vec<(DateTime<Utc>, f64)>>,
    #[serde(default)]
    pub params: Vec<Parameter>,
    pub title: String,
    pub label: String,
    #[serde(default, deserialize_with = "deserialize_map_to_vec")]
    pub data: Vec<(DateTime<Utc>, f64)>,
    pub total: f64,
    // Since we occasionally divide by 0.0 this can become NAN so default to 0, or it will serialize
    // as NAN/null and break deserialization https://github.com/serde-rs/json/issues/202
    #[serde(deserialize_with = "deserialize_null_default")]
    pub avg: f64,
}

impl ValueSet {
    pub fn new(
        inputs: HashMap<String, Vec<(DateTime<Utc>, f64)>>,
        data: Vec<(DateTime<Utc>, f64)>,
        title: String,
        label: String,
        params: Vec<Parameter>,
    ) -> ValueSet {
        let total = data.iter().map(|(_, v)| v).sum();
        let avg = total / data.len() as f64;
        ValueSet {
            inputs,
            data,
            total,
            avg,
            title,
            label,
            params,
        }
    }
}



fn deserialize_map_to_vec<'de, D>(deserializer: D) -> Result<Vec<(DateTime<Utc>, f64)>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Deserialize into a HashMap first
    let map: HashMap<DateTime<Utc>, f64> = HashMap::deserialize(deserializer)?;
    // Convert HashMap into a Vec of tuples
    Ok(map.into_iter().collect())
}