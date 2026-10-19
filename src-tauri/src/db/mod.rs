pub mod collections;
pub mod cookies;
pub mod environments;
pub mod folders;
pub mod history;
pub mod import;
pub mod requests;
pub mod workspace;

#[cfg(test)]
mod tree_tests;

use std::path::Path;

use rusqlite::{Connection, Row};
use rusqlite_migration::{M, Migrations};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(include_str!("../../migrations/0001_init.sql")),
    M::up(include_str!("../../migrations/0002_cookies.sql")),
];
const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);
const MAX_NAME_LEN: usize = 200;

pub fn open(path: &Path) -> AppResult<Connection> {
    prepare(Connection::open(path)?)
}

#[cfg(test)]
pub fn open_in_memory() -> AppResult<Connection> {
    prepare(Connection::open_in_memory()?)
}

pub fn schema_version(conn: &Connection) -> AppResult<i64> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

fn prepare(mut conn: Connection) -> AppResult<Connection> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    MIGRATIONS.to_latest(&mut conn)?;
    Ok(conn)
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

/// Reads a TEXT column as a UUID.
pub fn uuid_at(row: &Row<'_>, idx: usize) -> rusqlite::Result<Uuid> {
    let text: String = row.get(idx)?;
    Uuid::parse_str(&text).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(idx, rusqlite::types::Type::Text, Box::new(e))
    })
}

pub fn opt_uuid_at(row: &Row<'_>, idx: usize) -> rusqlite::Result<Option<Uuid>> {
    let text: Option<String> = row.get(idx)?;
    text.map(|t| Uuid::parse_str(&t)).transpose().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(idx, rusqlite::types::Type::Text, Box::new(e))
    })
}

pub fn validate_name(name: &str) -> AppResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("Nama tidak boleh kosong".into()));
    }
    if trimmed.chars().count() > MAX_NAME_LEN {
        return Err(AppError::Validation(format!(
            "Nama maksimal {MAX_NAME_LEN} karakter"
        )));
    }
    Ok(trimmed.to_string())
}

pub fn ensure_changed(affected: usize, what: &'static str) -> AppResult<()> {
    if affected == 0 {
        Err(AppError::NotFound(what))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_valid() {
        MIGRATIONS.validate().expect("migrations should be valid");
    }

    #[test]
    fn open_applies_latest_schema() {
        let conn = open_in_memory().unwrap();
        assert_eq!(
            schema_version(&conn).unwrap(),
            MIGRATIONS_SLICE.len() as i64
        );
    }

    #[test]
    fn validate_name_trims_and_rejects_empty() {
        assert_eq!(validate_name("  Users ").unwrap(), "Users");
        assert!(validate_name("   ").is_err());
    }
}
