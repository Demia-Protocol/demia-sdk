use std::sync::{Arc, LazyLock};

use super::constants::*;
use crate::{
    analytics::defaults::analytics::*,
    models::{AnalyticsProfile, AsyncCalculationFunctionWrapper, Calculation, Parameter},
};

pub static MOLINA_CALCULATIONS: LazyLock<Vec<Calculation>> = LazyLock::new(|| {
    vec![
        Calculation {
            id: "WasteWaterTreatment".to_string(),
            text: "Wastewater (liquid industrial waste) of the given stream".to_string(),
            parameters: vec![Parameter::Input(WASTE_WATER_VOLUME.clone())],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation5(&params, &records).await })
            })),
        },
        Calculation {
            id: "SolidWasteEmissions".to_string(),
            text: "Methane emissions from solid waste disposal sites (using first order decay method)".to_string(),
            parameters: vec![Parameter::Input(FEEDSTOCK_TYPE.clone())],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation6(&params, &records).await })
            })),
        },
        Calculation {
            id: "TotalGHGEmissions".to_string(),
            text: "Total GHG emissions from the given stream".to_string(),
            parameters: vec![
                Parameter::Input(BIOGAS_GENERATED.clone()),
                Parameter::Input(BIOGAS_GENERATED_NO_FLARE.clone()),
                Parameter::Input(METHANE_CONCENTRATION.clone()),
            ],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation7(&params, &records).await })
            })),
        },
        Calculation {
            id: "ElectricityConsumed".to_string(),
            text: "Emissions from electricity consumed from the grid".to_string(),
            parameters: vec![],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation8(&params, &records).await })
            })),
        },
        Calculation {
            id: "FossilFuelConsumption".to_string(),
            text: "Fossil Fuel Use for AD Project (daily)".to_string(),
            parameters: vec![],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation9(&params, &records).await })
            })),
        },
        Calculation {
            id: "AnaerobicDigestion".to_string(),
            text: "Anaerobic Digestion Emissions".to_string(),
            parameters: vec![
                Parameter::Input(BIOGAS_GENERATED.clone()),
                Parameter::Input(METHANE_CONCENTRATION.clone()),
            ],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation10(&params, &records).await })
            })),
        },
        Calculation {
            id: "MethaneCollected".to_string(),
            text: "Quantity of Methane Collected and Metered".to_string(),
            parameters: vec![
                Parameter::Input(BIOGAS_GENERATED.clone()),
                Parameter::Input(METHANE_CONCENTRATION.clone()),
            ],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation11(&params, &records).await })
            })),
        },
        Calculation {
            id: "WeightedBiogasAvg".to_string(),
            text: "Weighted average of all destruction devices used (fraction)".to_string(),
            parameters: vec![Parameter::Input(BIOGAS_GENERATED.clone())],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation12(&params, &records).await })
            })),
        },
        Calculation {
            id: "BiogasCollected".to_string(),
            text: "Volume of biogas collected for the given time interval".to_string(),
            parameters: vec![Parameter::Input(BIOGAS_GENERATED.clone())],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation14(&params, &records).await })
            })),
        },
        Calculation {
            id: "EffluentStorageGHGEmissions".to_string(),
            text: "Total GHG Emissions for Effluent Storage for the Reporting Period".to_string(),
            parameters: vec![Parameter::Input(BIOGAS_GENERATED.clone())],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation15(&params, &records).await })
            })),
        },
        Calculation {
            id: "TotalMeteredMethaneDestroyed".to_string(),
            text: "Total Metered Quantity of Methane Captured and Destroyed by the AD Project".to_string(),
            parameters: vec![
                Parameter::Input(BIOGAS_GENERATED.clone()),
                Parameter::Input(METHANE_CONCENTRATION.clone()),
                Parameter::Input(BIOGAS_GENERATED_NO_FLARE.clone()),
            ],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation18(&params, &records).await })
            })),
        },
    ]
});

pub static MOLINA_PROFILE: LazyLock<Arc<AnalyticsProfile>> = LazyLock::new(|| {
    Arc::new(AnalyticsProfile {
        id: String::from("MolinaBiogas"),
        calculation_interval: core::time::Duration::from_secs(60 * 60 * 24), // 1 day
        calculations: MOLINA_CALCULATIONS.clone(),
    })
});

pub fn molina_profile() -> Arc<AnalyticsProfile> {
    MOLINA_PROFILE.clone()
}
