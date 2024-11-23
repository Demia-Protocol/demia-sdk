use iota_sdk::types::block::address::Address;
use serde_json::Value;

use crate::{
    clients::ApiClient,
    configuration::{
        ApplicationConfiguration, IdentityConfiguration, LoggingConfiguration, StreamsConfiguration,
        StrongholdConfiguration,
    },
    errors::ApiResult,
};

impl ApiClient {
    // TODO: Make this... cleaner we have a token we can use to fetch!
    pub async fn add_retriever_new_user(
        &self,
        bearer: &str,
        streams: &StreamsConfiguration,
        identity: &IdentityConfiguration,
        stronghold: &StrongholdConfiguration,
        application: &ApplicationConfiguration,
        logging: &LoggingConfiguration,
        stream_addresses: Vec<String>,
    ) -> ApiResult<Value> {
        let path = "new-user";

        let mut url = self.retriever_url.clone();
        url.set_path(path);

        let json = serde_json::json!({
            "streams": streams,
            "identity": identity,
            "stronghold": stronghold,
            "application": application,
            "logging": logging,
            "stream_addresses": stream_addresses,
        });

        let res = self
            .http_client
            .post_json(url, bearer, self.get_timeout(), json)
            .await?;
        print!("code: {}", &res.status());
        res.into_json().await
    }

    pub async fn add_retriever_user_address(
        &self,
        bearer: &str,
        user_id: String,
        address: &Address,
    ) -> ApiResult<Value> {
        let addr = address.as_ed25519().to_string();
        let path = "add-address";

        let mut url = self.retriever_url.clone();
        url.set_path(path);

        let json = serde_json::json!({
            "id": user_id,
            "address": addr,
        });

        let res = self
            .http_client
            .post_json(url, bearer, self.get_timeout(), json)
            .await?;
        print!("code: {}", &res.status());
        res.into_json().await
    }
}
