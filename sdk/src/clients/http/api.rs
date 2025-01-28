use std::{fmt::Debug, time::Duration};

use iota_sdk::types::block::address::Address;
use serde::{Serialize, de::DeserializeOwned};
use url::Url;

use super::{GuardianApiClient, retriever::RetrieverApi};
use crate::{
    clients::{HttpClient, query_tuples_to_query_string},
    configuration::ApplicationConfiguration,
    errors::{ApiError, ApiResult},
    utils::{
        API_TIMEOUT,
        constants::{GUARDIAN_API, LOCAL_API, RETRIEVER_API},
    },
};

pub struct ApiClient {
    pub(crate) cloud_api_url: Url,
    pub(crate) retriever: RetrieverApi,
    pub(crate) guardian: GuardianApiClient,
    pub(crate) http_client: HttpClient,
}

impl Default for ApiClient {
    fn default() -> Self {
        // Ensure its the same port as you set in the .env from the local_api folder
        Self {
            cloud_api_url: Url::parse(LOCAL_API).unwrap(),
            retriever: RetrieverApi::new(RETRIEVER_API).unwrap(),
            guardian: GuardianApiClient::new(GUARDIAN_API).unwrap(),
            http_client: HttpClient::new("demia".to_string()),
        }
    }
}

impl TryFrom<&ApplicationConfiguration> for ApiClient {
    type Error = ApiError;
    fn try_from(config: &ApplicationConfiguration) -> Result<Self, Self::Error> {
        log::info!("{}, {}", config.amazon_api, Url::parse(&config.amazon_api)?);

        Ok(Self {
            cloud_api_url: Url::parse(&config.amazon_api)?,
            guardian: GuardianApiClient::new(config.guardian_api.as_ref())?,
            retriever: RetrieverApi::new(config.retriever_api.as_ref())?,
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
            client.retriever = RetrieverApi::new(retriever_url)?;
        }
        if let Some(guardian_url) = guardian_url {
            client.guardian = GuardianApiClient::new(guardian_url)?;
        }

        Ok(client)
    }

    pub(crate) fn get_timeout() -> Duration {
        API_TIMEOUT
    }

    pub fn retriever(&self) -> &RetrieverApi {
        &self.retriever
    }

    pub fn guardian(&self) -> &GuardianApiClient {
        &self.guardian
    }

    pub async fn request_balance(
        &self,
        bearer: &str,
        address: &Address,
        faucet: &str,
        node: &str,
    ) -> ApiResult<String> {
        let addr = address.as_ed25519().to_string();
        let path = "/v1/balance";
        let query = query_tuples_to_query_string([
            Some(("address", addr)),
            Some(("faucetUrl", faucet.to_string())),
            Some(("nodeUrl", node.to_string())),
        ]);

        let mut url = self.cloud_api_url.clone();

        // Account for existing path in uri
        let mut full_path = url.path().to_string();
        if let Some(stripped) = full_path.strip_suffix("/") {
            full_path = stripped.to_string();
        }
        full_path.push_str(path);

        url.set_path(&full_path);
        url.set_query(query.as_deref());

        // TODO: Add bearer

        let res = self.http_client.get_bytes(url, bearer, Self::get_timeout()).await?;
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

        let res = self.http_client.post_json(url, bearer, Self::get_timeout(), json).await;
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

        let res = self.http_client.get_bytes(url, bearer, Self::get_timeout()).await;
        match res {
            Ok(r) => r.into_json().await,
            Err(e) => Err(e),
        }
    }
}
