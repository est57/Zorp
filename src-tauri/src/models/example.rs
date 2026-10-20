use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// A response saved with a request, shared through the workspace folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ResponseExample {
    pub id: Uuid,
    pub request_id: Uuid,
    pub name: String,
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub sort_order: f64,
}

/// What the sidebar needs to list an example without loading its body.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ExampleSummary {
    pub id: Uuid,
    pub request_id: Uuid,
    pub name: String,
    pub status: u16,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NewExample {
    pub request_id: Uuid,
    pub name: String,
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}
