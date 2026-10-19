use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::db::now_ms;
use crate::error::AppResult;
use crate::models::workspace::Workspace;

const DEFAULT_NAME: &str = "Workspace Saya";

pub fn first(conn: &Connection) -> AppResult<Option<Workspace>> {
    let row = conn
        .query_row(
            "SELECT id, name, created_at FROM workspaces ORDER BY created_at LIMIT 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;

    Ok(row.and_then(|(id, name, created_at)| {
        Uuid::parse_str(&id).ok().map(|id| Workspace {
            id,
            name,
            created_at,
        })
    }))
}

pub fn ensure_default(conn: &Connection) -> AppResult<Workspace> {
    if let Some(existing) = first(conn)? {
        return Ok(existing);
    }
    let workspace = Workspace {
        id: Uuid::now_v7(),
        name: DEFAULT_NAME.into(),
        created_at: now_ms(),
    };
    conn.execute(
        "INSERT INTO workspaces (id, name, created_at) VALUES (?1, ?2, ?3)",
        params![
            workspace.id.to_string(),
            workspace.name,
            workspace.created_at
        ],
    )?;
    Ok(workspace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn ensure_default_is_idempotent() {
        let conn = open_in_memory().unwrap();
        let first_call = ensure_default(&conn).unwrap();
        let second_call = ensure_default(&conn).unwrap();
        assert_eq!(first_call.id, second_call.id);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
