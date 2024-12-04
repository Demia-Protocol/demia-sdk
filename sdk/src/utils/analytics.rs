use std::{collections::HashMap, vec};

use chrono::{DateTime, NaiveDate, Utc};

use crate::{
    models::{Record, ValueSet},
    utils::feedstock_types::feedstock_types,
};

// Constants
const B_OWW_S: f64 = 0.21;
const MCF_ATS: f64 = 0.03;
const GWP_CH4: f64 = 28.0;
const UNCERTAINTY_FACTOR: f64 = 0.89;
const OX: f64 = 0.1;
const F: f64 = 0.5;
const MCF_Y: f64 = 1.0;
const K: f64 = 0.1;
const X: f64 = 1.0;
const Y: f64 = 10.0;
const F_Y: f64 = 0.26;

// Wastewater (liquid industrial waste) of the given stream
pub async fn equation5(feedstock_data: &[Record], cod_lab_sheet: f64) -> ValueSet {
    let q_ww_s_i = feedstock_data
        .iter()
        .filter(|record| &record.residue == "Lodo" || &record.residue == "Lodo Liquido")
        .cloned()
        .collect::<Vec<Record>>();

    let daily_feedstock = daily_average(&q_ww_s_i, "Toneladas ", true).await;

    let result: Vec<(DateTime<Utc>, f64)> = daily_feedstock
        .iter()
        .map(|record| {
            (
                record.data_timestamp.and_utc(),
                (record.sum + cod_lab_sheet) * B_OWW_S * MCF_ATS * GWP_CH4 * UNCERTAINTY_FACTOR,
            )
        })
        .collect();

    ValueSet::new(
        HashMap::new(),
        result,
        "Waste Water (liquid industrial waste)".to_string(),
        "Tonnes".to_string(),
        vec![],
    )
}

// Methane emissions from solid waste disposal sites (using first order decay method)
pub async fn equation6(feedstock_data: &[Record]) -> ValueSet {
    let daily_feedstock = daily_average(feedstock_data, "Toneladas ", true).await;

    let result: Vec<(DateTime<Utc>, f64)> = daily_feedstock
        .iter()
        .map(|record| {
            let cod_ww_s_i = 1.0;
            let feedstock_types = feedstock_types();
            let feedstock_type = feedstock_types
                .iter()
                .find(|element| element.company.eq(record.company.as_str()));

            ////info!("Feedstop Type: {:?}", feedstock_type);
            ////info!("Sum: {}", record.sum);
            let sum = if record.sum > 10000.0 {
                record.sum / 1000.0
            } else {
                record.sum
            };

            let value = if feedstock_type.is_some() && feedstock_type.unwrap().type_of_feedstock.as_str() == "Manure" {
                0.21 * 0.03 * GWP_CH4 * UNCERTAINTY_FACTOR * (sum + cod_ww_s_i)
            } else {
                let (doc, fie, f_y) = feedstock_type
                    .map(|feedstock_type| (feedstock_type.doc, feedstock_type.fie, feedstock_type.f_y))
                    .unwrap_or((0.01, 0.85, F_Y));
                fie * (1.0 - f_y)
                    * GWP_CH4
                    * (1.0 - OX)
                    * (16.0 / 12.0)
                    * F
                    * doc
                    * MCF_Y
                    * sum
                    * doc
                    * f64::exp(-K * Y - X)
                    * (1.0 - f64::exp(-K))
            };
            (record.data_timestamp.and_utc(), value)
        })
        .collect();

    ValueSet::new(
        HashMap::new(),
        result,
        "Methane emissions from solid waste disposal sites".to_string(),
        "t C02e".to_string(),
        vec![],
    )
}

