use std::{
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
    sync::Arc,
};

use indexmap::IndexMap;
use rocket_okapi::okapi::schemars;

use super::AnalyticsProfile;
use crate::{
    models::{Equipment, GHGInfo, Notification, ProjectInfo, Record, Sensor, Sensors, ValueSet},
    utils::map_serialize,
};

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
    #[serde(with = "map_serialize")]
    #[serde(default)]
    #[schemars(with = "HashMap<String, ValueSet>")] // Needs to added due to with breaking jsonSchema
    pub value_sets: HashMap<String, ValueSet>,
    #[serde(alias = "calc_data")]
    pub calc_data: Vec<Record>,
}

impl SiteState {
    pub fn get_map(&self) -> HashMap<String, serde_json::Value> {
        let json_value = serde_json::to_value(self).unwrap();
        serde_json::from_value(json_value).unwrap()
    }
}

impl Index<&str> for SiteState {
    type Output = ValueSet;

    fn index(&self, index: &str) -> &Self::Output {
        &self.value_sets[index]
    }
}

impl IndexMut<&str> for SiteState {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.value_sets.entry(index.to_string()).or_default()
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
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub profiles: HashSet<Arc<AnalyticsProfile>>,
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

    pub fn get_analytics_profiles(&self) -> &HashSet<Arc<AnalyticsProfile>> {
        &self.profiles
    }

    pub fn get_analytics_profile(&self, profile_id: String) -> Option<&AnalyticsProfile> {
        self.profiles.iter().find_map(|profile| {
            if profile.id == profile_id {
                Some(profile.as_ref())
            } else {
                None
            }
        })
    }

    pub fn add_analytics_profile(&mut self, profile: Arc<AnalyticsProfile>) -> bool {
        self.profiles.insert(profile)
    }
    pub fn remove_analytics_profile_by_id(&mut self, profile: String) -> bool {
        let profile = self.profiles.iter().find(|p| p.id == profile);
        if let Some(p) = profile {
            self.profiles.remove(&p.clone())
        } else {
            false
        }
    }

    pub fn remove_analytics_profile(&mut self, profile: &Arc<AnalyticsProfile>) -> bool {
        self.profiles.remove(profile)
    }

    pub fn remove_analytics_profile(&mut self, id: &str) {
        let profiles = self.profiles.get_or_insert_with(Vec::new);
        profiles.retain(|p| p.id != id);
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

#[cfg(test)]
mod tests {
    use tokio::test;

    use super::*;

    #[test]
    async fn test_get_map_with_camel_case_keys() {
        let mut value_sets = HashMap::new();
        value_sets.insert("ch4_emission".to_string(), ValueSet::default());
        value_sets.insert("elec_prod".to_string(), ValueSet::default());

        let site_state = SiteState {
            value_sets,
            calc_data: vec![],
        };

        let map = serde_json::to_value(site_state).unwrap();

        assert!(map.get("ch4Emission").is_some());
        assert!(map.get("elecProd").is_some());
        assert!(map.get("calcData").is_some());

        let site_state: SiteState = serde_json::from_value(map).unwrap();

        assert!(site_state.value_sets.contains_key("ch4_emission"));
        assert!(site_state.value_sets.contains_key("elec_prod"));
        assert!(site_state.value_sets.contains_key("calc_data"));
    }
}
