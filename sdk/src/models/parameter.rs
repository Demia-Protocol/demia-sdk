#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum Parameter {
    Static(StaticParameter),
    Input(InputParameter),
    Calculation(CalculationParameter),
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct StaticParameter {
    pub id: String,
    pub unit: String,
    pub text: String,
    pub value: f64,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct InputParameter {
    pub id: String,
    pub unit: String,
    pub text: String,
    // For spreadsheets
    pub label: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CalculationParameter {
    pub id: String,
    pub unit: String,
    pub text: String,
}
