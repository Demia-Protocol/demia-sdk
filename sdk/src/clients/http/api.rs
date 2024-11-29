use std::{fmt::Debug, time::Duration};

use iota_sdk::types::block::address::Address;
use serde::{Serialize, de::DeserializeOwned};
use url::Url;

use crate::{
    clients::{HttpClient, query_tuples_to_query_string},
    configuration::ApplicationConfiguration,
    errors::{ApiError, ApiResult},
    utils::constants::{GUARDIAN_API, LOCAL_API, RETRIEVER_API},
};

pub struct ApiClient {
    pub(crate) cloud_api_url: Url,
    pub(crate) retriever_url: Url,
    pub(crate) guardian_url: Url,
    pub(crate) http_client: HttpClient,
}

impl Default for ApiClient {
    fn default() -> Self {
        // Ensure its the same port as you set in the .env from the local_api folder
        Self {
            cloud_api_url: Url::parse(LOCAL_API).unwrap(),
            retriever_url: Url::parse(RETRIEVER_API).unwrap(),
            guardian_url: Url::parse(GUARDIAN_API).unwrap(),
            http_client: HttpClient::new("demia".to_string()),
        }
    }
}

impl TryFrom<&ApplicationConfiguration> for ApiClient {
    type Error = ApiError;
    fn try_from(config: &ApplicationConfiguration) -> Result<Self, Self::Error> {
        Ok(Self {
            cloud_api_url: Url::parse(&config.amazon_api)?,
            retriever_url: Url::parse(&config.retriever_api)?,
            guardian_url: Url::parse(&config.guardian_api)?,
            ..Default::default()
        })
    }
}

impl ApiClient {
    pub fn new<T: TryInto<Url>>(cloud_api_url: T, retriever_url: Option<T>, guardian_url: Option<T>) -> ApiResult<Self>
    where
        T::Error: std::fmt::Display,
    {
        let mut client = Self {
            http_client: HttpClient::new("demia".to_string()),
            cloud_api_url: cloud_api_url
                .try_into()
                .map_err(|e| ApiError::NotFound(e.to_string()))?,
            ..Default::default()
        };

        if let Some(retriever_url) = retriever_url {
            client.retriever_url = retriever_url
                .try_into()
                .map_err(|e| ApiError::NotFound(e.to_string()))?;
        }
        if let Some(guardian_url) = guardian_url {
            client.guardian_url = guardian_url.try_into().map_err(|e| ApiError::NotFound(e.to_string()))?;
        }

        Ok(client)
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
