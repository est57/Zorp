use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct Scripts {
    pub pre_request: String,
    pub test: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
}

/// Results of the pre-request and test scripts for one request.
#[derive(Debug, Clone, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ScriptReport {
    pub tests: Vec<TestResult>,
    pub logs: Vec<LogEntry>,
    pub errors: Vec<String>,
    /// True when a script changed environment or collection variables.
    pub variables_changed: bool,
    /// Values read by the request's "after response" variables, in table order.
    pub captured: Vec<CapturedVar>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CapturedVar {
    pub name: String,
    /// None when the expression found nothing in the response.
    pub value: Option<String>,
}
