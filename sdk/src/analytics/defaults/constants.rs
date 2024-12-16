use std::sync::LazyLock;

use crate::models::{InputParameter, StaticParameter};

// Input Parameters
pub static BIOGAS_GENERATED: LazyLock<InputParameter> = LazyLock::new(|| InputParameter {
    id: String::from("BiogasGenerated"),
    unit: String::from("Nm3"),
    text: String::from("Total Biogas Generated"),
    label: Some(String::from("Biogás Generado (Nm3)")),
});

pub static METHANE_CONCENTRATION: LazyLock<InputParameter> = LazyLock::new(|| InputParameter {
    id: String::from("MethaneConcentration"),
    unit: String::from("lbs/f3"),
    text: String::from("Methane Concentration"),
    label: Some(String::from("%CH4 DF")),
});

pub static BIOGAS_GENERATED_NO_FLARE: LazyLock<InputParameter> = LazyLock::new(|| InputParameter {
    id: String::from("BiogasGeneratedNoFlare"),
    unit: String::from("Nm3"),
    text: String::from("Total Biogas Generated without flaring"),
    label: Some(String::from("Biogás Generado sin antorcha (Nm3)")),
});

pub static WASTE_WATER_VOLUME: LazyLock<InputParameter> = LazyLock::new(|| InputParameter {
    id: String::from("WasteWaterVolume"),
    unit: String::from("Tonnes"),
    text: String::from("Liquid Waste Volume"),
    label: Some(String::from("Toneladas ")),
});

pub static FEEDSTOCK_TYPE: LazyLock<InputParameter> = LazyLock::new(|| InputParameter {
    id: String::from("FeedstockType"),
    unit: String::from(""),
    text: String::from("Type of solid waste feedstock"),
    label: Some(String::from("Toneladas ")),
});

// Static Parameters
pub static B_OWW_S: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("B_OWW_S"),
    text: String::from("Methane producing capacity of the wastewater stream 'S'"),
    unit: String::from(""),
    value: 0.21,
});

pub static MCF_ATS: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("MCF_ATS"),
    text: String::from(
        "Methane conversion factor of the anaerobic treatment lagoon, pond, or tank where the waste was treated pre-project",
    ),
    unit: String::from(""),
    value: 0.03,
});

pub static GWP_CH4: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("GWP_CH4"),
    text: String::from("Global Warming Potential of Methane"),
    unit: String::from(""),
    value: 28.0,
});

pub static UNCERTAINTY_FACTOR: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("UNCERTAINTY_FACTOR"),
    text: String::from("Baseline uncertainty factor to account for model uncertainties"),
    unit: String::from(""),
    value: 0.89,
});

pub static OX: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("OX"),
    text: String::from(
        "Oxidation factor (reflecting the amount of methane from SWDS that is oxidized in the soil or other material covering the waste)",
    ),
    unit: String::from(""),
    value: 0.1,
});

pub static F: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("F"),
    text: String::from("Fraction of methane in the SWDS gas (volume fraction)"),
    unit: String::from(""),
    value: 0.5,
});

pub static MCF_Y: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("MCF_Y"),
    text: String::from("Methane correction factor for year y"),
    unit: String::from(""),
    value: 1.0,
});

pub static K: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("K"),
    text: String::from("Decay rate for the residue type j"),
    unit: String::from(""),
    value: 0.1,
});

pub static X: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("X"),
    text: String::from("Years in the time period in which residue is disposed at the landfill"),
    unit: String::from(""),
    value: 1.0,
});

pub static Y: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("Y"),
    text: String::from("Year of the crediting period for which methane emissions are calculated"),
    unit: String::from(""),
    value: 10.0,
});

pub static F_Y: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("F_Y"),
    text: String::from("Model correction factor to account for model uncertainties for year y"),
    unit: String::from(""),
    value: 0.26,
});

pub static EF_ELEC: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("EF_ELEC"),
    text: String::from("Emission factor for electricity"),
    unit: String::from(""),
    value: 0.4137,
});

pub static FUEL_CONSUMPTION: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("FUEL_CONSUMPTION"),
    text: String::from("Fossil fuel consumption for Anaerobic Digestion project"),
    unit: String::from("t CO2e"),
    value: 23.533,
});

pub static AD_EFFICIENCY: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("AD_EFFICIENCY"),
    text: String::from("Efficiency of the Anaerobic Digestion process"),
    unit: String::from(""),
    value: 0.98,
});

pub static CH4_VENT: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("CH4_VENT"),
    text: String::from("Methane vented from the Anaerobic Digestion process"),
    unit: String::from(""),
    value: 0.0,
});

pub static CH4_DENSITY: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("CH4_DENSITY"),
    text: String::from("Density of Methane"),
    unit: String::from("lbs/f3"),
    value: 0.0423,
});

pub static LBS_TO_TONNES: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("LBS_TO_TONNES"),
    text: String::from("Conversion factor for lbs to tonnes"),
    unit: String::from(""),
    value: 0.000454,
});

