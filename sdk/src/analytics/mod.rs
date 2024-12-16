use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use crate::analytics::defaults::analytics::DailyAverage;
use crate::models::{InputParameter, NestedReadingValue, Record};

pub mod defaults;

pub async fn get_values_and_inputs(
    params: &[InputParameter],
    records: &[Record],
) -> (HashMap<String, Vec<Record>>, HashMap<String, Vec<(DateTime<Utc>, f64)>>) {
    let mut values: HashMap<String, Vec<Record>> = HashMap::new();
    let mut inputs: HashMap<String, Vec<(DateTime<Utc>, f64)>> = HashMap::new();
    for param in params {
        if let Some(label) = param.label.as_ref() {
            let feedstock = daily_average(records, label, true).await;
            inputs.insert(
                param.id.to_string(),
                feedstock
                    .iter()
                    .map(|record| (record.data_timestamp.and_utc(), record.f64()))
                    .collect(),
            );
            values.insert(param.id.to_string(), feedstock);
        }
    }

    (values, inputs)
}

pub async fn all_daily_averages(data: &[Record]) -> HashMap<NaiveDate, DailyAverage> {
    let mut daily_data: HashMap<NaiveDate, DailyAverage> = HashMap::new();
    for record in data {
        let day: NaiveDate = record.data_timestamp.date();
        if record.f64() >= 0.0 {
            let element = daily_data.entry(day).or_insert(DailyAverage {
                day,
                sensors: HashMap::new(),
            });

            let sensor_avg = element.sensors.entry(record.sensor_id.clone()).or_default();
            sensor_avg.records.push(record);
            // for testing
            sensor_avg.sum += record.f64();
            sensor_avg.avg_val = sensor_avg.sum / sensor_avg.records.len() as f64;
        }
    }

    daily_data
}

pub async fn daily_average(data: &[Record], dataset: &str, _calc: bool) -> Vec<Record> {
    let mut daily_sensor_data: Vec<Record> = Vec::new();
    let daily_data = all_daily_averages(data).await;

    daily_data.iter().for_each(|(_day, daily_avg)| {
        daily_avg.sensors.iter().for_each(|(_id, sensor_avg)| {
            for record in sensor_avg.records.iter() {
                if record.raw.as_ref().and_then(|raw| raw.get(dataset)).is_some() {
                    let mut record = (*record).clone();
                    if let Some(raw) = record.raw.as_ref() {
                        let raw = raw.get(dataset).unwrap();
                        record.sum = NestedReadingValue::Float(raw
                            .clone()
                            .as_str()
                            .map(|s| {
                                // Temp, some values may use commas as decimal separators
                                s.replace(",", "").parse::<f64>().unwrap_or_default()
                            })
                            .unwrap()
                        )
                    }
                    daily_sensor_data.push(record);
                }
            }
        })
    });
    daily_sensor_data.sort_by(|a, b| a.data_timestamp.cmp(&b.data_timestamp));
    daily_sensor_data
}
