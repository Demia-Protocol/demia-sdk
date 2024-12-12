use std::{collections::HashMap, vec};

use chrono::{DateTime, NaiveDate, Utc};

use super::{defaults::constants::*, get_values_and_inputs};
use crate::{
    errors::{AnalyticsError, AnalyticsResult as Result},
    models::{CalculationParameter, InputParameter, Parameter, Record, ValueSet},
};

pub async fn run_equation(label: &str, params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    match label {
        "WasteWaterTreatment" => equation5(params, records).await,
        "SolidWasteEmissions" => equation6(params, records).await,
        "TotalGHGEmissions" => equation7(params, records).await,
        "ElectricityConsumed" => equation8(params, records).await,
        "FossilFuelConsumption" => equation9(params, records).await,
        "AnaerobicDigestion" => equation10(params, records).await,
        "MethaneCollected" => equation11(params, records).await,
        "WeightedBiogasAvg" => equation12(params, records).await,
        "BiogasCollected" => equation14(params, records).await,
        "EffluentStorageGHGEmissions" => equation15(params, records).await,
        "TotalMeteredMethaneDestroyed" => equation18(params, records).await,
        _ => Err(AnalyticsError::NoCalculationFound(label.to_string())),
    }
}

// Wastewater (liquid industrial waste) of the given stream
pub async fn equation5(params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    let waste_water_records = records
        .iter()
        .filter(|record| &record.residue == "Lodo" || &record.residue == "Lodo Liquido")
        .cloned()
        .collect::<Vec<Record>>();

    // log::info!("Waste water records: {:?}", waste_water_records.len());
    if waste_water_records.is_empty() && !records.is_empty() {
        // log::info!("{:#?}", records);
    }

    let (values, inputs) = get_values_and_inputs(params, &waste_water_records).await;
    if !values.is_empty() {
        // log::info!("Waste water values: {:?}", values.keys().collect::<Vec<_>>());
    }
    let mut results = Vec::new();

    if let Some(waste_water) = values.get("WasteWaterVolume") {
        results = waste_water
            .iter()
            .map(|record| {
                (
                    record.data_timestamp.and_utc(),
                    (record.sum + 1.0) * B_OWW_S.value * MCF_ATS.value * GWP_CH4.value * UNCERTAINTY_FACTOR.value,
                )
            })
            .collect()
    }

    let mut new_params = vec![
        Parameter::Static(B_OWW_S.clone()),
        Parameter::Static(MCF_ATS.clone()),
        Parameter::Static(GWP_CH4.clone()),
        Parameter::Static(UNCERTAINTY_FACTOR.clone()),
    ];

    for param in params {
        new_params.push(Parameter::Input(param.clone()));
    }

    if !results.is_empty() {
        // log::info!("Waste water results: {:?}", results);
    }

    Ok(ValueSet::new(
        inputs,
        results,
        "Waste Water (liquid industrial waste)".to_string(),
        "Tonnes".to_string(),
        new_params,
    ))
}