pub static BDE_DD: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("BDE_DD"),
    text: String::from("Biogas destruction efficiency of the destruction device (enclosed flare)"),
    unit: String::from(""),
    value: 0.995,
});

pub static AMBIENT_GAS_TEMP: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("AMBIENT_GAS_TEMP"),
    text: String::from("Ambient temperature (average) of biogas"),
    unit: String::from("°R"),
    value: 25.0,
});

pub static NORMALIZED_PRESSURE: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("NORMALIZED_PRESSURE"),
    text: String::from("Normalized pressure of biogas"),
    unit: String::from("atm"),
    value: 1.0,
});

pub static CH4_CONVERSION_FACTOR: LazyLock<StaticParameter> = LazyLock::new(|| StaticParameter {
    id: String::from("CH4_CONVERSION_FACTOR"),
    text: String::from("Methane conversion factor of effluent storage pond"),
    unit: String::from(""),
    value: 0.3,
});

pub mod feedstock_types {

    #[derive(Clone, Default, Debug)]
    pub struct FeedstockType {
        pub company: String,
        pub type_of_feedstock: String,
        pub region: String,
        pub __empty: String,
        pub doc: f64,
        pub k: f64,
        pub fie: f64,
        pub f_y: f64,
    }

    pub fn feedstock_types() -> Vec<FeedstockType> {
        vec![
            FeedstockType {
                company: "Aconcagua Food".to_string(),
                type_of_feedstock: "Liquid industrial waste".to_string(),
                region: "Libertador General Bernardo O'Higgins".to_string(),
                __empty: "wws".to_string(),
                doc: 0.09,
                k: 0.06,
                fie: 0.8,
                f_y: 0.0,
            },
            FeedstockType {
                company: "VSPT (Lodo)".to_string(),
                type_of_feedstock: "Biosolids – Stabilized sludge".to_string(),
                region: "Maule".to_string(),
                __empty: "landfill".to_string(),
                doc: 0.2,
                k: 0.185,
                fie: 0.85,
                f_y: 0.0,
            },
            FeedstockType {
                company: "VSPT".to_string(),
                type_of_feedstock: "Agricultural residue".to_string(),
                region: "Maule".to_string(),
                __empty: "landfill".to_string(),
                doc: 0.15,
                k: 0.06,
                fie: 0.8,
                f_y: 0.0,
            },
            FeedstockType {
                company: "Nestle".to_string(),
                type_of_feedstock: "Liquid industrial waste".to_string(),
                region: "Maule".to_string(),
                __empty: "wws".to_string(),
                doc: 0.09,
                k: 0.185,
                fie: 0.85,
                f_y: 0.0,
            },
            FeedstockType {
                company: "CMPC".to_string(),
                type_of_feedstock: "Biosolids – Stabilized sludge".to_string(),
                region: "Metropolitana de Santiago".to_string(),
                __empty: "landfill".to_string(),
                doc: 0.05,
                k: 0.06,
                fie: 0.8,
                f_y: 0.26,
            },
            FeedstockType {
                company: "Ecoser".to_string(),
                type_of_feedstock: "Biosolids – Stabilized sludge".to_string(),
                region: "Maule".to_string(),
                __empty: "landfill".to_string(),
                doc: 0.05,
                k: 0.185,
                fie: 0.85,
                f_y: 0.0,
            },
            FeedstockType {
                company: "Lincoyan RIL Santiago".to_string(),
                type_of_feedstock: "Biosolids – Stabilized sludge".to_string(),
                region: "Maule".to_string(),
                __empty: "landfill".to_string(),
                doc: 0.05,
                k: 0.185,
                fie: 0.85,
                f_y: 0.0,
            },
            FeedstockType {
                company: "Lincoyan Supermercado".to_string(),
                type_of_feedstock: "Biosolids – Stabilized sludge".to_string(),
                region: "Maule".to_string(),
                __empty: "landfill".to_string(),
                doc: 0.05,
                k: 0.185,
                fie: 0.85,
                f_y: 0.0,
            },
            FeedstockType {
                company: "Lincoyan Aceite".to_string(),
                type_of_feedstock: "Liquid industrial waste".to_string(),
                region: "Maule".to_string(),
                __empty: "wws".to_string(),
                doc: 0.09,
                k: 0.185,
                fie: 0.85,
                f_y: 0.0,
            },
            FeedstockType {
                company: "Pesquera Pacific Star".to_string(),
                type_of_feedstock: "Organic residues from fishery and meat industry".to_string(),
                region: "Los Lagos".to_string(),
                __empty: "landfill".to_string(),
                doc: 0.15,
                k: 0.185,
                fie: 0.85,
                f_y: 0.14,
            },
            FeedstockType {
                company: "Agrosuper Lo Miranda".to_string(),
                type_of_feedstock: "Organic residues from fishery and meat industry".to_string(),
                region: "Libertador General Bernardo O'Higgins".to_string(),
                __empty: "landfill".to_string(),
                doc: 0.15,
                k: 0.06,
                fie: 0.8,
                f_y: 0.0,
            },
        ]
    }
}
