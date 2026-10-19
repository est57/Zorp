use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::error::AppResult;

pub fn load(conn: &Connection, workspace_id: Uuid) -> AppResult<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT jar_json FROM cookie_jars WHERE workspace_id = ?1",
            [workspace_id.to_string()],
            |row| row.get(0),
        )
        .optional()?)
}

pub fn save(conn: &Connection, workspace_id: Uuid, json: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO cookie_jars (workspace_id, jar_json) VALUES (?1, ?2)
         ON CONFLICT(workspace_id) DO UPDATE SET jar_json = excluded.jar_json",
        params![workspace_id.to_string(), json],
    )?;
    Ok(())
}