// Methane emissions from solid waste disposal sites (using first order decay method)
pub async fn equation6(params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    let (values, inputs) = get_values_and_inputs(params, records).await;

    let mut results = Vec::new();

    if let Some(feedstock) = values.get("FeedstockType") {
        results = feedstock
            .iter()
            .map(|record| {
                let types = feedstock_types::feedstock_types();
                let feedstock_type = types.iter().find(|element| element.company.eq(record.company.as_str()));

                let sum = if record.sum > 10000.0 {
                    record.sum / 1000.0
                } else {
                    record.sum
                };
                let cod_ww_s_i = 1.0;

                let value =
                    if feedstock_type.is_some() && feedstock_type.unwrap().type_of_feedstock.as_str() == "Manure" {
                        0.21 * 0.03 * GWP_CH4.value * UNCERTAINTY_FACTOR.value * (sum + cod_ww_s_i)
                    } else {
                        let (doc, fie, f_y) = feedstock_type
                            .map(|feedstock_type| (feedstock_type.doc, feedstock_type.fie, feedstock_type.f_y))
                            .unwrap_or((0.01, 0.85, F_Y.value));
                        fie * (1.0 - f_y)
                            * GWP_CH4.value
                            * (1.0 - OX.value)
                            * (16.0 / 12.0)
                            * F.value
                            * doc
                            * MCF_Y.value
                            * sum
                            * doc
                            * f64::exp(-K.value * Y.value - X.value)
                            * (1.0 - f64::exp(-K.value))
                    };
                (record.data_timestamp.and_utc(), value)
            })
            .collect();
    }

    let mut new_params = vec![
        Parameter::Static(F_Y.clone()),
        Parameter::Static(GWP_CH4.clone()),
        Parameter::Static(OX.clone()),
        Parameter::Static(F.clone()),
        Parameter::Static(MCF_Y.clone()),
        Parameter::Static(K.clone()),
        Parameter::Static(X.clone()),
        Parameter::Static(Y.clone()),
    ];

    for param in params {
        new_params.push(Parameter::Input(param.clone()));
    }

    Ok(ValueSet::new(
        inputs,
        results,
        "Methane emissions from solid waste disposal sites".to_string(),
        "t CO2e".to_string(),
        new_params,
    ))
}

// Total Emissions for the Reporting Period
pub async fn equation7(params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    let elec_use = equation8(params, records).await?.data;
    let fossil_fuel = equation9(params, records).await?.data;
    let ann_dig = equation10(params, records).await?.data;
    let bde = equation12(params, records).await?;
    let effluent_storage = equation15(params, records).await?.data;

    let (values, inputs) = get_values_and_inputs(params, records).await;

    let mut flare_e = Vec::new();
    if let Some(daily_biogas) = values.get("BiogasGenerated") {
        if let Some(daily_biogas_no_flare) = values.get("BiogasGeneratedNoFlare") {
            flare_e = daily_biogas
                .iter()
                .enumerate()
                .map(|(i, record)| {
                    (
                        record.data_timestamp.and_utc(),
                        record.sum - daily_biogas_no_flare[i].sum,
                    )
                })
                .collect()
        }
    }

    let result: Vec<(DateTime<Utc>, f64)> = if !bde.data.is_empty() {
        bde.data
            .iter()
            // For Equations 8 and 9 there is only one value in the array
            .map(|(timestamp, record)| {
                let ann_dig_val = ann_dig
                    .iter()
                    .find(|(t, _)| timestamp.eq(t))
                    .map(|(_, v)| v)
                    .cloned()
                    .unwrap_or_default();
                let effl_storage_val = effluent_storage
                    .iter()
                    .find(|(t, _)| timestamp.eq(t))
                    .map(|(_, v)| v)
                    .cloned()
                    .unwrap_or_default();
                let flare_e_val = flare_e
                    .iter()
                    .find(|(t, _)| timestamp.eq(t))
                    .map(|(_, v)| v)
                    .cloned()
                    .unwrap_or_default();
                (
                    *timestamp,
                    record + elec_use[0].1 + fossil_fuel[0].1 + ann_dig_val + flare_e_val + effl_storage_val,
                )
            })
            .collect()
    } else {
        Vec::new()
    };
    let mut new_params = vec![
        Parameter::Calculation(CalculationParameter {
            id: "ElectricityConsumed".to_string(),
            text: "Emissions from electricity consumed from the grid".to_string(),
            unit: "t CO2e".to_string(),
        }),
        Parameter::Calculation(CalculationParameter {
            id: "FossilFuelConsumption".to_string(),
            text: "Fossil Fuel Use for AD Project (daily)".to_string(),
            unit: "t CO2e".to_string(),
        }),
        Parameter::Calculation(CalculationParameter {
            id: "AnaerobicDigestion".to_string(),
            text: "Anaerobic Digestion Emissions".to_string(),
            unit: "t CO2e".to_string(),
        }),
        Parameter::Calculation(CalculationParameter {
            id: "EffluentStorageGHGEmissions".to_string(),
            text: "Total GHG Emissions for Effluent Storage for the Reporting Period".to_string(),
            unit: "t CO2e".to_string(),
        }),
        Parameter::Calculation(CalculationParameter {
            id: "WeightedBiogasAvg".to_string(),
            text: "Weighted average of all destruction devices used (flaring)".to_string(),
            unit: "t CO2e".to_string(),
        }),
    ];
    for param in params {
        new_params.push(Parameter::Input(param.clone()));
    }

    Ok(ValueSet::new(
        inputs,
        result,
        "Total GHG emissions from the given stream".to_string(),
        "t CO2e".to_string(),
        new_params,
    ))
}

