pub mod analytics;
pub use analytics::*;

pub mod constants;
use base64::Engine;
pub use constants::*;
use iota_sdk::crypto::signatures::ed25519::SecretKey;
use log::info;
use serde::Deserialize;

pub fn new_stronghold_key() -> String {
    let key = SecretKey::generate().expect("Shouldn't be a problem to generate a new key");
    let str = base64::engine::general_purpose::STANDARD.encode(key.as_slice());
    info!("new stronghold key: {}", str);
    str
}

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

    pub fn get_feedstock_doc() -> Vec<FeedstockType> {
        feedstock_types().to_vec()
    }
}

pub fn make_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
