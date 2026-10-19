use rusqlite::{Connection, params};
use uuid::Uuid;

use super::{now_ms, uuid_at};
use crate::error::AppResult;
use crate::models::history::{HistoryItem, HistoryPage};
use crate::models::request::RequestSpec;

const MAX_ITEMS: i64 = 500;
const MAX_PAGE: u32 = 200;

pub struct NewEntry<'a> {
    pub request: &'a RequestSpec,
    pub status: Option<u16>,
    pub duration_ms: Option<u32>,
    pub size_bytes: Option<u64>,
}

pub fn insert(conn: &Connection, workspace_id: Uuid, entry: NewEntry<'_>) -> AppResult<()> {
    conn.execute(
        "INSERT INTO history (id, workspace_id, request_json, status, duration_ms, size_bytes, sent_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            Uuid::now_v7().to_string(),
            workspace_id.to_string(),
            serde_json::to_string(entry.request)?,
            entry.status,
            entry.duration_ms,
            entry.size_bytes.map(|s| s as i64),
            now_ms()
        ],
    )?;
    conn.execute(
        "DELETE FROM history WHERE workspace_id = ?1 AND id NOT IN (
           SELECT id FROM history WHERE workspace_id = ?1 ORDER BY sent_at DESC LIMIT ?2)",
        params![workspace_id.to_string(), MAX_ITEMS],
    )?;
    Ok(())
}

pub fn list(
    conn: &Connection,
    workspace_id: Uuid,
    cursor: Option<i64>,
    limit: u32,
) -> AppResult<HistoryPage> {
    let limit = limit.clamp(1, MAX_PAGE);
    let mut stmt = conn.prepare(
        "SELECT id, request_json, status, duration_ms, size_bytes, sent_at FROM history
         WHERE workspace_id = ?1 AND sent_at < ?2 ORDER BY sent_at DESC LIMIT ?3",
    )?;
    let rows = stmt.query_map(
        params![
            workspace_id.to_string(),
            cursor.unwrap_or(i64::MAX),
            limit + 1
        ],
        |row| {
            Ok((
                uuid_at(row, 0)?,
                row.get::<_, String>(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get(5)?,
            ))
        },
    )?;
    let mut items = Vec::new();
    for row in rows {
        let (id, json, status, duration_ms, size, sent_at) = row?;
        if let Ok(request) = serde_json::from_str(&json) {
            items.push(HistoryItem {
                id,
                request,
                status,
                duration_ms,
                size_bytes: size.map(|s| s as u64),
                sent_at,
            });
        }
    }
    let next_cursor = if items.len() > limit as usize {
        items.truncate(limit as usize);
        items.last().map(|i| i.sent_at)
    } else {
        None
    };
    Ok(HistoryPage { items, next_cursor })
}

pub fn clear(conn: &Connection, workspace_id: Uuid) -> AppResult<()> {
    conn.execute(
        "DELETE FROM history WHERE workspace_id = ?1",
        [workspace_id.to_string()],
    )?;
    Ok(())
}
