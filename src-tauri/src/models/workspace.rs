use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    #[ts(type = "number")]
    pub created_at: i64,
    /// Folder the workspace is synced with; null for a workspace stored only in this app.
    pub folder: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WorkspaceList {
    pub active_id: Uuid,
    pub workspaces: Vec<Workspace>,
}
