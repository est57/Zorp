use rusqlite::{Connection, OptionalExtension, Row, params};
use uuid::Uuid;

use super::{ensure_changed, now_ms, uuid_at, validate_name};
use crate::error::AppResult;
use crate::models::workspace::Workspace;

const DEFAULT_NAME: &str = "Workspace Saya";
const COLUMNS: &str = "id, name, created_at, folder_path";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Workspace> {
    Ok(Workspace {
        id: uuid_at(row, 0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        folder: row.get(3)?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<Workspace>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM workspaces ORDER BY name COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map([], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn get(conn: &Connection, id: Uuid) -> AppResult<Option<Workspace>> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM workspaces WHERE id = ?1"),
            [id.to_string()],
            from_row,
        )
        .optional()?)
}

pub fn find_by_folder(conn: &Connection, folder: &str) -> AppResult<Option<Workspace>> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM workspaces WHERE folder_path = ?1"),
            [folder],
            from_row,
        )
        .optional()?)
}

/// The workspace that already stores one of these collection or environment ids.
pub fn owner_of(conn: &Connection, ids: &[Uuid]) -> AppResult<Option<Workspace>> {
    for id in ids {
        let owner: Option<String> = conn
            .query_row(
                "SELECT workspace_id FROM collections WHERE id = ?1
                 UNION SELECT workspace_id FROM environments WHERE id = ?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(owner) = owner.and_then(|o| Uuid::parse_str(&o).ok()) {
            return get(conn, owner);
        }
    }
    Ok(None)
}

pub fn create(conn: &Connection, name: &str, folder: Option<&str>) -> AppResult<Workspace> {
    let workspace = Workspace {
        id: Uuid::now_v7(),
        name: validate_name(name)?,
        created_at: now_ms(),
        folder: folder.map(str::to_string),
    };
    conn.execute(
        "INSERT INTO workspaces (id, name, created_at, folder_path) VALUES (?1, ?2, ?3, ?4)",
        params![
            workspace.id.to_string(),
            workspace.name,
            workspace.created_at,
            workspace.folder
        ],
    )?;
    Ok(workspace)
}

/// The workspace opened most recently, creating the default one on first start.
pub fn current(conn: &Connection) -> AppResult<Workspace> {
    let recent = conn
        .query_row(
            &format!(
                "SELECT {COLUMNS} FROM workspaces ORDER BY last_opened_at DESC, created_at LIMIT 1"
            ),
            [],
            from_row,
        )
        .optional()?;
    match recent {
        Some(workspace) => Ok(workspace),
        None => create(conn, DEFAULT_NAME, None),
    }
}

pub fn touch(conn: &Connection, id: Uuid) -> AppResult<()> {
    let n = conn.execute(
        "UPDATE workspaces SET last_opened_at = ?1 WHERE id = ?2",
        params![now_ms(), id.to_string()],
    )?;
    ensure_changed(n, "Workspace")
}

pub fn rename(conn: &Connection, id: Uuid, name: &str) -> AppResult<()> {
    let n = conn.execute(
        "UPDATE workspaces SET name = ?1 WHERE id = ?2",
        params![validate_name(name)?, id.to_string()],
    )?;
    ensure_changed(n, "Workspace")
}

pub fn set_folder(conn: &Connection, id: Uuid, folder: Option<&str>) -> AppResult<()> {
    let n = conn.execute(
        "UPDATE workspaces SET folder_path = ?1 WHERE id = ?2",
        params![folder, id.to_string()],
    )?;
    ensure_changed(n, "Workspace")
}

/// Removes a workspace and its data from this app; files in a linked folder are left untouched.
pub fn delete(conn: &Connection, id: Uuid) -> AppResult<()> {
    let n = conn.execute("DELETE FROM workspaces WHERE id = ?1", [id.to_string()])?;
    ensure_changed(n, "Workspace")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn current_follows_the_most_recently_opened_workspace() {
        let conn = open_in_memory().unwrap();
        let first = current(&conn).unwrap();
        assert_eq!(current(&conn).unwrap().id, first.id);
        let team = create(&conn, "Tim", Some("/tmp/zorp-team")).unwrap();
        touch(&conn, team.id).unwrap();
        assert_eq!(current(&conn).unwrap().id, team.id);
        assert_eq!(
            find_by_folder(&conn, "/tmp/zorp-team").unwrap().unwrap().id,
            team.id
        );
        assert_eq!(list(&conn).unwrap().len(), 2);
        delete(&conn, team.id).unwrap();
        assert_eq!(current(&conn).unwrap().id, first.id);
    }
}
