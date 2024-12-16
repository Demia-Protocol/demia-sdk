// TODO: add function for fetching from db for externally triggered notifications (i.e. sub requests)
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification2 {
    pub user: String,
    pub notification_type: NotificationType,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NotificationType {
    #[default]
    Login,
    SubRequest {
        site: String,
    },
    NewSite,
}
