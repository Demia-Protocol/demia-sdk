use std::{fmt::Debug, time::Duration};

use iota_sdk::types::block::address::Address;
use serde::{Serialize, de::DeserializeOwned};
use url::Url;

use crate::{
    clients::{HttpClient, query_tuples_to_query_string},
    errors::{ApiError, ApiResult},
};

pub struct ApiClient {
    pub(crate) cloud_api_url: Url,
    pub(crate) retriever_url: Url,
    pub(crate) http_client: HttpClient,
}

impl Default for ApiClient {
    fn default() -> Self {
        // Ensure its the same port as you set in the .env from the local_api folder
        Self {
            cloud_api_url: Url::parse("http://localhost:1111").unwrap(),
            retriever_url: Url::parse("http://localhost:9000").unwrap(),
            http_client: HttpClient::new("demia".to_string()),
        }
    }
}

impl ApiClient {
    pub fn new<T: TryInto<Url>>(cloud_api_url: T, retriever_url: T) -> ApiResult<Self>
    where
        T::Error: std::fmt::Display,
    {
        Ok(Self {
            cloud_api_url: cloud_api_url
                .try_into()
                .map_err(|e| ApiError::NotFound(e.to_string()))?,
            retriever_url: retriever_url
                .try_into()
                .map_err(|e| ApiError::NotFound(e.to_string()))?,
            http_client: HttpClient::new("demia".to_string()),
        })
    }

    pub(crate) fn get_timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    pub async fn request_balance(&self, bearer: &str, address: &Address) -> ApiResult<String> {
        let addr = address.as_ed25519().to_string();
        let path = "v1/balance";
        let query = query_tuples_to_query_string([Some(("address", addr))]);

        let mut url = self.cloud_api_url.clone();
        url.set_path(path);
        url.set_query(query.as_deref());

        // TODO: Add bearer

        let res = self.http_client.get_bytes(url, bearer, self.get_timeout()).await?;
        print!("code: {}", &res.status());
        res.into_text().await
    }

    #[allow(dead_code)]
    pub(crate) async fn post_request<T: DeserializeOwned + Debug + Serialize>(
        &self,
        bearer: &str,
        path: &str,
        query: Option<&str>,
        json: serde_json::Value,
    ) -> ApiResult<T> {
        let mut url = self.cloud_api_url.clone();
        url.set_path(path);
        url.set_query(query);

        // TODO set username & password?

        let res = self.http_client.post_json(url, bearer, self.get_timeout(), json).await;
        match res {
            Ok(r) => r.into_json().await,
            Err(e) => Err(e),
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn get_request<T: DeserializeOwned + Debug + Serialize>(
        &self,
        bearer: &str,
        path: &str,
        query: Option<&str>,
    ) -> ApiResult<T> {
        let mut url = self.cloud_api_url.clone();
        url.set_path(path);
        url.set_query(query);

        // TODO: Add bearer

        let res = self.http_client.get_bytes(url, bearer, self.get_timeout()).await;
        match res {
            Ok(r) => r.into_json().await,
            Err(e) => Err(e),
        }
    }
}
