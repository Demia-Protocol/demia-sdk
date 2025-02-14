use std::{
    collections::{HashMap, HashSet},
    ops::{Index, IndexMut},
    sync::Arc,
};
use chrono::{DateTime, Utc};
use futures_util::{FutureExt, future::BoxFuture};
use indexmap::IndexMap;
use rocket_okapi::okapi::schemars;

use super::{update_sensor_readings, AnalyticsProfile, Asset, InputParameter, Parameter, Reading};
use crate::{
    clients::{AwsClient, FileInfo, StorageClient},
    errors::{StorageResult, SdkResult},
    models::{Equipment, GHGInfo, Notification, ProjectInfo, Record, Sensor, Sensors, ValueSet},
    utils::{default_as_true, map_serialize},
};
use crate::analytics::all_daily_averages;
use crate::analytics::defaults::analytics::{equation6, run_equation};
use crate::analytics::defaults::constants::FEEDSTOCK_TYPE;
use crate::errors::{Error, UserError};

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


#[derive(serde::Serialize, serde::Deserialize, Debug, schemars::JsonSchema)]
pub struct NewStreamRequest {
    pub address: String,
    pub author: String,
    #[serde(rename = "subIdentifier")]
    pub sub_identifier: String,
    pub project: NewSite,
}


/// Context for "create_new_project" Response
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateProjectResponse {
    pub project_id: String,
    pub project_name: String,
    pub announcement: String,
    pub location: SiteLocation,
    pub sensors: Sensors,
}


impl From<&Site> for CreateProjectResponse {
    fn from(site: &Site) -> Self {
        Self {
            project_id: site.project_id.clone(),
            project_name: site.project_name.clone(),
            announcement: site.project_announcement.clone(),
            location: site.location.clone(),
            sensors: site.sensors.clone(),
        }
    }
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

#[derive(Default, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
    #[serde(default)]
    pub profiles: HashSet<Arc<AnalyticsProfile>>,
    #[serde(default)]
    pub asset_url: Option<String>,
    // Custom assets used in displaying the site
    #[serde(skip, default)]
    pub assets: Option<HashMap<Asset, FileInfo>>,
    #[serde(default = "default_as_true")]
    pub loading: bool
}

