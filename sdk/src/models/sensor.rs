use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::models::SensorDashboardContext;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sensors {
    pub total: u16,
    pub online: u16,
    pub sensors: IndexMap<String, SensorDashboardContext>,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sensor {}

/// Holds state data for the sensor to be represented in the dashboard
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct SensorStateData {
    pub real_time_flow: f64,
    pub total_flow: f64,
    pub current_day_avg: f64,
}
