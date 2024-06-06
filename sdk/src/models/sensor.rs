use std::collections::HashMap;

use alvarium_sdk_rust::annotations::Annotation;
use chrono::NaiveDateTime;
use indexmap::IndexMap;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::AnnotationWrap;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sensors {
    pub total: u16,
    pub online: u16,
    pub sensors: IndexMap<String, Sensor>,
    pub unprocessed: HashMap<String, Vec<AnnotationWrap>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sensor {
    pub id: String,
    pub total: usize,
    pub avgcf: f32,
    pub equipment: Equipment,
    pub readings: HashMap<String, Reading>,
    pub last_updated: NaiveDateTime,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Equipment {
    pub id: String,
    pub group: String,
    pub units: String,
    #[serde(rename = "eqType")]
    pub eq_type: String,
    pub name: String,
    pub accuracy: f32,
    pub installed: u16,
    #[serde(rename = "serialNo")]
    pub serial_no: String,
    pub manufacturer: String,
}

impl Equipment {
    pub fn new(
        id: String,
        name: String,
        group: String,
        units: String,
        eq_type: String,
        accuracy: f32,
        installed: u16,
        serial_no: String,
        manufacturer: String,
    ) -> Self {
        Self {
            id,
            name,
            group,
            units,
            eq_type,
            accuracy,
            installed,
            serial_no,
            manufacturer,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Reading {
    pub id: String,
    pub address: String,
    pub timestamp: String,
    pub value: f32,
    #[serde(rename = "sheetData")]
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
            readings: HashMap::new(),
            last_updated: chrono::Local::now().naive_local(),
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
            last_updated: chrono::Local::now().naive_local(),
        }
    }
}
