use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use super::collections::auth_from;
use super::{ensure_changed, opt_uuid_at, uuid_at, validate_name};
use crate::error::AppResult;
use crate::models::collection::Folder;

pub fn list_folders(conn: &Connection, workspace_id: Uuid) -> AppResult<Vec<Folder>> {
    let mut stmt = conn.prepare(
        "SELECT f.id, f.collection_id, f.parent_id, f.name, f.auth_json, f.sort_order
         FROM folders f JOIN collections c ON c.id = f.collection_id
         WHERE c.workspace_id = ?1 ORDER BY f.sort_order, f.name",
    )?;
    let rows = stmt.query_map([workspace_id.to_string()], |row| {
        Ok(Folder {
            id: uuid_at(row, 0)?,
            collection_id: uuid_at(row, 1)?,
            parent_id: opt_uuid_at(row, 2)?,
            name: row.get(3)?,
            auth: auth_from(row.get(4)?),
            sort_order: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn create_folder(
    conn: &Connection,
    collection_id: Uuid,
    parent_id: Option<Uuid>,
    name: &str,
) -> AppResult<Folder> {
    let name = validate_name(name)?;
    let sort_order: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM folders
         WHERE collection_id = ?1 AND parent_id IS ?2",
        params![collection_id.to_string(), parent_id.map(|p| p.to_string())],
        |row| row.get(0),
    )?;
    let folder = Folder {
        id: Uuid::now_v7(),
        collection_id,
        parent_id,
        name,
        auth: None,
        sort_order,
    };
    conn.execute(
        "INSERT INTO folders (id, collection_id, parent_id, name, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            folder.id.to_string(),
            collection_id.to_string(),
            parent_id.map(|p| p.to_string()),
            folder.name,
            sort_order
        ],
    )?;
    Ok(folder)
}

pub fn rename_folder(conn: &Connection, id: Uuid, name: &str) -> AppResult<()> {
    let name = validate_name(name)?;
    let n = conn.execute(
        "UPDATE folders SET name = ?1 WHERE id = ?2",
        params![name, id.to_string()],
    )?;
    ensure_changed(n, "Folder")
}

pub fn delete_folder(conn: &Connection, id: Uuid) -> AppResult<()> {
    let n = conn.execute("DELETE FROM folders WHERE id = ?1", [id.to_string()])?;
    ensure_changed(n, "Folder")
}

/// Moves a folder; folders keep their collection, so only the parent and order change.
pub fn move_folder(
    conn: &Connection,
    id: Uuid,
    parent_id: Option<Uuid>,
    sort_order: f64,
) -> AppResult<()> {
    if let Some(parent) = parent_id
        && is_descendant(conn, parent, id)?
    {
        return Err(crate::error::AppError::Validation(
            "Folder tidak bisa dipindahkan ke dalam dirinya sendiri".into(),
        ));
    }
    let n = conn.execute(
        "UPDATE folders SET parent_id = ?1, sort_order = ?2 WHERE id = ?3",
        params![parent_id.map(|p| p.to_string()), sort_order, id.to_string()],
    )?;
    ensure_changed(n, "Folder")
}

fn is_descendant(conn: &Connection, candidate: Uuid, ancestor: Uuid) -> AppResult<bool> {
    let mut current = Some(candidate);
    while let Some(id) = current {
        if id == ancestor {
            return Ok(true);
        }
        current = folder_parent(conn, id)?;
    }
    Ok(false)
}

pub(super) fn folder_parent(conn: &Connection, id: Uuid) -> AppResult<Option<Uuid>> {
    let parent: Option<Option<String>> = conn
        .query_row(
            "SELECT parent_id FROM folders WHERE id = ?1",
            [id.to_string()],
            |row| row.get(0),
        )
        .optional()?;
    Ok(parent.flatten().and_then(|p| Uuid::parse_str(&p).ok()))
}
