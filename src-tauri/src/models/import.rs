use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ImportReport {
    pub collection_id: Uuid,
    pub collection_name: String,
    pub folders: u32,
    pub requests: u32,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExportReport {
    pub requests: u32,
    /// Requests left out because Postman collections cannot represent them.
    pub skipped: Vec<String>,
}