// Electricity Consumption (No active monitoring)
pub async fn equation8(_params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    let params = vec![BIOGAS_GENERATED.clone()];
    let (values, inputs) = get_values_and_inputs(&params, records).await;
    let mut results = Vec::new();

    if let Some(daily_biogas) = values.get("BiogasGenerated") {
        results = daily_biogas.iter().map(|r| (r.data_timestamp.and_utc(), 0.0)).collect()
    }

    let params = vec![Parameter::Static(EF_ELEC.clone())];

    Ok(ValueSet::new(
        inputs,
        results,
        "Emissions from electricity consumed from the grid".to_string(),
        "t CO2e".to_string(),
        params,
    ))
}

// Fossil Fuel Use for AD Project (daily)
pub async fn equation9(_params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    // Calculation is conducted on daily intervals
    let daily_result = FUEL_CONSUMPTION.value / 365.0;
    let params = vec![BIOGAS_GENERATED.clone()];
    let (values, inputs) = get_values_and_inputs(&params, records).await;
    let mut results = Vec::new();

    if let Some(daily_biogas) = values.get("BiogasGenerated") {
        results = daily_biogas
            .iter()
            .map(|r| (r.data_timestamp.and_utc(), daily_result * 2.8316846592)) // tonnes to m^3
            .collect()
    }

    let params = vec![Parameter::Static(FUEL_CONSUMPTION.clone())];

    Ok(ValueSet::new(
        inputs,
        results,
        "Daily Fossil Fuel Use for Anaerobic Digestion".to_string(),
        "m3".to_string(),
        params,
    ))
}

// Anaerobic Digestion Emissions
pub async fn equation10(params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    let bde = equation12(params, records).await?;
    let ch4 = equation11(params, records).await?;

    let (values, inputs) = get_values_and_inputs(params, records).await;
    let mut results = Vec::new();

    if let Some(biogas) = values.get("BiogasGenerated") {
        if !bde.data.is_empty() && !ch4.data.is_empty() && !biogas.is_empty() {
            results = bde
                .data
                .iter()
                .map(|(i, r)| {
                    let ch4_data = ch4.data.iter().find(|(t, _)| t.eq(i)).map(|(_, v)| *v).unwrap_or(0.0);
                    let n = if *r == 0.0 || ch4_data == 0.0 { 1.0 } else { 2.0 };

                    (
                        *i,
                        GWP_CH4.value * (ch4_data * (n / (AD_EFFICIENCY.value - r) + CH4_VENT.value)),
                    )
                })
                .collect();
        }
    }

    let mut new_params = vec![
        Parameter::Static(GWP_CH4.clone()),
        Parameter::Static(AD_EFFICIENCY.clone()),
        Parameter::Static(CH4_VENT.clone()),
        Parameter::Calculation(CalculationParameter {
            id: "WeightedBiogasAvg".to_string(),
            text: "Weighted average of all destruction devices used (flaring)".to_string(),
            unit: "t CO2e".to_string(),
        }),
        Parameter::Calculation(CalculationParameter {
            id: "MethaneCollected".to_string(),
            text: "Quantity of Methane Collected and Metered".to_string(),
            unit: "t CH4".to_string(),
        }),
    ];

    for param in params {
        new_params.push(Parameter::Input(param.clone()));
    }

    Ok(ValueSet::new(
        inputs,
        results,
        "Anaerobic Digestion Emissions".to_string(),
        "t CO2e".to_string(),
        new_params,
    ))
}

