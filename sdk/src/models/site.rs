use std::collections::HashMap;

use indexmap::IndexMap;
use rocket_okapi::okapi::schemars;

use crate::models::{Equipment, GHGInfo, Notification, ProjectInfo, Record, Sensor, Sensors, ValueSet};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SiteLocation {
    pub address: String,
    pub lat: f32,
    #[serde(alias = "long")]
    pub lon: f32,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct NewSite {
    pub id: String,
    pub name: String,
    pub location: SiteLocation,
    pub sensors: Vec<(String, Equipment)>,
    pub project: ProjectInfo,
    pub announcement: String,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SiteState {
    // ToDo: Temporary alias's to be replaced with a nesting of value_sets with arbitrary names
    #[serde(alias = "ch4_emission")]
    pub ch4_emission: ValueSet,
    pub wws: ValueSet,
    #[serde(alias = "elec_prod")]
    pub elec_prod: ValueSet,
    #[serde(alias = "fossil_fuel")]
    pub fossil_fuel: ValueSet,
    pub ch4: ValueSet,
    pub bde: ValueSet,
    #[serde(alias = "an_dig")]
    pub an_dig: ValueSet,
    #[serde(alias = "biogas_adjusted")]
    pub biogas_adjusted: ValueSet,
    #[serde(alias = "effluent_storage")]
    pub effluent_storage: ValueSet,
    #[serde(alias = "ch4_destroyed")]
    pub ch4_destroyed: ValueSet,
    #[serde(alias = "e_project")]
    pub e_project: ValueSet,
    #[serde(alias = "calc_data")]
    pub calc_data: Vec<Record>,
}

impl SiteState {
    pub fn get_map(&self) -> HashMap<String, serde_json::Value> {
        // map to string to include labels
        let str = serde_json::to_string(self).unwrap();
        // parse the string to a HashMap
        let map: HashMap<String, serde_json::Value> = serde_json::from_str(&str).unwrap();
        map
    }
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    #[serde(alias = "id")]
    pub project_id: String,
    #[serde(alias = "announcement")]
    pub project_announcement: String,
    #[serde(alias = "name")]
    pub project_name: String,
    pub location: SiteLocation,
    pub sensors: Sensors,
    pub notifications: Vec<Notification>,
    #[serde(alias = "project")]
    pub project_info: ProjectInfo,
    pub ghg_last_30_days: GHGInfo,
    #[serde(default)]
    pub records: HashMap<String, Record>,
    pub ghg_annual: GHGInfo,
    #[serde(alias = "state_data", default)]
    pub state_data: SiteState,
    pub avg_dcf: Option<String>,
}

impl Site {
    pub fn new(
        id: String,
        announcement: String,
        name: String,
        location: SiteLocation,
        sensors: Sensors,
        project: ProjectInfo,
    ) -> Self {
        Self {
            project_id: id,
            project_name: name,
            location,
            sensors,
            project_info: project,
            project_announcement: announcement,
            ..Default::default()
        }
    }
}

impl From<&NewSite> for Site {
    fn from(new_site: &NewSite) -> Self {
        let mut sensors = IndexMap::new();
        for (sensor, equipment) in &new_site.sensors {
            let context = Sensor {
                id: sensor.clone(),
                equipment: equipment.clone(),
                ..Default::default()
            };
            sensors.insert(sensor.clone(), context);
        }
        let sensors = Sensors {
            total: 0,
            online: 0,
            sensors,
            unprocessed: HashMap::new(),
        };
        Site::new(
            new_site.id.clone(),
            new_site.announcement.clone(),
            new_site.name.clone(),
            new_site.location.clone(),
            sensors,
            new_site.project.clone(),
        )
    }
}
