use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::WrappedReadingType;

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DataSendRequest {
    pub site: String,  // Site Id
    pub topic: String, // Sensor Name
    pub data: Value,
    #[serde(default)]
    pub batched: bool,
    #[serde(default)]
    pub file_name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DataSendWrap {
    pub send_req: DataSendRequest,
    pub readings: HashMap<String, Vec<WrappedReadingType>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TransportMessageWrap {
    pub address: streams::Address,
    pub message: Vec<u8>,
}

impl TransportMessageWrap {
    pub fn new(address: streams::Address, message: Vec<u8>) -> Self {
        TransportMessageWrap { address, message }
    }
}

impl AsRef<[u8]> for TransportMessageWrap {
    fn as_ref(&self) -> &[u8] {
        self.message.as_ref()
    }
}
