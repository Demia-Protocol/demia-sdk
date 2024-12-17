use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::NestedReadingValue;

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: String,
    #[serde(alias = "sensor_id")]
    pub sensor_id: String,
    #[serde(alias = "data_timestamp")]
    pub data_timestamp: NaiveDateTime,
    pub sum: NestedReadingValue,
    pub company: String,
    pub simulated: bool,
    #[serde(alias = "avg_val")]
    pub avg_val: f64,
    #[serde(alias = "total_count")]
    pub total_count: u32,
    pub residue: String,
    pub raw: Option<Value>, // Add other fields as needed to match your data structure
}

impl Record {
    pub fn new(
        id: String,
        date: NaiveDateTime,
        value: NestedReadingValue,
        company: String,
        sensor_id: String,
        raw: Option<Value>,
    ) -> Self {
        Record {
            id,
            sensor_id,
            data_timestamp: date,
            sum: value,
            company,
            simulated: false,
            avg_val: 0.0,
            total_count: 0,
            residue: String::new(),
            raw,
        }
    }

    pub fn f64(&self) -> f64 {
        self.sum.as_f64().unwrap_or_default()
    }
}
