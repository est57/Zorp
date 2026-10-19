use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::settings::WorkspaceSettings;

pub fn load(conn: &Connection, workspace_id: Uuid) -> AppResult<WorkspaceSettings> {
    let json: Option<String> = conn
        .query_row(
            "SELECT settings_json FROM workspace_settings WHERE workspace_id = ?1",
            [workspace_id.to_string()],
            |row| row.get(0),
        )
        .optional()?;
    Ok(json
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default())
}

/// Stores settings; the proxy password is never written to SQLite.
pub fn save(conn: &Connection, workspace_id: Uuid, settings: &WorkspaceSettings) -> AppResult<()> {
    let mut stored = settings.clone();
    stored.proxy.password.clear();
    conn.execute(
        "INSERT INTO workspace_settings (workspace_id, settings_json) VALUES (?1, ?2)
         ON CONFLICT(workspace_id) DO UPDATE SET settings_json = excluded.settings_json",
        params![workspace_id.to_string(), serde_json::to_string(&stored)?],
    )?;
    Ok(())
}
