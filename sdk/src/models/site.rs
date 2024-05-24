use std::collections::HashMap;
use indexmap::IndexMap;
use crate::{
    models::{
        SensorDashboardContext, EquipmentDashboardContext, GHGInfo, Notification, ProjectInfo, ValueSet, Sensors
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

impl Site {
    pub fn new(
        id: String,
        announcement: String,
        name: String,
        location: SiteLocation,
        sensors: Sensors,
        project: ProjectInfo,
    ) -> Self{
        Self {
            id,
            name,
            location,
            sensors,
            project,
            announcement,
            ..Default::default()
        }
    }
}

impl From<&NewSite> for Site {
    fn from(new_site: &NewSite) -> Self {
        let mut sensors = IndexMap::new();
        for (sensor, equipment) in &new_site.sensors {
            let context = SensorDashboardContext {
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
