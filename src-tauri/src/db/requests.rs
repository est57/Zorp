use rusqlite::{Connection, params};
use uuid::Uuid;

use super::{ensure_changed, now_ms, opt_uuid_at, uuid_at, validate_name};
use crate::error::AppResult;
use crate::models::collection::{MoveTarget, SavedRequest};
use crate::models::request::RequestSpec;

pub fn list(conn: &Connection, workspace_id: Uuid) -> AppResult<Vec<SavedRequest>> {
    let mut stmt = conn.prepare(
        "SELECT r.collection_id, r.folder_id, r.sort_order, r.spec_json
         FROM requests r JOIN collections c ON c.id = r.collection_id
         WHERE c.workspace_id = ?1 ORDER BY r.sort_order",
    )?;
    let rows = stmt.query_map([workspace_id.to_string()], |row| {
        Ok((
            uuid_at(row, 0)?,
            opt_uuid_at(row, 1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    let mut items = Vec::new();
    for row in rows {
        let (collection_id, folder_id, sort_order, json) = row?;
        // Skip rows that no longer match the current schema instead of failing the whole tree.
        if let Ok(spec) = serde_json::from_str::<RequestSpec>(&json) {
            items.push(SavedRequest {
                collection_id,
                folder_id,
                sort_order,
                spec,
            });
        }
    }
    Ok(items)
}

/// Inserts or updates a request, keeping its position when it already exists.
pub fn save(
    conn: &Connection,
    mut spec: RequestSpec,
    collection_id: Uuid,
    folder_id: Option<Uuid>,
) -> AppResult<SavedRequest> {
    spec.name = validate_name(&spec.name)?;
    let json = serde_json::to_string(&spec)?;
    let next_order: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM requests
         WHERE collection_id = ?1 AND folder_id IS ?2",
        params![collection_id.to_string(), folder_id.map(|f| f.to_string())],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO requests (id, collection_id, folder_id, spec_json, sort_order, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET spec_json = excluded.spec_json, updated_at = excluded.updated_at",
        params![
            spec.id.to_string(),
            collection_id.to_string(),
            folder_id.map(|f| f.to_string()),
            json,
            next_order,
            now_ms()
        ],
    )?;
    let (collection_id, folder_id, sort_order) = conn.query_row(
        "SELECT collection_id, folder_id, sort_order FROM requests WHERE id = ?1",
        [spec.id.to_string()],
        |row| Ok((uuid_at(row, 0)?, opt_uuid_at(row, 1)?, row.get(2)?)),
    )?;
    Ok(SavedRequest {
        collection_id,
        folder_id,
        sort_order,
        spec,
    })
}

pub fn rename(conn: &Connection, id: Uuid, name: &str) -> AppResult<()> {
    let name = validate_name(name)?;
    let n = conn.execute(
        "UPDATE requests SET spec_json = json_set(spec_json, '$.name', ?1), updated_at = ?2 WHERE id = ?3",
        params![name, now_ms(), id.to_string()],
    )?;
    ensure_changed(n, "Request")
}

pub fn delete(conn: &Connection, id: Uuid) -> AppResult<()> {
    let n = conn.execute("DELETE FROM requests WHERE id = ?1", [id.to_string()])?;
    ensure_changed(n, "Request")
}

pub fn move_to(conn: &Connection, id: Uuid, target: &MoveTarget) -> AppResult<()> {
    let n = conn.execute(
        "UPDATE requests SET collection_id = ?1, folder_id = ?2, sort_order = ?3 WHERE id = ?4",
        params![
            target.collection_id.to_string(),
            target.folder_id.map(|f| f.to_string()),
            target.sort_order,
            id.to_string()
        ],
    )?;
    ensure_changed(n, "Request")
}

/// Location of a saved request, used to resolve inherited auth and variables.
pub fn location(conn: &Connection, id: Uuid) -> AppResult<Option<(Uuid, Option<Uuid>)>> {
    use rusqlite::OptionalExtension;
    Ok(conn
        .query_row(
            "SELECT collection_id, folder_id FROM requests WHERE id = ?1",
            [id.to_string()],
            |row| Ok((uuid_at(row, 0)?, opt_uuid_at(row, 1)?)),
        )
        .optional()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{collections, open_in_memory, workspace};
    use crate::models::request::{Auth, Body, HttpMethod, RequestSettings};

    #[test]
    fn save_upserts_and_move_changes_location() {
        let conn = open_in_memory().unwrap();
        let ws = workspace::current(&conn).unwrap().id;
        let col = collections::create_collection(&conn, ws, "API").unwrap();
        let folder = crate::db::folders::create_folder(&conn, col.id, None, "Users").unwrap();
        let spec = RequestSpec {
            docs: String::new(),
            id: Uuid::now_v7(),
            name: "List".into(),
            method: HttpMethod::Get,
            url: "{{base}}/users".into(),
            params: vec![],
            headers: vec![],
            body: Body::None,
            auth: Auth::Inherit,
            settings: RequestSettings::default(),
            mode: Default::default(),
            scripts: Default::default(),
        };
        let first = save(&conn, spec.clone(), col.id, None).unwrap();
        let again = save(
            &conn,
            RequestSpec {
                name: "List users".into(),
                ..spec.clone()
            },
            col.id,
            None,
        )
        .unwrap();
        assert_eq!(first.sort_order, again.sort_order);

        move_to(
            &conn,
            spec.id,
            &MoveTarget {
                collection_id: col.id,
                folder_id: Some(folder.id),
                sort_order: 5.0,
            },
        )
        .unwrap();
        rename(&conn, spec.id, "Renamed").unwrap();
        let items = list(&conn, ws).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].folder_id, Some(folder.id));
        assert_eq!(items[0].spec.name, "Renamed");

        collections::delete_collection(&conn, col.id).unwrap();
        assert!(list(&conn, ws).unwrap().is_empty());
    }
}
