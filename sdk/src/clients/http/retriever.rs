use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use streams::Address;
use url::Url;

use super::HttpClient;
use crate::{
    configuration::BaseConfiguration,
    errors::{ApiError, ApiResult},
    utils::{API_TIMEOUT, RETRIEVER_API},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieverApi {
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) http_client: HttpClient,
    pub(crate) url: Url,
}

impl Default for RetrieverApi {
    fn default() -> Self {
        Self {
            url: Url::parse(RETRIEVER_API).unwrap(),
            http_client: HttpClient::new("demia".to_string()),
        }
    }
}

impl RetrieverApi {
    pub fn new<T: TryInto<Url>>(url: T) -> ApiResult<Self>
    where
        T::Error: std::fmt::Display,
    {
        Ok(Self {
            http_client: HttpClient::new("demia".to_string()),
            url: url.try_into().map_err(|e| ApiError::NotFound(e.to_string()))?,
        })
    }

    // TODO: Make this... cleaner we have a token we can use to fetch!
    pub async fn add_retriever_new_user(&self, bearer: &str, config: &BaseConfiguration) -> ApiResult<Value> {
        let path = "new-user";

        let mut url = self.url.clone();
        url.set_path(path);

        let json = serde_json::to_value(config)?;
        let res = self
            .http_client
            .post_json(url, bearer, RetrieverApi::get_timeout(), json)
            .await?;
        res.into_json().await
    }

    pub(crate) fn get_timeout() -> Duration {
        API_TIMEOUT
    }

    pub async fn add_retriever_user_address(
        &self,
        bearer: &str,
        user_id: String,
        address: &Address,
    ) -> ApiResult<Value> {
        let path = "add-address";

        let mut url = self.url.clone();
        url.set_path(path);

        let json = serde_json::json!({
            "id": user_id,
            "address": address.to_string(),
        });

        let res = self
            .http_client
            .post_json(url, bearer, Self::get_timeout(), json)
            .await?;
        print!("code: {}", &res.status());
        res.into_json().await
    }
}
