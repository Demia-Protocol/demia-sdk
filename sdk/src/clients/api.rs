use std::{fmt::Debug, time::Duration};

use iota_sdk::types::block::address::Address;
use serde::{Serialize, de::DeserializeOwned};
use url::Url;

use crate::{clients::HttpClient, errors::ApiResult};

pub struct ApiClient {
    url: Url,
    http_client: HttpClient,
}

impl Default for ApiClient {
    fn default() -> Self {
        // Ensure its the same port as you set in the .env from the local_api folder
        Self {
            url: Url::parse("http://localhost:1111").unwrap(),
            http_client: HttpClient::new("demia".to_string()),
        }
    }
}

impl ApiClient {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            http_client: HttpClient::new("demia".to_string()),
        }
    }

    fn get_timeout(&self) -> Duration {
        Duration::from_secs(10)
    }

    pub async fn request_balance(&self, bearer: &str, address: &Address) -> ApiResult<String> {
        let addr = address.as_ed25519().to_string();
        let path = "v1/balance";
        let query = query_tuples_to_query_string([Some(("address", addr))]);

        let mut url = self.url.clone();
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
        let mut url = self.url.clone();
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
        let mut url = self.url.clone();
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

pub(crate) fn query_tuples_to_query_string(
    tuples: impl IntoIterator<Item = Option<(&'static str, String)>>,
) -> Option<String> {
    let query = tuples
        .into_iter()
        .filter_map(|tuple| tuple.map(|(key, value)| format!("{}={}", key, value)))
        .collect::<Vec<_>>();

    if query.is_empty() { None } else { Some(query.join("&")) }
}
