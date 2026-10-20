use serde::Serialize;
use ts_rs::TS;

/// Folder sync state of the active workspace.
#[derive(Debug, Clone, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SyncStatus {
    pub folder: Option<String>,
    /// Files that could not be read; sync is paused until they are fixed.
    pub problems: Vec<String>,
}
