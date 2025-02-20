use std::collections::HashMap;

use chrono::NaiveDateTime;
use indexmap::IndexMap;
use rand::Rng;
use rocket_okapi::okapi::schemars;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Missing JsonSchema on alvarium
// use alvarium_sdk_rust::annotations::Annotation;
use crate::models::{Annotation, AnnotationWrap, NestedReadingValue};
use crate::utils::deserialize_null_default;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Sensors {
    pub total: u16,
    pub online: u16,
    pub sensors: IndexMap<String, Sensor>,
    pub unprocessed: HashMap<String, Vec<AnnotationWrap>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Sensor {
    pub id: String,
    pub total: usize,
    // Since we occasionally divide by 0.0 this can become NAN so default to 0, or it will serialize
    // as NAN/null and break deserialization https://github.com/serde-rs/json/issues/202
    #[serde(deserialize_with = "deserialize_null_default")]
    pub avgcf: f32,
    pub equipment: Equipment,
    pub readings: HashMap<String, Reading>,
    pub last_updated: Option<NaiveDateTime>,
    #[serde(default)]
    pub state: SensorStateData,
    pub asset_url: Option<String>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Equipment {
    pub id: String,
    pub group: String,
    pub units: String,
    pub eq_type: String,
    pub name: String,
    pub accuracy: f32,
    pub installed: u16,
    pub serial_no: String,
    pub manufacturer: String,
    #[serde(default)]
    pub asset_url: Option<String>,
}

impl Equipment {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    pub id: String,
    pub address: String,
    pub timestamp: String,
    pub value: NestedReadingValue,
    #[serde(rename = "sheetData")]
    pub sheet_data: Option<Value>,
    pub annotations: HashMap<String, Annotation>,
    pub score: f32,
    pub unit: Option<String>,
}

impl Default for Sensor {
    fn default() -> Self {
        let s: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        Sensor {
            id: format!("Sensor_{}", s),
            total: 0,
            avgcf: 0.0,
            equipment: Equipment::default(),
            readings: HashMap::new(),
            last_updated: None,
            state: SensorStateData::default(),
            asset_url: None,
        }
    }
}

impl From<Equipment> for Sensor {
    fn from(equipment: Equipment) -> Self {
        Sensor {
            id: equipment.name.clone(),
            total: 0,
            avgcf: 0.0,
            equipment,
            readings: HashMap::new(),
            last_updated: None,
            state: SensorStateData::default(),
            asset_url: None,
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SensorStateData {
    pub real_time_flow: f64,
    pub total_flow: f64,
    pub current_day_avg: f64,
}
