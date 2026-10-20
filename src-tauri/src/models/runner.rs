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
    /// CSV or JSON file with one iteration per row; every column becomes a variable.
    #[serde(default)]
    pub data_path: Option<String>,
}

/// The first rows of an iteration data file, shown before a run.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunDataPreview {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunItemResult {
    pub index: u32,
    pub total: u32,
    /// 1-based iteration, always 1 without data.
    pub iteration: u32,
    /// First value of the iteration's data row, to tell iterations apart.
    pub iteration_label: Option<String>,
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
    pub iterations: u32,
    pub passed: u32,
    pub failed: u32,
    pub duration_ms: u32,
    pub cancelled: bool,
}
