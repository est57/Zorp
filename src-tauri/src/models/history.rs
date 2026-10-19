use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use super::request::RequestSpec;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HistoryItem {
    pub id: Uuid,
    pub request: RequestSpec,
    pub status: Option<u16>,
    pub duration_ms: Option<u32>,
    #[ts(type = "number | null")]
    pub size_bytes: Option<u64>,
    #[ts(type = "number")]
    pub sent_at: i64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HistoryPage {
    pub items: Vec<HistoryItem>,
    #[ts(type = "number | null")]
    pub next_cursor: Option<i64>,
}
