use crate::errors::{StorageError, StorageResult};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum Asset {
    /// User ID
    Profile(String),
    /// Site ID
    Site(String),
    /// Sensor name
    Sensor(String),
    /// Equipment name
    Equipment(String),
    /// custom ID
    Custom(String),
    /// Site ID
    Link(String),
}

impl Asset {
    pub fn storage_path(&self) -> String {
        match self {
            Asset::Profile(id) => format!("{}", id),
            Asset::Site(id) => format!("{}", id),
            Asset::Equipment(name) => format!("{}", name),
            Asset::Sensor(id) => format!("{}", id),
            Asset::Custom(id) => format!("{}", id),
            Asset::Link(id) => format!("{}", id),
        }
    }

    //
    pub fn from_id(url: String) -> StorageResult<Self> {
        let segments = &url.split('/').collect::<Vec<_>>();
        let segment = segments.last().ok_or(StorageError::InvalidName(url.clone()))?;
        let parts = &segment.split('.').collect::<Vec<_>>();
        let name = parts.first().ok_or(StorageError::InvalidName(url.clone()))?;
        if *name == "site" {
            Ok(Self::Site(url))
        } else if *name == "equipment" {
            Ok(Self::Equipment(url))
        } else if *name == "sensor" {
            Ok(Self::Sensor(url))
        } else if *name == "custom" {
            Ok(Self::Custom(url))
        } else if *name == "link" {
            Ok(Self::Link(parts[1].to_string())) // link.site_id, has no file extension
        } else {
            Ok(Self::Custom(url))
        }
    }
}
