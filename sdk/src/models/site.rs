use std::collections::HashMap;

use crate::{
    models::{
        sensor::*, EquipmentDashboardContext, GHGInfo, Notification, ProjectInfo, ValueSet,
    },
    utils::Record,
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SiteLocation {
    pub address: String,
    pub lat: f32,
    pub lon: f32,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewSite {
    pub id: String,
    pub name: String,
    pub location: SiteLocation,
    pub sensors: Vec<(String, EquipmentDashboardContext)>,
    pub project: ProjectInfo,
    pub announcement: String,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SiteState {
    pub ch4_emission: ValueSet,
    pub wws: ValueSet,
    pub elec_prod: ValueSet,
    pub fossil_fuel: ValueSet,
    pub ch4: ValueSet,
    pub bde: ValueSet,
    pub an_dig: ValueSet,
    pub biogas_adjusted: ValueSet,
    pub effluent_storage: ValueSet,
    pub ch4_destroyed: ValueSet,
    pub e_project: ValueSet,
    pub calc_data: Vec<Record>,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Site {
    pub id: String,
    pub announcement: String,
    pub name: String,
    pub location: SiteLocation,
    pub sensors: Sensors,
    pub notifications: Vec<Notification>,
    pub project: ProjectInfo,
    pub ghg_last_30_days: GHGInfo,
    #[serde(default)]
    pub records: HashMap<String, Record>,
    pub ghg_annual: GHGInfo,
    pub state_data: SiteState,
    pub avg_dcf: Option<String>,
}