// Quantity of Methane Collected and Metered
pub async fn equation11(params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    let (values, inputs) = get_values_and_inputs(params, records).await;
    let mut results = Vec::new();

    if let Some(biogas) = values.get("BiogasGenerated") {
        if let Some(methane_concentration) = values.get("MethaneConcentration") {
            if !biogas.is_empty() && !methane_concentration.is_empty() {
                results = biogas
                    .iter()
                    .enumerate()
                    .map(|(i, record)| {
                        (
                            record.data_timestamp.and_utc(),
                            record.sum * methane_concentration[i].sum * CH4_DENSITY.value * LBS_TO_TONNES.value,
                        )
                    })
                    .collect();
            }
        }
    }

    let mut new_params = vec![
        Parameter::Static(CH4_DENSITY.clone()),
        Parameter::Static(LBS_TO_TONNES.clone()),
    ];

    for param in params {
        new_params.push(Parameter::Input(param.clone()));
    }

    Ok(ValueSet::new(
        inputs,
        results,
        "Quantity of Methane Collected and Metered".to_string(),
        "t CH4".to_string(),
        new_params,
    ))
}

// Weighted average of all destruction devices used (fraction)
pub async fn equation12(params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    let (values, inputs) = get_values_and_inputs(params, records).await;
    let mut results = Vec::new();

    if let Some(biogas) = values.get("BiogasGenerated") {
        results = biogas
            .iter()
            .map(|r| (r.data_timestamp.and_utc(), r.sum * BDE_DD.value))
            .collect();
    }

    let mut new_params = vec![Parameter::Static(BDE_DD.clone())];
    for param in params {
        new_params.push(Parameter::Input(param.clone()));
    }

    Ok(ValueSet::new(
        inputs,
        results,
        "Weighted average of all destruction devices used".to_string(),
        "Nm3".to_string(),
        vec![Parameter::Static(BDE_DD.clone())],
    ))
}

// Volume of biogas collected for the given time interval
pub async fn equation14(params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    let (values, inputs) = get_values_and_inputs(params, records).await;
    let mut results = Vec::new();

    if let Some(biogas) = values.get("BiogasGenerated") {
        results = biogas
            .iter()
            .map(|r| {
                (
                    r.data_timestamp.and_utc(),
                    r.sum * (520.0 / AMBIENT_GAS_TEMP.value) * NORMALIZED_PRESSURE.value,
                )
            })
            .collect();
    }

    let mut new_params = vec![
        Parameter::Static(AMBIENT_GAS_TEMP.clone()),
        Parameter::Static(NORMALIZED_PRESSURE.clone()),
    ];

    for param in params {
        new_params.push(Parameter::Input(param.clone()));
    }

    Ok(ValueSet::new(
        inputs,
        results,
        "Volume of biogas collected for the given time interval".to_string(),
        "Nm3".to_string(),
        new_params,
    ))
}

// Total GHG Emissions for Effluent Storage for the Reporting Period
pub async fn equation15(params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    let (values, inputs) = get_values_and_inputs(params, records).await;
    let mut results = Vec::new();

    if let Some(biogas) = values.get("BiogasGenerated") {
        results = biogas
            .iter()
            .map(|r| {
                (
                    r.data_timestamp.and_utc(),
                    r.sum * CH4_CONVERSION_FACTOR.value * GWP_CH4.value * B_OWW_S.value,
                )
            })
            .collect();
    }

    let mut new_params = vec![
        Parameter::Static(CH4_CONVERSION_FACTOR.clone()),
        Parameter::Static(GWP_CH4.clone()),
        Parameter::Static(B_OWW_S.clone()),
    ];

    for param in params {
        new_params.push(Parameter::Input(param.clone()));
    }

    Ok(ValueSet::new(
        inputs,
        results,
        "Total GHG Emissions for Effluent Storage for the Reporting Period".to_string(),
        "t CO2e".to_string(),
        new_params,
    ))
}

