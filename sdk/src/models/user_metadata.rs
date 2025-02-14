use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use chrono::Utc;
use lets::address::Address;
use log::warn;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::configuration::GuardianConfigs;
use crate::models::{Notification, NotificationType, StreamsAddresses, TokenWrap, Site};
use crate::errors::{GuardianError, GuardianResult, UserError, UserResult as Result};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserMetadata {
    #[serde(deserialize_with = "deserialize_sites")]
    pub sites: HashMap<String, Site>,
    pub addresses: Option<StreamsAddresses>,
    #[serde(default)]
    pub guardian: HashMap<String, GuardianConfigs>,
    #[serde(default)]
    pub notifications: Vec<Notification>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub identity_loaded: bool,
}


/// Custom deserializer for handling both direct and wrapped `SdkSite` formats.
fn deserialize_sites<'de, D>(deserializer: D) -> std::result::Result<HashMap<String, Site>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_map: HashMap<String, Value> = HashMap::deserialize(deserializer)?;
    let mut sites = HashMap::new();

    for (key, value) in raw_map {
        if let Ok(site) = serde_json::from_value::<Site>(value.clone()) {
            sites.insert(key, site);
        } else if let Ok(wrapped) = serde_json::from_value::<WrappedSite>(value.clone()) {
            sites.insert(key, wrapped.sdk_site);
        } else {
            return Err(serde::de::Error::custom("Invalid site format"));
        }
    }

    Ok(sites)
}

#[derive(Debug, serde::Deserialize)]
struct WrappedSite {
    #[serde(rename = "sdkSite")]
    sdk_site: Site,
    loading: bool, // Ignored during conversion
}


#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct UserProfile {
    pub username: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub did: String,
    pub sites: Vec<UserSite>,
}


#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct UserSite {
    pub name: String,
    pub address: String,
    pub id: String,
    #[serde(rename = "projectDeveloper")]
    pub project_developer: String,
}


impl UserMetadata {
    pub fn site_by_id(&self, site_id: &str) -> Result<&Site> {
        self.sites
            .get(site_id)
            .ok_or(UserError::SiteNotFound(site_id.to_string()))
    }

    pub async fn add_site(&mut self, site_id: String, site: Site, token: &TokenWrap) -> Result<()> {
        let site_name = site.project_name.clone();
        self.sites.insert(site_id, site);
        log::debug!("Metadata stored: {:?}", self);
        let id = token.get_email().unwrap_or(token.get_sub().unwrap());
        self.notifications.push(Notification {
            user: id,
            message: format!("Project {} added", site_name),
            timestamp: Utc::now(),
            notification_type: NotificationType::NewSite,
        });
        Ok(())
    }

    pub fn add_guardian(&mut self, site_id: String, guardian: GuardianConfigs) {
        self.guardian.insert(site_id.clone(), guardian);
    }

    pub fn get_guardian(&mut self, site_id: &str) -> GuardianResult<&mut GuardianConfigs> {
        match self.guardian.get_mut(site_id) {
            Some(guardian) => Ok(guardian),
            None => Err(GuardianError::NoGuardianConfig(site_id.to_string())),
        }
    }

    pub fn remove_guardian(&mut self, site_id: &str) {
        if self.guardian.remove(site_id).is_none() {
            warn!("No guardian found for site: {}", site_id);
        }
    }

    pub async fn get_streams_addresses(&self) -> Vec<Address> {
        match &self.addresses {
            Some(addresses) => addresses
                .0
                .iter()
                .map(|addr| Address::from_str(addr).unwrap())
                .collect(),
            None => Vec::new(),
        }
    }

    pub async fn add_streams_address(&mut self, address: String) {
        match self.addresses.as_mut() {
            Some(addresses) => {
                addresses.0.insert(address.clone());
            }
            None => self.addresses = Some(StreamsAddresses(HashSet::from([address]))),
        }
    }

    pub async fn remove_streams_address(&mut self, address: &String) {
        if let Some(addresses) = self.addresses.as_mut() {
            addresses.0.remove(address);
        }
    }
}