impl std::fmt::Debug for Site {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Site")
            .field("project_id", &self.project_id)
            .field("project_announcement", &self.project_announcement)
            .field("project_name", &self.project_name)
            .field("sensors", &self.sensors)
            .field("assets", &self.assets)
            .finish()
    }
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

    pub async fn fetch_custom_assets(
        &self,
        storage: &StorageClient<AwsClient>,
    ) -> StorageResult<HashMap<Asset, FileInfo>> {
        let mut assets = HashMap::new();
        Site::fetch_site_assets(self.project_id.clone(), storage, &mut assets).await?;
        Ok(assets)
    }

    pub fn fetch_site_assets<'a>(
        project_id: String,
        storage: &'a StorageClient<AwsClient>,
        assets: &'a mut HashMap<Asset, FileInfo>,
    ) -> BoxFuture<'a, StorageResult<()>> {
        async move {
            let path = format!("assets/{}/", project_id);
            let files = storage.list_objects(path.to_string(), false, true).await?;

            let mut links = vec![];
            for file in files {
                if file.name.ends_with("/") {
                    continue;
                }

                let asset = Asset::from_id(file.name.clone())?;
                if let Asset::Link(l) = &asset {
                    if !assets.contains_key(&asset) && l != &project_id {
                        links.push(l.clone());
                    } else {
                        continue;
                    }
                }

                assets.insert(asset, file);
            }

            for site_id in links {
                Site::fetch_site_assets(site_id.to_string(), storage, assets).await?;
            }

            Ok(())
        }
        .boxed()
    }

    pub async fn get_assets(&self) -> Option<&HashMap<Asset, FileInfo>> {
        self.assets.as_ref()
    }

    pub fn set_assets(&mut self, assets: HashMap<Asset, FileInfo>) {
        self.assets.get_or_insert_with(HashMap::new).extend(assets);
    }

    pub fn sensor_by_id(&self, sensor_id: &str) -> SdkResult<&Sensor> {
        Ok(self.sensors.sensors.get(sensor_id).ok_or(UserError::SiteMissingSensor)?)
    }

    pub async fn update_sensors(&mut self) -> SdkResult<&mut Self> {
        self.sensors.online = 0;
        let sensors = &mut self.sensors.sensors;
        log::info!("Sensors preloaded: {}", sensors.len(),);
        for sensor in sensors.values_mut() {
            let existing_readings = &mut sensor.readings;
            for (_id, reading) in existing_readings.iter_mut() {
                update_annotations_and_score(reading);
                // break;
            }
        }
        update_sensor_readings(sensors);

        // For each message in the messages vector
        self.sensors.total = self.sensors.sensors.len() as u16;

        self.calculate_avg_dcf().await?;
        self.calculate_ghg_info().await?;

        Ok(self)
    }

    async fn calculate_avg_dcf(&mut self) -> SdkResult<()> {
        let mut avg_dcf = 0.0;
        let mut total = 0.0;

        let now = Utc::now();
        let ten_years_ago = now - chrono::Duration::days(365 * 10);

        let records = self.get_records(ten_years_ago, now).await?;
        let daily = all_daily_averages(&records).await;

        let mut daily_readings = Vec::new();
        let local_time = chrono::Utc::now().naive_local();
        let mut online_sensors = 0;

        self.sensors
            .sensors
            .values_mut()
            .try_for_each(|sensor| -> SdkResult<()> {
                let state = &mut sensor.state;
                for (_, reading) in sensor
                    .readings
                    .iter()
                    .filter(|(_, reading)| !reading.timestamp.is_empty())
                {
                    log::debug!("\tReading: {}", reading.id);
                    total += 1.0;
                    let mut value = reading.value.as_f64().unwrap_or_default();
                    if value > 10_000.0 {
                        value /= 1_000_000_000.0 // Nm3 -> m3
                    };
                    state.total_flow += value;
                    avg_dcf += reading.score;

                    let reading_time = chrono::DateTime::parse_from_rfc3339(&reading.timestamp)
                        .map_err(|e| Error::Calculations(e.to_string()))?
                        .naive_local();
                    let duration_since_reading = local_time.signed_duration_since(reading_time);

                    if duration_since_reading.num_hours() < 24 {
                        daily_readings.push(reading.clone());
                    }
                    if duration_since_reading.num_minutes() < 60 {
                        state.real_time_flow += value;
                    }

                    let offset = match sensor.equipment.group.as_str() {
                        "Manual" => 60 * 24,
                        _ => 60,
                    };
                    if chrono::Utc::now()
                        .signed_duration_since(reading_time.and_utc())
                        .num_minutes()
                        < offset
                    {
                        online_sensors += 1;
                    }
                }

                if let Some(updated) = sensor.last_updated {
                    let updated_date = updated.date();
                    if let Some(day) = daily.get(&updated_date) {
                        if let Some(sensor_avg) = day.sensors.get(&sensor.id) {
                            state.total_flow = sensor_avg
                                .records
                                .iter()
                                .map(|record| {
                                    if record.f64() > 10_000.0 {
                                        record.f64() / 1_000_000_000.0
                                    } else {
                                        record.f64()
                                    }
                                })
                                .sum();
                            state.real_time_flow = sensor_avg.records.iter().last().map_or(0.0, |record| {
                                if record.f64() > 10_000.0 {
                                    record.f64() / 1_000_000_000.0
                                } else {
                                    record.f64()
                                }
                            });

                            state.current_day_avg = if sensor_avg.avg_val > 10_000.0 {
                                sensor_avg.avg_val / 1_000_000_000.0
                            } else {
                                sensor_avg.avg_val
                            };
                        }
                    }
                    log::debug!("\tSensor total flow: {}", state.total_flow);
                }
                Ok(())
            })?;

        log::info!(
            "Avg dcf: {}, total annotations: {}, {}",
            avg_dcf,
            total,
            avg_dcf * 100.0 / total
        );
        self.avg_dcf = Some(format!("{:.1}", (avg_dcf * 100.0 / total)));
        self.sensors.online = online_sensors;
        Ok(())
    }

    async fn run_equations(&mut self, feedstock_data: &mut [Record]) {
        let mut value_sets = HashMap::new();
        for profile in &self.profiles {
            for calculation in &profile.calculations {
                let inputs: Vec<InputParameter> = calculation
                    .parameters
                    .iter()
                    .filter_map(|p| match p {
                        Parameter::Input(p) => Some(p.clone()),
                        _ => None,
                    })
                    .collect();
                let results = run_equation(&calculation.id, &inputs, feedstock_data).await.unwrap();
                value_sets.insert(calculation.id.clone(), results);
            }
        }
        self.state_data.value_sets.extend(value_sets);
    }

    pub async fn get_records(&mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> SdkResult<Vec<Record>> {
        let dev = self.project_info.developer.clone();
        let records = &mut self.records;
        let mut new_records = vec![];

        self.sensors.sensors.values_mut().for_each(|sensor| {
            for reading in sensor.readings.values_mut().filter(|r| !r.timestamp.is_empty()) {
                let timestamp = chrono::DateTime::parse_from_rfc3339(&reading.timestamp).unwrap();
                if timestamp >= start && timestamp <= end {
                    let record = records.entry(reading.address.clone()).or_insert({
                        let mut record = Record::new(
                            reading.id.clone(),
                            chrono::DateTime::parse_from_rfc3339(&reading.timestamp)
                                .unwrap()
                                .naive_local(),
                            reading.value.clone(),
                            dev.clone(),
                            sensor.id.clone(),
                            reading.sheet_data.clone(),
                        );

                        if let Some(sheet) = &mut reading.sheet_data {
                            if let Some(residue) = sheet.get("Residuo") {
                                record.residue = residue.to_string().replace('"', "");
                            }
                            if let Some(company) = sheet.get("Empresa") {
                                record.company = company.to_string().replace('"', "");
                            }
                        }
                        record
                    });

                    new_records.push(record.clone())
                }
            }
        });

        new_records.sort_by(|a, b| a.data_timestamp.cmp(&b.data_timestamp));
        Ok(new_records)
    }

    async fn calculate_ghg_info(&mut self) -> SdkResult<()> {
        let now = Utc::now();
        let thirty_days_ago = now - chrono::Duration::days(30);
        let one_year_ago = now - chrono::Duration::days(365);

        let sum_30_records = self.get_records(thirty_days_ago, now).await?;
        let mut sum_365_records = self.get_records(one_year_ago, now).await?;

        let r = equation6(vec![FEEDSTOCK_TYPE.clone()].as_slice(), &sum_30_records).await?;

        let val = format!("{:.3}", r.data.iter().map(|(_, v)| v).sum::<f64>());
        let ghg_last_30_days = &mut self.ghg_last_30_days;
        if ghg_last_30_days.label.eq("GHG emission reductions last 30 day") {
            ghg_last_30_days.value.clone_from(&val);
            ghg_last_30_days.data.push(val.clone())
        } else {
            self.ghg_last_30_days = GHGInfo::new(&val, "t CO2e", "GHG emission reductions last 30 day")
        }

        let val = format!("{:.3}", r.data.iter().map(|(_, v)| v).sum::<f64>() * 12.0);
        let ghg_annual = &mut self.ghg_annual;
        if ghg_annual.label.eq("Annual GHG emission reductions") {
            ghg_annual.value.clone_from(&val);
            ghg_annual.data.push(val.clone())
        } else {
            self.ghg_annual = GHGInfo::new(&val, "t CO2e", "Annual GHG emission reductions")
        }

        self.run_equations(&mut sum_365_records).await;
        Ok(())
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



impl From<&Site> for NewSite {
    fn from(site: &Site) -> Self {
        Self {
            id: site.project_id.clone(),
            name: site.project_name.clone(),
            location: site.location.clone(),
            sensors: site
                .sensors
                .sensors
                .iter()
                .map(|(id, s)| (id.clone(), s.equipment.clone()))
                .collect(),
            project: site.project_info.clone(),
            announcement: site.project_announcement.clone(),
        }
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

fn update_annotations_and_score(reading: &mut Reading) {
    reading.score = 0.0;
    if !reading.timestamp.is_empty() {
        for ann in reading.annotations.values() {
            if ann.0.is_satisfied {
                match ann.0.kind.kind() {
                    "automated" => reading.score += 3.33333 / 10.0,
                    "source" => reading.score += 1.33333 / 10.0,
                    "tls" => reading.score += 2.00000 / 10.0,
                    "pki" => reading.score += 3.333333 / 10.0,
                    _ => (),
                }
            }
        }
    }
}
