use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MockRouteInfo {
    pub method: String,
    /// Path pattern, with parameters shown as `:name`.
    pub path: String,
    pub request_name: String,
    /// Name and status of the example served first, if the request has any.
    pub example: Option<String>,
    pub status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MockStatus {
    pub collection_id: Uuid,
    pub port: u16,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MockLogEntry {
    pub collection_id: Uuid,
    #[ts(type = "number")]
    pub at: i64,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub example: Option<String>,
}
