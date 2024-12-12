use std::sync::{Arc, LazyLock};

use default_molina::MOLINA_PROFILE;

use crate::models::AnalyticsProfile;

pub mod constants;
pub mod default_molina;

pub static PROFILES: LazyLock<Vec<Arc<AnalyticsProfile>>> = LazyLock::new(|| vec![(*MOLINA_PROFILE).clone()]);

pub fn profile_by_id(profile_id: &str) -> Option<Arc<AnalyticsProfile>> {
    PROFILES
        .iter()
        .find(|&profile| profile.id == profile_id)
        .map(|v| v.clone())
}
