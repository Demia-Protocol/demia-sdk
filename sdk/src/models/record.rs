use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: String,
    pub sensor_id: String,
    pub data_timestamp: NaiveDateTime,
    pub sum: f64,
    pub company: String,
    pub simulated: bool,
    pub avg_val: f64,
    pub total_count: u32,
    pub residue: String,
    pub raw: Option<Value>, // Add other fields as needed to match your data structure
}

impl Record {
    pub fn new(
        id: String,
        date: NaiveDateTime,
        value: f64,
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
}
