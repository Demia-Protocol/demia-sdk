use std::collections::HashMap;

use alvarium_sdk_rust::annotations::Annotation;
use chrono::NaiveDateTime;
use indexmap::IndexMap;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sensors {
    pub total: u16,
    pub online: u16,
    pub sensors: IndexMap<String, Sensor>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
            readings: Vec::new(),
            local_time: chrono::Local::now().naive_local(),
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
            readings: Vec::new(),
            local_time: chrono::Local::now().naive_local(),
        }
    }
}
