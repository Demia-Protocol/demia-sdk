use std::collections::HashMap;

use super::Parameter;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ValueSet {
    pub inputs: HashMap<String, Vec<f64>>,
    pub params: Vec<Parameter>,
    pub title: String,
    pub label: String,
    pub values: Vec<f64>,
    pub timestamps: Vec<String>,
    pub total: f64,
    pub avg: f64,
}

impl ValueSet {
    pub fn new(mut values: Vec<f64>, timestamps: Vec<String>, title: String, label: String) -> ValueSet {
        if values.len() == 1 {
            (1..timestamps.len()).for_each(|_| values.push(values[0]))
        }
        let total = values.iter().sum();
        let avg = total / values.len() as f64;
        ValueSet {
            values,
            timestamps,
            total,
            avg,
            title,
            label,
            ..Default::default()
        }
    }
}
