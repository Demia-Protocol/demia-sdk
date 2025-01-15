use crate::{
    analytics::{all_daily_averages, defaults::constants::feedstock_types::feedstock_types},
    models::Record,
};

// Constants
const B_OWW_S: f64 = 0.21;
const MCF_ATS: f64 = 0.03;
const GWP_CH4: f64 = 28.0;
const UNCERTAINTY_FACTOR: f64 = 0.89;
const OX: f64 = 0.1;
const F: f64 = 0.5;
const MCF_Y: f64 = 1.0;
const X: f64 = 1.0;
const Y: f64 = 10.0;

// Baseline Emissions Calculation
pub async fn baseline_emissions(calc_data: &[Record]) -> Vec<f64> {
    let feedstock_types = feedstock_types();

    // Calculate methane emissions from organic waste
    let methane_emissions_landfill: Vec<f64> = calc_data
        .iter()
        .filter_map(|record| {
            let feedstock_type = feedstock_types.iter().find(|&ft| ft.company == record.company);
            if let Some(ft) = feedstock_type {
                if ft.__empty == "landfill" {
                    Some(
                        record.f64()
                            * ft.doc
                            * ft.fie
                            * (1.0 - ft.f_y)
                            * GWP_CH4
                            * (1.0 - OX)
                            * F
                            * MCF_Y
                            * f64::exp(-ft.k * Y - X)
                            * (1.0 - f64::exp(-ft.k)),
                    )
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Calculate methane emissions from wastewater
    let methane_emissions_wastewater: Vec<f64> = calc_data
        .iter()
        .filter_map(|record| {
            let feedstock_type = feedstock_types.iter().find(|&ft| ft.company == record.company);
            if let Some(ft) = feedstock_type {
                if ft.__empty == "wws" {
                    Some(record.f64() * B_OWW_S * MCF_ATS * GWP_CH4 * UNCERTAINTY_FACTOR)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let daily_fuel_emissions = 23.533 / 365.0; // tonnes (CO2e /year) / 365 days

    let fossil_fuel_emissions: Vec<f64> = vec![daily_fuel_emissions; all_daily_averages(calc_data).await.len()];

    // Aggregate all baseline emissions
    let baseline_emissions: Vec<f64> = methane_emissions_landfill
        .iter()
        .zip(methane_emissions_wastewater.iter())
        .zip(fossil_fuel_emissions.iter())
        .map(|((&landfill, &wastewater), &fossil_fuel)| landfill + wastewater + fossil_fuel)
        .collect();

    baseline_emissions
}
