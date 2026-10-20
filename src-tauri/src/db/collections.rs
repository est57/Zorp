use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use super::folders::folder_parent;
use super::node_settings::scripts_from;
use super::{ensure_changed, uuid_at, validate_name};
use crate::error::AppResult;
use crate::models::collection::{Collection, NodeKind};
use crate::models::request::Auth;

pub(super) fn auth_from(json: Option<String>) -> Option<Auth> {
    json.and_then(|j| serde_json::from_str(&j).ok())
}

pub fn list_collections(conn: &Connection, workspace_id: Uuid) -> AppResult<Vec<Collection>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, auth_json, sort_order, scripts_json, docs FROM collections
         WHERE workspace_id = ?1 ORDER BY sort_order, name",
    )?;
    let rows = stmt.query_map([workspace_id.to_string()], |row| {
        Ok(Collection {
            id: uuid_at(row, 0)?,
            name: row.get(1)?,
            auth: auth_from(row.get(2)?),
            sort_order: row.get(3)?,
            scripts: scripts_from(row.get(4)?),
            docs: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn create_collection(
    conn: &Connection,
    workspace_id: Uuid,
    name: &str,
) -> AppResult<Collection> {
    let name = validate_name(name)?;
    let sort_order: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM collections WHERE workspace_id = ?1",
        [workspace_id.to_string()],
        |row| row.get(0),
    )?;
    let collection = Collection {
        scripts: Default::default(),
        docs: String::new(),
        id: Uuid::now_v7(),
        name,
        auth: None,
        sort_order,
    };
    conn.execute(
        "INSERT INTO collections (id, workspace_id, name, sort_order) VALUES (?1, ?2, ?3, ?4)",
        params![
            collection.id.to_string(),
            workspace_id.to_string(),
            collection.name,
            sort_order
        ],
    )?;
    Ok(collection)
}

pub fn rename_collection(conn: &Connection, id: Uuid, name: &str) -> AppResult<()> {
    let name = validate_name(name)?;
    let n = conn.execute(
        "UPDATE collections SET name = ?1 WHERE id = ?2",
        params![name, id.to_string()],
    )?;
    ensure_changed(n, "Koleksi")
}

pub fn delete_collection(conn: &Connection, id: Uuid) -> AppResult<()> {
    let n = conn.execute("DELETE FROM collections WHERE id = ?1", [id.to_string()])?;
    ensure_changed(n, "Koleksi")
}

/// Auth inherited by a request: nearest folder auth, then collection auth.
pub fn inherited_auth(
    conn: &Connection,
    collection_id: Uuid,
    folder_id: Option<Uuid>,
) -> AppResult<Option<Auth>> {
    let mut current = folder_id;
    while let Some(id) = current {
        let auth: Option<String> = conn
            .query_row(
                "SELECT auth_json FROM folders WHERE id = ?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .optional()?
            .flatten();
        if let Some(auth) = auth_from(auth)
            && !matches!(auth, Auth::Inherit)
        {
            return Ok(Some(auth));
        }
        current = folder_parent(conn, id)?;
    }
    let auth: Option<String> = conn
        .query_row(
            "SELECT auth_json FROM collections WHERE id = ?1",
            [collection_id.to_string()],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    Ok(auth_from(auth))
}

/// Stores auth for a collection or folder; `Inherit` (and `None` on collections) clears it.
pub fn set_auth(conn: &Connection, kind: NodeKind, id: Uuid, auth: &Auth) -> AppResult<()> {
    let clear = matches!(auth, Auth::Inherit)
        || (matches!(kind, NodeKind::Collection) && matches!(auth, Auth::None));
    let json = if clear {
        None
    } else {
        Some(serde_json::to_string(auth)?)
    };
    let sql = match kind {
        NodeKind::Collection => "UPDATE collections SET auth_json = ?1 WHERE id = ?2",
        NodeKind::Folder => "UPDATE folders SET auth_json = ?1 WHERE id = ?2",
        NodeKind::Request => {
            return Err(crate::error::AppError::Validation(
                "Auth request diatur di editor request".into(),
            ));
        }
    };
    let n = conn.execute(sql, params![json, id.to_string()])?;
    ensure_changed(n, "Koleksi atau folder")
}

/// Collection-level variables, stored as `[key, value]` pairs.
pub fn variables(conn: &Connection, collection_id: Uuid) -> AppResult<Vec<(String, String)>> {
    let json: Option<String> = conn
        .query_row(
            "SELECT variables_json FROM collections WHERE id = ?1",
            [collection_id.to_string()],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    Ok(json
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default())
}
