use std::time::Duration;
use std::sync::Arc;
use identity_demia::demia::DemiaDocument;
use serde::{Deserialize, Serialize};
use url::Url;
use tokio::sync::RwLock;

use crate::models::GuardianReport;
use crate::clients::HttpClient;
use crate::errors::{ApiError, ApiResult};
use crate::models::{CreateIdResponse, CreateProjectResponse, DataSendWrap, GuardianAccessTokenWrap, HederaLoginForm, LoginCredentials, LoginResponse, NewStreamRequest, TransportMessageWrap, UserMetadata, UserProfile};
use crate::utils::{USER_STATE_API, USER_STATE_TIMEOUT};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStateApi {
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) http_client: HttpClient,
    pub(crate) url: Url,
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) access_token: Arc<RwLock<String>>,
}

impl Default for UserStateApi {
    fn default() -> Self {
        Self {
            url: Url::parse(USER_STATE_API).unwrap(),
            http_client: HttpClient::new("demia".to_string()),
            access_token: Arc::new(RwLock::new(String::default())),
        }
    }
}

impl UserStateApi {
    pub fn new<T: TryInto<Url>>(url: T) -> ApiResult<Self>
    where
        T::Error: std::fmt::Display,
    {
        Ok(Self {
            http_client: HttpClient::new("demia".to_string()),
            url: url.try_into().map_err(|e| ApiError::NotFound(e.to_string()))?,
            access_token: Arc::new(RwLock::new(String::new()))
        })
    }


    fn get_path(&self, path: &str, site_id: Option<&str>) -> String {
        match path {
            "login" => "api/login".to_string(),
            "metadata" => "api/metadata".to_string(),
            "profile" => "api/profile".to_string(),
            "guardian_login" => "api/guardian/login".to_string(),
            "guardian_report" => format!("api/{}/guardian/report", site_id.unwrap_or_default()),
            "did_doc" => "api/did/doc".to_string(),
            "guardian_credentials" => "api/guardian/credentials".to_string(),
            "is_site_admin" => format!("api/{}/admin", site_id.unwrap_or_default()),
            "identity_create" => "api/identity/create".to_string(),
            "data_send" => "api/data/send".to_string(),
            "project_create" => "api/project/create".to_string(),
            _ => self.url.to_string(),
        }
    }

    fn get_timeout(&self) -> Duration {
        USER_STATE_TIMEOUT
    }


    pub async fn get_token(&self, login_credentials: LoginCredentials) -> ApiResult<LoginResponse> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("login", None));

        let response: LoginResponse = self.http_client.post_json(
            url,
            &self.access_token().await,
            self.get_timeout(),
            serde_json::to_value(login_credentials)?
        )
            .await?
            .into_json()
            .await?;

        *self.access_token.write().await = response.token.clone();
        Ok(response)
    }

    async fn access_token(&self) -> String {
        self.access_token.read().await.to_string()
    }


    pub async fn get_metadata(&self) -> ApiResult<UserMetadata> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("metadata", None));

        log::info!("Access Token: {}", self.access_token().await);
        let response: UserMetadata = self.http_client.get(
            url,
            &self.access_token().await,
            self.get_timeout()
        )
            .await?
            .into_json()
            .await?;

        log::info!("Got metadata");
        Ok(response)
    }

    pub async fn set_metadata(&self, metadata: UserMetadata) -> ApiResult<()> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("metadata", None));

        self.http_client.post_json(
            url,
            &self.access_token().await,
            self.get_timeout(),
            serde_json::to_value(metadata)?
        )
            .await?;

        Ok(())
    }

    pub async fn get_profile(&self) -> ApiResult<UserProfile> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("profile", None));

        let response: UserProfile = self.http_client.get(
            url,
            &self.access_token().await,
            self.get_timeout()
        )
            .await?
            .into_json()
            .await?;

        Ok(response)
    }

    pub async fn get_guardian_token(&self) -> ApiResult<GuardianAccessTokenWrap> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("guardian_login", None));

        let response: GuardianAccessTokenWrap = self.http_client.get(
            url,
            &self.access_token().await,
            self.get_timeout()
        )
            .await?
            .into_json()
            .await?;

        Ok(response)
    }

    pub async fn post_guardian_report(&self, site_id: &str, report: GuardianReport) -> ApiResult<String> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("guardian_report", Some(site_id)));

        let response: String = self.http_client.post_json(
            url,
            &self.access_token().await,
            self.get_timeout(),
            serde_json::to_value(report)?
        )
            .await?
            .into_text()
            .await?;

        Ok(response)
    }

    pub async fn get_demia_doc(&self) -> ApiResult<DemiaDocument> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("did_doc", None));

        let response: DemiaDocument = self.http_client.get(
            url,
            &self.access_token().await,
            self.get_timeout()
        )
            .await?
            .into_json()
            .await?;

        Ok(response)
    }

    pub async fn set_demia_doc(&self, doc: DemiaDocument) -> ApiResult<String> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("did_doc", None));

        let response: String = self.http_client.post_json(
            url,
            &self.access_token().await,
            self.get_timeout(),
            serde_json::to_value(doc)?
        )
            .await?
            .into_text()
            .await?;

        Ok(response)
    }

    pub async fn set_guardian_credentials(&self, credentials: HederaLoginForm) -> ApiResult<String> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("guardian_credentials", None));

        let response: String = self.http_client.post_json(
            url,
            &self.access_token().await,
            self.get_timeout(),
            serde_json::to_value(credentials)?
        )
            .await?
            .into_text()
            .await?;

        Ok(response)
    }

    pub async fn get_is_site_admin(&self, site_id: &str) -> ApiResult<bool> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("is_site_admin", Some(site_id)));

        let response: bool = self.http_client.get(
            url,
            &self.access_token().await,
            self.get_timeout()
        )
            .await?
            .into_json()
            .await?;

        Ok(response)
    }

    pub async fn create_identity(&self) -> ApiResult<CreateIdResponse> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("identity_create", None));

        let response: CreateIdResponse = self.http_client.post_json(
            url,
            &self.access_token().await,
            self.get_timeout(),
            serde_json::to_value(())?
        )
            .await?
            .into_json()
            .await?;

        Ok(response)
    }

    pub async fn send_data(&self, data: DataSendWrap) -> ApiResult<Vec<TransportMessageWrap>> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("data_send", None));

        let response: Vec<TransportMessageWrap> = self.http_client.post_json(
            url,
            &self.access_token().await,
            self.get_timeout(),
            serde_json::to_value(data)?
        )
            .await?
            .into_json()
            .await?;

        Ok(response)
    }

    pub async fn send_project(&self, data: NewStreamRequest) -> ApiResult<CreateProjectResponse> {
        let mut url = self.url.clone();
        url.set_path(&self.get_path("project_create", None));

        let response: CreateProjectResponse = self.http_client.post_json(
            url,
            &self.access_token().await,
            self.get_timeout(),
            serde_json::to_value(data)?
        )
            .await?
            .into_json()
            .await?;

        Ok(response)
    }

}
