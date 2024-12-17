use std::sync::{Arc, LazyLock};

use super::constants::*;
use crate::{
    analytics::analytics::*,
    models::{AnalyticsProfile, AsyncCalculationFunctionWrapper, Calculation, Parameter},
};

pub static MOLINA_CALCULATIONS: LazyLock<Vec<Calculation>> = LazyLock::new(|| {
    vec![
        Calculation {
            id: "WasteWaterTreatment".to_string(),
            text: "Wastewater (liquid industrial waste) of the given stream".to_string(),
            equation: "BE_{CH_{4},WW,S} = B_{O,WW,S} * MCF_{AT,S} * GWP_{CH_{4}} * 0.89 * \\sum (Q_{ww,s,i} * COD_{ww,s,i})".to_string(),
            parameters: vec![Parameter::Input(WASTE_WATER_VOLUME.clone())],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation5(&params, &records).await })
            })),
        },
        Calculation {
            id: "SolidWasteEmissions".to_string(),
            text: "Methane emissions from solid waste disposal sites (using first order decay method)".to_string(),
            equation: "E_{SSRB9} = \\phi_{y} * (1 - f_{y}) * GWP_{CH_{4}} * (1 - OX) * \\frac{16}{12} * F * DOC_{f,y} * MCF_{y} * \\sum_{x=1}^{y} \\sum_{j} (W_{j,x} * DOC_{j} * e^{-k_{j} * (y - x)} * (1 - e^{-k_{j}}))".to_string(),
            parameters: vec![Parameter::Input(FEEDSTOCK_TYPE.clone())],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation6(&params, &records).await })
            })),
        },
        Calculation {
            id: "TotalGHGEmissions".to_string(),
            text: "Total GHG emissions from the given stream".to_string(),
            equation: "E_{Project} = P5_{Elec\\ Gen} + P6_{FF\\ Use} + P8_{Waste\\ Proc} + P9_{Anaerobic\\ Dig} + P10_{Flare} + P11_{Pipe\\ Upgrading} + P12_{Pipe/Vehicle} + P13_{Boiler} + P14_{Eng/Turb}".to_string(),
            parameters: vec![
                Parameter::Input(BIOGAS_GENERATED.clone()),
                Parameter::Input(BIOGAS_GENERATED_NO_FLARE.clone()),
            ],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation7(&params, &records).await })
            })),
        },
        Calculation {
            id: "ElectricityConsumed".to_string(),
            text: "Emissions from electricity consumed from the grid".to_string(),
            parameters: vec![],
            equation: "E_{SSRP5} = \\sum_{i} (EL_{PR} * EF_{Elec}) / 1000".to_string(),
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation8(&params, &records).await })
            })),
        },
        Calculation {
            id: "FossilFuelConsumption".to_string(),
            text: "Fossil Fuel Use for AD Project (daily)".to_string(),
            equation: "E_{SSRP6} = \\biggl[ \\sum_{i} (FF_{PR,i} * EF_{FF,i,CO_{2}}) + (FF_{PR,i} * EF_{FF,i,N_{2}O}) * GWP_{N_{2}O} + (FF_{PR,i} * EF_{FF,i,CH_{4}}) * GWP_{CH_{4}} \\biggr] / 1000".to_string(),
            parameters: vec![],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation9(&params, &records).await })
            })),
        },
        Calculation {
            id: "AnaerobicDigestion".to_string(),
            text: "Anaerobic Digestion Emissions".to_string(),
            equation: "E_{SSRP9} = GWP_{CH_{4}} * \\sum_{i} (CH_{4,meter,mo} * (\\frac{1}{AD} - BDE_{mo,weighted}) + CH_{4,vent,mo})".to_string(),
            parameters: vec![],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation10(&params, &records).await })
            })),
        },
        Calculation {
            id: "MethaneCollected".to_string(),
            text: "Quantity of Methane Collected and Metered".to_string(),
            equation: "CH_{4,meter,mo} = F_{mo} * CH_{4,conc,mo} * 0.04230 * 0.000454".to_string(),
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
            equation: "BDE_{mo,weighted} = \\frac{\\sum_{DD} (BDE_{DD} * F_{mo,DD})}{F_{mo}}".to_string(),
            parameters: vec![Parameter::Input(BIOGAS_GENERATED.clone())],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation12(&params, &records).await })
            })),
        },
        Calculation {
            id: "BiogasCollected".to_string(),
            text: "Volume of biogas collected for the given time interval".to_string(),
            equation: "F_{scf} = F_{unadjusted} * \\frac{520}{T} * \\frac{P}{1}".to_string(),
            parameters: vec![Parameter::Input(BIOGAS_GENERATED.clone())],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation14(&params, &records).await })
            })),
        },
        Calculation {
            id: "EffluentStorageGHGEmissions".to_string(),
            text: "Total GHG Emissions for Effluent Storage for the Reporting Period".to_string(),
            equation: "E_{SSRP16} = B_{0,EF} * 0.3 * GWP_{CH_{4}} * 1.12 * \\sum_{i} (Q_{EF,i} * COD_{EF,i})".to_string(),
            parameters: vec![Parameter::Input(BIOGAS_GENERATED.clone())],
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|params, records| {
                Box::pin(async move { equation15(&params, &records).await })
            })),
        },
        Calculation {
            id: "TotalMeteredMethaneDestroyed".to_string(),
            text: "Total Metered Quantity of Methane Captured and Destroyed by the AD Project".to_string(),
            equation: "E_{CH_{4},destroyed} = \\sum_{i} (CH_{4,meter,i} * BDE_{i}) * GWP_{CH_{4}}".to_string(),
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