// Emissions for the Reporting Period
pub async fn equation7(
    eq8: Vec<(DateTime<Utc>, f64)>,
    eq9: Vec<(DateTime<Utc>, f64)>,
    eq10: Vec<(DateTime<Utc>, f64)>,
    calc_data: &[Record],
    eq12: Vec<(DateTime<Utc>, f64)>,
    eq15: Vec<(DateTime<Utc>, f64)>,
) -> ValueSet {
    let daily_biogas = daily_average(calc_data, "Biogás Generado (Nm3)", true).await;
    let daily_biogas_no_flare = daily_average(calc_data, "Biogás Generado sin antorcha (Nm3)", true).await;
    let flare_e: Vec<(DateTime<Utc>, f64)> = if !daily_biogas.is_empty() && !daily_biogas_no_flare.is_empty() {
        daily_biogas
            .iter()
            .enumerate()
            .map(|(i, record)| {
                (
                    record.data_timestamp.and_utc(),
                    record.sum - daily_biogas_no_flare[i].sum,
                )
            })
            .collect()
    } else {
        Vec::new()
    };

    let result: Vec<(DateTime<Utc>, f64)> = if !eq12.is_empty() {
        eq12.iter()
            .enumerate()
            .map(|(i, (timestamp, record))| {
                (
                    *timestamp,
                    record + eq8[i].1 + eq9[i].1 + eq10[i].1 + flare_e[i].1 + eq15[i].1,
                )
            })
            .collect()
    } else {
        Vec::new()
    };

    ValueSet::new(
        HashMap::new(),
        result,
        "Emissions for the Reporting Period".to_string(),
        "t C02e".to_string(),
        vec![],
    )
}

// Electricity Generation and Transmission
pub async fn equation8() -> f64 {
    let ef_elec = 0.4137;
    let el_pr = 0.0;

    (el_pr * ef_elec) / 1000.0
}

// Fossil Fuel Use for AD Project (yearly)
pub async fn equation9() -> f64 {
    let result = 23.533; // tonnes CO2e /year - given calculated value
    let daily_result = result / 365.0;
    daily_result * 2.8316846592 // tonnes to m^3
}

// Anaerobic Digestor
pub async fn equation10(
    bde: Vec<(DateTime<Utc>, f64)>,
    ch4: Vec<(DateTime<Utc>, f64)>,
    calc_data: &[Record],
) -> ValueSet {
    let ad = 0.98;
    let ch4_vent = 0.0;

    let daily_f_mo = daily_average(calc_data, "Biogás Generado (Nm3)", true).await;

    let result: Vec<(DateTime<Utc>, f64)> = if !bde.is_empty() && !ch4.is_empty() && !daily_f_mo.is_empty() {
        bde.iter()
            .enumerate()
            .map(|(i, (date, record))| {
                let n = if *record == 0.0 || ch4[i].1 == 0.0 { 1.0 } else { 2.0 };
                (*date, GWP_CH4 * (ch4[i].1 * (n / (ad - record) + ch4_vent)))
            })
            .collect()
    } else {
        Vec::new()
    };

    ValueSet::new(
        HashMap::new(),
        result,
        "Anaerobic Digestor".to_string(),
        "t C02e".to_string(),
        vec![],
    )
}

// Quantity of Methane Collected and Metered
pub async fn equation11(calc_data: &[Record]) -> ValueSet {
    let methane_density = 0.0423;
    let conversion_factor = 0.000454;

    let daily_f_mo = daily_average(calc_data, "Biogás Generado (Nm3)", true).await;
    let daily_ch4_conc_mo = daily_average(calc_data, "%CH4 DF", true).await;

    let result: Vec<(DateTime<Utc>, f64)> = if !daily_f_mo.is_empty() && !daily_ch4_conc_mo.is_empty() {
        daily_f_mo
            .iter()
            .enumerate()
            .map(|(i, record)| {
                (
                    record.data_timestamp.and_utc(),
                    record.sum * daily_ch4_conc_mo[i].sum * methane_density * conversion_factor,
                )
            })
            .collect()
    } else {
        Vec::new()
    };

    ValueSet::new(
        HashMap::new(),
        result,
        "Quantity of Methane Collected and Metered".to_string(),
        "t CH4".to_string(),
        vec![],
    )
}