// Total Metered Quantity of Methane Captured and Destroyed by the AD Project
pub async fn equation18(params: &[InputParameter], records: &[Record]) -> Result<ValueSet> {
    let (values, inputs) = get_values_and_inputs(params, records).await;
    let mut results = Vec::new();

    if let Some(biogas) = values.get("BiogasGenerated") {
        if let Some(methane_concentration) = values.get("MethaneConcentration") {
            if let Some(biogas_no_flare) = values.get("BiogasGeneratedNoFlare") {
                results = biogas
                    .iter()
                    .enumerate()
                    .map(|(i, record)| {
                        let n = if biogas_no_flare[i].sum == 0.0 || record.sum == 0.0 {
                            1.0
                        } else {
                            2.0
                        };

                        (
                            record.data_timestamp.and_utc(),
                            ((record.sum + biogas_no_flare[i].sum) / n) * methane_concentration[i].sum * GWP_CH4.value,
                        )
                    })
                    .collect();
            }
        }
    }

    let mut new_params = vec![Parameter::Static(GWP_CH4.clone())];

    for param in params {
        new_params.push(Parameter::Input(param.clone()));
    }

    Ok(ValueSet::new(
        inputs,
        results,
        "Total Metered Quantity of Methane Captured and Destroyed by the Anaerobic Digester Project".to_string(),
        "t CO2e".to_string(),
        new_params,
    ))
}

#[derive(Debug, Default, Clone)]
pub struct DailyAverage<'a> {
    pub day: NaiveDate,
    pub sensors: HashMap<String, SensorAverage<'a>>,
}

#[derive(Debug, Default, Clone)]
pub struct SensorAverage<'a> {
    pub sum: f64,
    pub avg_val: f64,
    pub records: Vec<&'a Record>,
}

pub async fn daily_average(data: &[Record], dataset: &str, _calc: bool) -> Vec<Record> {
    let mut daily_sensor_data: Vec<Record> = Vec::new();
    let daily_data = all_daily_averages(data).await;

    ////info!("Daily Data: {:?}", daily_data);
    daily_data.iter().for_each(|(_day, daily_avg)| {
        daily_avg.sensors.iter().for_each(|(_id, sensor_avg)| {
            for record in sensor_avg.records.iter() {
                ////info!("Raw? {}", record.raw.as_ref().and_then(|raw| raw.get(dataset)).is_some());
                if dataset.eq("Toneladas ") || record.raw.as_ref().and_then(|raw| raw.get(dataset)).is_some() {
                    daily_sensor_data.push((*record).clone());
                }
            }
        })
    });
    ////info!("Daily sensor Data: {}", daily_sensor_data.len());

    daily_sensor_data.sort_by(|a, b| a.data_timestamp.cmp(&b.data_timestamp));
    daily_sensor_data
}

pub async fn all_daily_averages(data: &[Record]) -> HashMap<NaiveDate, DailyAverage> {
    let mut daily_data: HashMap<NaiveDate, DailyAverage> = HashMap::new();
    for record in data {
        let day: NaiveDate = record.data_timestamp.date();
        if record.sum >= 0.0 {
            let element = daily_data.entry(day).or_insert(DailyAverage {
                day,
                sensors: HashMap::new(),
            });

            let sensor_avg = element.sensors.entry(record.sensor_id.clone()).or_default();
            sensor_avg.records.push(record);
            // for testing
            sensor_avg.sum += record.sum;
            sensor_avg.avg_val = sensor_avg.sum / sensor_avg.records.len() as f64;
        }
    }

    daily_data
}

// Example usage:
// let (daily_data, period_timestamps) = daily_average(your_data, "Toneladas ", true).await;
