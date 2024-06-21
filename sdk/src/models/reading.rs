use std::str::FromStr;

use alvarium_sdk_rust::annotations::Annotation;
use chrono::{DateTime, Utc};
use rocket_okapi::okapi::schemars;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::json_scheme_wrap::AnnotationDef;

/// Represents the Wrapper for an Annotation, including the reading_id that they represent
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
pub struct AnnotationWrap {
    pub reading_id: String,
    #[serde(with = "AnnotationDef")]
    pub annotation: Annotation,
}

/// Represents a Reading Type for demo data
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum WrappedReadingType {
    Sensor(SensorReading),
    Sheet(SheetReading),
}

impl WrappedReadingType {
    pub fn id(&self) -> &str {
        match self {
            WrappedReadingType::Sensor(reading) => &reading.id,
            WrappedReadingType::Sheet(sheet) => &sheet.id,
        }
    }

    /// Extract the data from the sheets
    // TODO: Make a more universal way of extracting custom columns with expected value type
    pub fn val(&self) -> f32 {
        match self {
            WrappedReadingType::Sensor(reading) => reading.value,
            WrappedReadingType::Sheet(sheet) => match sheet.value.get("Toneladas ") {
                Some(str) => f32::from_str(&str.to_string().replace([',', '"'], "")).unwrap(),
                None => match sheet.value.get("Biogás Generado (Nm3)") {
                    Some(str) => f32::from_str(&str.to_string().replace([',', '"'], "")).unwrap(),
                    None => 0.0,
                },
            },
        }
    }

    /// Retrieve the timestamp of the reading
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            WrappedReadingType::Sensor(reading) => reading.timestamp,
            WrappedReadingType::Sheet(sheet) => sheet.timestamp,
        }
    }
}

/// A reading from an automated sensor source
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SensorReading {
    pub id: String,
    pub value: f32,
    pub timestamp: DateTime<Utc>,
}

/// A reading from a spreadsheet in json form
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SheetReading {
    pub id: String,
    pub value: Value,
    pub timestamp: DateTime<Utc>,
}

/// Wraps the reading with an address and id reference
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ReadingWrap {
    pub id: String,
    pub address: String,
    pub reading: WrappedReadingType,
}

// TODO: Determine required fields vs variable fields to isolate necessary data
#[derive(Debug, Clone, Default, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SheetData {
    #[serde(rename = "dataTimestamp")]
    pub date_timestamp: i64,
    #[serde(rename = "Empresa")]
    pub empresa: String,
    #[serde(rename = "Fecha")]
    pub fecha: String,
    #[serde(rename = "Toneladas ")]
    pub toneladas: f32,
    pub simulated: String,
}
