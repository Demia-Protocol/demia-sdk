// Copyright 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! The user manager that takes care of sending requests with healthy users and quorum if enabled

use std::time::Duration;

use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
use serde_json::Value;
use url::Url;

use crate::errors::{ApiError, ApiResult as Result};

pub(crate) struct Response(reqwest::Response);

impl Response {
    pub(crate) fn status(&self) -> u16 {
        self.0.status().as_u16()
    }

    pub(crate) async fn into_json<T: DeserializeOwned>(self) -> Result<T> {
        self.0.json().await.map_err(Into::into)
    }

    #[cfg(not(target_family = "wasm"))]
    pub(crate) async fn into_text(self) -> Result<String> {
        self.0.text().await.map_err(Into::into)
    }

    #[allow(dead_code)]
    pub(crate) async fn into_bytes(self) -> Result<Vec<u8>> {
        self.0.bytes().await.map(|b| b.to_vec()).map_err(Into::into)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct HttpClient {
    client: reqwest::Client,
    pub(crate) user_agent: String,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            user_agent: "demia".to_string(),
        }
    }
}

impl HttpClient {
    pub(crate) fn new(user_agent: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            user_agent,
        }
    }

    async fn parse_response(response: reqwest::Response, url: &url::Url) -> Result<Response> {
        let status = response.status();
        if status.is_success() {
            Ok(Response(response))
        } else {
            let text = response.text().await?;

            if status.as_u16() == 404 {
                Err(ApiError::NotFound(url.to_string()))
            } else {
                Err(ApiError::ResponseError {
                    code: status.as_u16(),
                    text,
                    url: url.to_string(),
                })
            }
        }
    }

    fn build_request(
        &self,
        request_builder: RequestBuilder,
        bearer: &str,
        _timeout: Duration,
    ) -> Result<RequestBuilder> {
        let mut request_builder = request_builder.header(reqwest::header::USER_AGENT, &self.user_agent);

        request_builder = request_builder.bearer_auth(bearer);

        #[cfg(not(target_family = "wasm"))]
        {
            request_builder = request_builder.timeout(_timeout);
        }
        Ok(request_builder)
    }

    #[allow(dead_code)]
    pub(crate) async fn get(&self, url: Url, bearer: &str, timeout: Duration) -> Result<Response> {
        let mut request_builder = self.client.get(url.clone());
        request_builder = self.build_request(request_builder, bearer, timeout)?;
        let start_time = tokio::time::Instant::now();
        let resp = request_builder.send().await?;
        log::debug!(
            "GET: {:?} ms for {} {}",
            start_time.elapsed().as_millis(),
            resp.status(),
            &url
        );
        Self::parse_response(resp, &url).await
    }

    // Get with header: "accept", "application/vnd.iota.serializer-v2"
    pub(crate) async fn get_bytes(&self, url: Url, bearer: &str, timeout: Duration) -> Result<Response> {
        let mut request_builder = self.client.get(url.clone());
        request_builder = self.build_request(request_builder, bearer, timeout)?;
        request_builder = request_builder.header("accept", "application/vnd.demia.serializer-v2");
        let resp = request_builder.send().await?;
        Self::parse_response(resp, &url).await
    }

    pub(crate) async fn post_json(&self, url: Url, bearer: &str, timeout: Duration, json: Value) -> Result<Response> {
        let mut request_builder = self.client.post(url.clone());
        request_builder = self.build_request(request_builder, bearer, timeout)?;
        Self::parse_response(request_builder.json(&json).send().await?, &url).await
    }

    #[allow(dead_code)]
    pub(crate) async fn post_bytes(&self, url: Url, bearer: &str, timeout: Duration, body: &[u8]) -> Result<Response> {
        let mut request_builder = self.client.post(url.clone());
        request_builder = self.build_request(request_builder, bearer, timeout)?;
        request_builder = request_builder.header("Content-Type", "application/vnd.demia.serializer-v2");
        Self::parse_response(request_builder.body(body.to_vec()).send().await?, &url).await
    }
}
