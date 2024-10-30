use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

use alvarium_sdk_rust::annotations::Annotation;
use chrono::{DateTime, Datelike, Timelike, Utc};
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
    Nested(NestedReading),
}

impl WrappedReadingType {
    pub fn id(&self) -> &str {
        match self {
            WrappedReadingType::Sensor(reading) => &reading.id,
            WrappedReadingType::Sheet(sheet) => &sheet.id,
            WrappedReadingType::Nested(nested) => &nested.id,
        }
    }

    /// Extract the data from the sheets
    // TODO: Make a more universal way of extracting custom columns with expected value type
    pub fn val(&self) -> NestedReadingValue {
        match self {
            WrappedReadingType::Sensor(reading) => NestedReadingValue::Float(reading.value),
            WrappedReadingType::Sheet(sheet) => match sheet.value.get("Toneladas ") {
                Some(str) => NestedReadingValue::Float(
                    f32::from_str(&str.to_string().replace([',', '"'], "")).unwrap()
                ),
                None => match sheet.value.get("Biogás Generado (Nm3)") {
                    Some(str) => NestedReadingValue::Float(
                        f32::from_str(&str.to_string().replace([',', '"'], "")).unwrap()
                    ),
                    None => NestedReadingValue::Empty,
                },
            },
            WrappedReadingType::Nested(nested) => nested.value.clone(),
        }
    }

    /// Retrieve the timestamp of the reading
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            WrappedReadingType::Sensor(reading) => reading.timestamp,
            WrappedReadingType::Sheet(sheet) => sheet.timestamp,
            WrappedReadingType::Nested(nested) => nested.timestamp,
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


#[derive(Debug, Clone, Default, Deserialize, Serialize, schemars::JsonSchema)]
pub struct NestedReading {
    pub id: String,
    pub value: NestedReadingValue,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, schemars::JsonSchema)]
pub enum NestedReadingValue {
    Float(f32),
    String(String),
    Int(i32),
    Bool(bool),
    #[default]
    Empty,
}

pub type NestedMap = HashMap<String, Vec<NestedReading>>;


pub fn parse_csv_to_map(csv_content: &str) -> Result<NestedMap, Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(csv_content.as_bytes());

    let mut sections = HashMap::new();

    let h = reader.headers()?;
    let headers = fill_empty_headers(&h);

    // Collect all records into a vector of StringRecords.
    let records: Vec<csv::StringRecord> = reader.records()
        .collect::<Result<Vec<csv::StringRecord>, csv::Error>>()?;

    // Transpose the data (swap rows and columns).
    let num_columns = records.iter().map(|r| r.len()).max().unwrap_or(0);  // Get max columns count
    let mut columns: Vec<Vec<String>> = vec![Vec::new(); num_columns];

    for record in records {
        for (i, field) in record.iter().enumerate() {
            columns[i].push(field.to_string());
        }
    }

    // Extract date time values from mappings
    let mut datetime = chrono::Utc::now();
    columns.iter().for_each(|c| {
        match c[0].as_str() {
            "DOY" => {
                println!("DOY: {}", c[2].parse::<f32>().unwrap().floor() as u32);
                datetime = datetime.with_ordinal(c[2].parse::<f32>().unwrap().floor() as u32).unwrap()
            },
            "time" => {
                let hour = c[2].split(":").nth(0).unwrap().parse::<u32>().unwrap();
                let minute = c[2].split(":").nth(1).unwrap().parse::<u32>().unwrap();
                datetime = datetime.with_hour(hour).unwrap()
                    .with_minute(minute).unwrap()
                    .with_second(0).unwrap()
                    .with_nanosecond(0).unwrap();
            },
            _ => ()
        }
    });


    for (pos, record) in columns.iter().enumerate() {
        let filename = headers.get(pos).cloned().unwrap_or_default(); // Assuming filename is in the first column

        sections.entry(filename.to_string())
            .or_insert_with(Vec::new)
            .push(NestedReading {
                id: record[0].clone(),
                value: parse_value(&record[2]),
                unit: record[1].clone(),
                timestamp: datetime,
            });
    }

    Ok(sections)
}


fn fill_empty_headers(headers: &csv::StringRecord) -> Vec<String> {
    // Create a mutable reference to store the last non-empty header.
    let mut last_non_empty = String::new();
    let mut new_headers = vec![];

    // Iterate over the headers.
    for header in headers.iter() {
        if header.is_empty() {
            // If header is empty, replace it with the last non-empty value.
            new_headers.push(last_non_empty.clone());
        } else {
            new_headers.push(header.to_string());
            // If the header is not empty, update the last non-empty value.
            last_non_empty = header.to_string();
        }
    }

    new_headers
}

fn parse_value(value: &str) -> NestedReadingValue {
    if let Ok(f) = value.parse::<f32>() {
        NestedReadingValue::Float(f)
    } else if let Ok(i) = value.parse::<i32>() {
        NestedReadingValue::Int(i)
    } else if value == "true" || value == "false" {
        NestedReadingValue::Bool(value == "true")
    } else {
        NestedReadingValue::String(value.to_string())
    }
}
