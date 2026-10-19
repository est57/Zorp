use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CookieInfo {
    pub domain: String,
    pub path: String,
    pub name: String,
    pub value: String,
    #[ts(type = "number | null")]
    pub expires_at: Option<i64>,
    pub secure: bool,
    pub http_only: bool,
    pub host_only: bool,
}