// Weighted average of all destruction devices used (fraction)
pub async fn equation12(calc_data: &[Record]) -> ValueSet {
    let bde_dd = 0.995;

    let daily_calc_data = daily_average(calc_data, "Biogás Generado (Nm3)", true).await;

    let result: Vec<(DateTime<Utc>, f64)> = if !daily_calc_data.is_empty() {
        daily_calc_data
            .iter()
            .map(|record| (record.data_timestamp.and_utc(), record.sum * bde_dd))
            .collect()
    } else {
        Vec::new()
    };

    ValueSet::new(
        HashMap::new(),
        result,
        "Weighted Biogas average of all destruction devices used".to_string(),
        "Nm3".to_string(),
        vec![],
    )
}

// Volume of biogas collected for the given time interval
pub async fn equation14(calc_data: &[Record]) -> ValueSet {
    let t = 32.0;
    let p = 1.0;

    let daily_calc_data = daily_average(calc_data, "Biogás Generado (Nm3)", true).await;

    let result: Vec<(DateTime<Utc>, f64)> = if !daily_calc_data.is_empty() {
        daily_calc_data
            .iter()
            .map(|record| (record.data_timestamp.and_utc(), record.sum * (520.0 / t) * p))
            .collect()
    } else {
        Vec::new()
    };

    ValueSet::new(
        HashMap::new(),
        result,
        "Volume of biogas collected for the given time interval".to_string(),
        "Nm3".to_string(),
        vec![],
    )
}

// Total GHG Emissions for Effluent Storage for the Reporting Period
pub async fn equation15(calc_data: &[Record]) -> ValueSet {
    let b_0_ef = 0.21;
    let methane_conversion_factor = 0.3;
    let gwp_ch4 = 28.0;

    let daily_calc_data = daily_average(calc_data, "Biogás Generado (Nm3)", true).await;

    let result: Vec<(DateTime<Utc>, f64)> = if !daily_calc_data.is_empty() {
        daily_calc_data
            .iter()
            .map(|record| {
                (
                    record.data_timestamp.and_utc(),
                    b_0_ef * methane_conversion_factor * gwp_ch4 * record.sum,
                )
            })
            .collect()
    } else {
        Vec::new()
    };

    ValueSet::new(
        HashMap::new(),
        result,
        "Total GHG Emissions for Effluent Storage for the Reporting Period".to_string(),
        "t C02e".to_string(),
        vec![],
    )
}

// Total Metered Quantity of Methane Captured and Destroyed by the AD Project
pub async fn equation18(calc_data: &[Record]) -> ValueSet {
    // Calculate daily averages
    let daily_biogas = daily_average(calc_data, "Biogás Generado (Nm3)", true).await;
    let daily_biogas_no_flare = daily_average(calc_data, "Biogás Generado sin antorcha (Nm3)", true).await;
    let daily_ch4_meter = daily_average(calc_data, "%CH4 DF", true).await;

    let mut result: Vec<(DateTime<Utc>, f64)> = Vec::new();

    if !daily_biogas.is_empty() {
        for i in 0..daily_biogas.len() {
            let n = if daily_biogas[i].sum == 0.0 || daily_biogas_no_flare[i].sum == 0.0 {
                1.0
            } else {
                2.0
            };

            let value = ((daily_biogas[i].sum + daily_biogas_no_flare[i].sum) / n) * daily_ch4_meter[i].sum * GWP_CH4;

            if !result
                .iter()
                .any(|(timestamp, _)| timestamp == &daily_biogas[i].data_timestamp.and_utc())
            {
                result.push((daily_biogas[i].data_timestamp.and_utc(), value));
            }
        }
    }

    ValueSet::new(
        HashMap::new(),
        result,
        "Total Metered Quantity of Methane Captured and Destroyed by Anaerobic Digestion".to_string(),
        "t CH4".to_string(),
        vec![],
    )
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
