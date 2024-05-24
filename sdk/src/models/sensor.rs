use std::collections::HashMap;

use alvarium_sdk_rust::annotations::Annotation;
use chrono::NaiveDateTime;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sensors {
    pub total: u16,
    pub online: u16,
    pub sensors: IndexMap<String, Sensor>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Sensor {
    pub id: String,
    pub total: usize,
    pub avgcf: f32,
    pub equipment: Equipment,
    pub readings: Vec<(String, Reading)>,
    pub local_time: NaiveDateTime,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Reading {
    pub id: String,
    pub address: String,
    pub timestamp: String,
    pub value: f32,
    pub sheet_data: Option<Value>,
    pub annotations: HashMap<String, Annotation>,
    pub score: f32,
}
