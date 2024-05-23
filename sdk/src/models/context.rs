use std::collections::HashMap;

use alvarium_sdk_rust::annotations::Annotation;
use chrono::NaiveDateTime;
use identity_demia::prelude::DemiaDID;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{Card, SensorStateData, Site};

/// Context for "login" page
#[derive(serde::Serialize)]
pub struct LoginContext {}

/// Context for "create_identity" page
#[derive(serde::Serialize)]
pub struct CreateIdentityContext {}

/// Context for "identity" page
#[derive(serde::Serialize)]
pub struct IdentityContext<'a> {
    pub user: UserContext,
    pub doc_id: &'a DemiaDID,
    pub methods: Vec<VerificationMethodContext<'a>>,
}

/// Context for "identity" page
#[derive(serde::Serialize)]
pub struct VerificationMethodContext<'a> {
    pub type_: String,
    pub id: String,
    pub fragment: &'a str,
    pub controller: String,
}

/// Context for "new_sensor" page
#[derive(serde::Serialize)]
pub struct NewSensorContext {
    /// The ID we select (The site we came from)
    pub selected_id: String,
    // All our available site IDs
    pub site_ids: Vec<String>,
}

/// Context for "new_stream" page
#[derive(serde::Serialize)]
pub struct NewStreamContext {
    pub user: UserContext,
}

/// Context for a user
#[derive(serde::Serialize, Default)]
pub struct UserContext {
    /// Streams address, if there is already a stream linked
    pub address: Option<String>,
    /// Streams identifier
    pub author: Option<String>,
    /// User identifier (DID)
    pub identifier: Option<String>,
    /// The username
    pub username: Option<String>,
    /// Keycloak ID
    pub keycloak_id: Option<String>,
}

/// Context for "site_info" page
#[derive(serde::Serialize)]
pub struct DashboardContext {
    pub cards: Vec<Card>,
    pub site: Site,
    pub sites: Vec<Site>,
}

/// Context for "overview" page
#[derive(serde::Serialize)]
pub struct OverviewContext<'a> {
    pub sites: Vec<&'a Site>,
}

/// Context for "<site_id>/sensors" page
#[derive(serde::Serialize)]
pub struct SensorsContext {
    pub site: Site,
    pub sensors: Vec<SensorDashboardContext>,
}

/// Context for "<site_id>/sensors/<sensor_id>" page
#[derive(serde::Serialize)]
pub struct SensorContext {
    pub site: Site,
    pub sensor: SensorDashboardContext,
}

/// Context for "<site_id>/analytics" page
#[derive(serde::Serialize)]
pub struct AnalyticsContext {
    pub site: Site,
}

/// Equipment details for a given data source
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EquipmentDashboardContext {
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

/// A Context that contains granular reading information for the dashboard
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ReadingDashboardContext {
    pub id: String,
    pub address: String,
    pub timestamp: String,
    pub value: f32,
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

/// HEDEREA

///  Context for "<site_id>/hedera/register" page
#[derive(serde::Serialize)]
pub struct HederaRegisterContext<'a> {
    pub site_id: &'a str,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct VCWrap {
    #[serde(rename = "msgId")]
    pub msg_id: String,
    pub vc: String,
}

///  Context for "<site_id>/hedera" page
#[derive(serde::Serialize)]
pub struct HederaContext<'a> {
    pub doc_id: String,
    pub methods: Vec<VerificationMethodContext<'a>>,
    pub vcs: Vec<VCWrap>,
}
