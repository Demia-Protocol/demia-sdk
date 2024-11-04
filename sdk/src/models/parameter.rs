#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(bound(deserialize = "'de: 'static"))]
pub enum Parameter {
    Static(StaticParameter),
    Input(InputParameter),
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct StaticParameter {
    pub id: &'static str,
    pub unit: &'static str,
    pub text: &'static str,
    pub value: f64,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct InputParameter {
    pub id: &'static str,
    pub unit: &'static str,
    pub text: &'static str,
    // For spreadsheets
    pub label: Option<&'static str>,
}
