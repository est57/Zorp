use rusqlite::{Connection, Row, params};
use uuid::Uuid;

use super::{ensure_changed, uuid_at, validate_name};
use crate::error::{AppError, AppResult};
use crate::models::example::{ExampleSummary, NewExample, ResponseExample};

/// Larger bodies make the workspace files hard to review in Git.
const MAX_BODY_BYTES: usize = 1024 * 1024;
/// Headers that carry session or credential data are never stored.
const DROPPED_HEADERS: [&str; 4] = [
    "set-cookie",
    "cookie",
    "authorization",
    "proxy-authorization",
];

const COLUMNS: &str =
    "e.id, e.request_id, e.name, e.status, e.status_text, e.headers_json, e.body, e.sort_order";

fn from_row(row: &Row<'_>) -> rusqlite::Result<ResponseExample> {
    let headers: String = row.get(5)?;
    Ok(ResponseExample {
        id: uuid_at(row, 0)?,
        request_id: uuid_at(row, 1)?,
        name: row.get(2)?,
        status: row.get(3)?,
        status_text: row.get(4)?,
        headers: serde_json::from_str(&headers).unwrap_or_default(),
        body: row.get(6)?,
        sort_order: row.get(7)?,
    })
}

/// All examples of a workspace, with bodies, in display order.
pub fn list(conn: &Connection, workspace_id: Uuid) -> AppResult<Vec<ResponseExample>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM request_examples e
         JOIN requests r ON r.id = e.request_id JOIN collections c ON c.id = r.collection_id
         WHERE c.workspace_id = ?1 ORDER BY e.sort_order, e.name"
    ))?;
    let rows = stmt.query_map([workspace_id.to_string()], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn summaries(conn: &Connection, workspace_id: Uuid) -> AppResult<Vec<ExampleSummary>> {
    Ok(list(conn, workspace_id)?
        .into_iter()
        .map(|e| ExampleSummary {
            id: e.id,
            request_id: e.request_id,
            name: e.name,
            status: e.status,
        })
        .collect())
}

pub fn get(conn: &Connection, id: Uuid) -> AppResult<ResponseExample> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM request_examples e WHERE e.id = ?1"),
        [id.to_string()],
        from_row,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("Contoh respons"),
        other => other.into(),
    })
}

pub fn insert(conn: &Connection, e: &ResponseExample) -> AppResult<()> {
    conn.execute(
        "INSERT INTO request_examples (id, request_id, name, status, status_text, headers_json, body, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            e.id.to_string(),
            e.request_id.to_string(),
            e.name,
            e.status,
            e.status_text,
            serde_json::to_string(&e.headers)?,
            e.body,
            e.sort_order
        ],
    )?;
    Ok(())
}

pub fn create(conn: &Connection, input: NewExample) -> AppResult<ResponseExample> {
    if input.body.len() > MAX_BODY_BYTES {
        return Err(AppError::Validation("Body contoh maksimal 1 MB".into()));
    }
    let sort_order: f64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM request_examples WHERE request_id = ?1",
        [input.request_id.to_string()],
        |row| row.get(0),
    )?;
    let example = ResponseExample {
        id: Uuid::now_v7(),
        request_id: input.request_id,
        name: validate_name(&input.name)?,
        status: input.status,
        status_text: input.status_text,
        headers: input
            .headers
            .into_iter()
            .filter(|(k, _)| !DROPPED_HEADERS.contains(&k.to_ascii_lowercase().as_str()))
            .collect(),
        body: input.body,
        sort_order,
    };
    insert(conn, &example).map_err(|e| match e {
        AppError::Db(rusqlite::Error::SqliteFailure(f, _))
            if f.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::Validation(
                "Simpan request ke koleksi sebelum menyimpan contoh respons".into(),
            )
        }
        other => other,
    })?;
    Ok(example)
}

pub fn rename(conn: &Connection, id: Uuid, name: &str) -> AppResult<()> {
    let n = conn.execute(
        "UPDATE request_examples SET name = ?1 WHERE id = ?2",
        params![validate_name(name)?, id.to_string()],
    )?;
    ensure_changed(n, "Contoh respons")
}

pub fn delete(conn: &Connection, id: Uuid) -> AppResult<()> {
    let n = conn.execute(
        "DELETE FROM request_examples WHERE id = ?1",
        [id.to_string()],
    )?;
    ensure_changed(n, "Contoh respons")
}
