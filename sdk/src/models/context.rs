use std::collections::HashMap;

use alvarium_sdk_rust::annotations::Annotation;
use chrono::NaiveDateTime;
use identity_demia::prelude::DemiaDID;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{Card, SensorStateData, Site};

/// Equipment details for a given data source
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EquipmentDashboardContext {
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

/// A Context that contains granular reading information for the dashboard
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ReadingDashboardContext {
    pub id: String,
    pub address: String,
    pub timestamp: String,
    pub value: f32,
    #[serde(rename = "sheetData")]
    pub sheet_data: Option<Value>,
    pub annotations: HashMap<String, Annotation>,
    pub score: f32,
    pub processed: bool,
}

/// Dashboard Context including readings, averages, state data etc
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorDashboardContext {
    pub id: String,
    pub total: usize,
    pub avgcf: f32,
    pub equipment: EquipmentDashboardContext,
    pub readings: Vec<(String, ReadingDashboardContext)>,
    pub state_data: SensorStateData,
    pub local_time: NaiveDateTime,
}

impl Default for SensorDashboardContext {
    fn default() -> Self {
        let s: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        SensorDashboardContext {
            id: format!("Sensor_{}", s),
            total: 0,
            avgcf: 0.0,
            equipment: EquipmentDashboardContext::default(),
            readings: Vec::new(),
            state_data: SensorStateData::default(),
            local_time: chrono::Local::now().naive_local(),
        }
    }
}

impl From<EquipmentDashboardContext> for SensorDashboardContext {
    fn from(equipment: EquipmentDashboardContext) -> Self {
        SensorDashboardContext {
            id: equipment.name.clone(),
            total: 0,
            avgcf: 0.0,
            equipment,
            readings: Vec::new(),
            state_data: SensorStateData::default(),
            local_time: chrono::Local::now().naive_local(),
        }
    }
}
