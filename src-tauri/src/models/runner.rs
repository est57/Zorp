use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::request::HttpMethod;
use super::script::TestResult;

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunnerOptions {
    /// Pause between requests.
    pub delay_ms: u32,
    pub stop_on_failure: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunItemResult {
    pub index: u32,
    pub total: u32,
    pub request_id: Uuid,
    pub name: String,
    pub method: HttpMethod,
    pub status: Option<u16>,
    pub duration_ms: Option<u32>,
    pub tests: Vec<TestResult>,
    pub error: Option<String>,
    /// No error and every test passed.
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunnerEvent {
    pub run_id: Uuid,
    pub item: RunItemResult,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunSummary {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub duration_ms: u32,
    pub cancelled: bool,
}
