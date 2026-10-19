use serde::Serialize;
use ts_rs::TS;

use super::workspace::Workspace;

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppInfo {
    pub version: String,
    #[ts(type = "number")]
    pub schema_version: i64,
    pub workspace: Workspace,
}
