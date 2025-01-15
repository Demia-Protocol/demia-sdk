use crate::{
    clients::FileInfo,
    errors::{StorageError, StorageResult},
};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]

pub struct AssetFileWrapper {
    pub asset: Asset,
    pub file: FileInfo,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", content = "name")]
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
            Asset::Profile(id) => id.to_string(),
            Asset::Site(id) => id.to_string(),
            Asset::Equipment(name) => name.to_string(),
            Asset::Sensor(id) => id.to_string(),
            Asset::Custom(id) => id.to_string(),
            Asset::Link(id) => id.to_string(),
        }
    }

    //
    pub fn from_id(url: String) -> StorageResult<Self> {
        let segments = &url.split('/').collect::<Vec<_>>();
        let segment = segments.last().ok_or(StorageError::InvalidName(url.clone()))?;
        let parts = &segment.split('.').collect::<Vec<_>>();
        let r#type = parts.first().ok_or(StorageError::InvalidName(url.clone()))?;

        // The rest, as IDs may contain a .
        let name = if r#type.len() + 1 < segment.len() {
            segment[r#type.len() + 1..].to_string()
        } else {
            // Shouldnt happen but just in case....
            segment.to_string()
        };

        if *r#type == "site" {
            Ok(Self::Site(name))
        } else if *r#type == "equipment" {
            Ok(Self::Equipment(name))
        } else if *r#type == "sensor" {
            Ok(Self::Sensor(name))
        } else if *r#type == "custom" {
            Ok(Self::Custom(name))
        } else if *r#type == "link" {
            Ok(Self::Link(parts[1].to_string())) // link.site_id, has no file extension
        } else {
            Ok(Self::Custom(name))
        }
    }
}
