use std::collections::HashMap;

use super::Parameter;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ValueSet {
    #[serde(default)]
    pub inputs: HashMap<String, Vec<f64>>,
    #[serde(default)]
    pub params: Vec<Parameter>,
    pub title: String,
    pub label: String,
    pub values: Vec<f64>,
    pub timestamps: Vec<String>,
    pub total: f64,
    pub avg: f64,
}

impl ValueSet {
    pub fn new(
        inputs: HashMap<String, Vec<f64>>,
        mut values: Vec<f64>,
        timestamps: Vec<String>,
        title: String,
        label: String,
        params: Vec<Parameter>,
    ) -> ValueSet {
        if values.len() == 1 {
            (1..timestamps.len()).for_each(|_| values.push(values[0]))
        }
        let total = values.iter().sum();
        let avg = total / values.len() as f64;
        ValueSet {
            inputs,
            values,
            timestamps,
            total,
            avg,
            title,
            label,
            params,
        }
    }
}
