use std::collections::HashMap;

use crate::{
    configuration::GuardianConfigs,
    models::{Notification, Site, StreamsAddresses},
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserMetadata {
    pub sites: HashMap<String, Site>,
    pub addresses: Option<StreamsAddresses>,
    #[serde(default)]
    pub guardian: HashMap<String, GuardianConfigs>,
    #[serde(default)]
    pub notifications: Vec<Notification>,
}
