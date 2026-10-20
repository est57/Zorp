//! Workspace-wide variables. They are stored as a hidden environment row whose id is the
//! workspace id, so secrets reuse the environment keychain handling.
use std::collections::HashMap;

use rusqlite::{Connection, params};
use uuid::Uuid;

use super::environments;
use crate::error::AppResult;
use crate::models::environment::{Environment, Variable};
use crate::services::secrets::{SecretStore, env_account};

pub const NAME: &str = "Global";

pub fn get(conn: &Connection, workspace_id: Uuid) -> AppResult<Environment> {
    Ok(Environment {
        id: workspace_id,
        name: NAME.into(),
        variables: environments::stored_variables(conn, workspace_id)?,
        is_active: false,
    })
}

pub fn save(
    conn: &Connection,
    workspace_id: Uuid,
    mut globals: Environment,
    secrets: &dyn SecretStore,
) -> AppResult<Environment> {
    globals.id = workspace_id;
    globals.name = NAME.into();
    globals.is_active = false;
    let saved = environments::save(conn, workspace_id, globals, secrets)?;
    conn.execute(
        "UPDATE environments SET is_global = 1, is_active = 0 WHERE id = ?1",
        params![workspace_id.to_string()],
    )?;
    Ok(saved)
}

pub fn values(
    conn: &Connection,
    workspace_id: Uuid,
    secrets: &dyn SecretStore,
) -> AppResult<Vec<(String, String)>> {
    environments::values_of(get(conn, workspace_id)?, secrets)
}

/// Replaces the globals with those read from a workspace folder, keeping keychain values
/// for secrets that are still declared. Returns keychain accounts to remove.
pub(super) fn replace(
    tx: &Connection,
    workspace_id: Uuid,
    incoming: &[Variable],
    previous: &[Variable],
) -> AppResult<Vec<String>> {
    if incoming.is_empty() && previous.is_empty() {
        return Ok(Vec::new());
    }
    let variables: Vec<Variable> = incoming
        .iter()
        .map(|v| Variable {
            has_value: v.secret
                && previous
                    .iter()
                    .any(|p| p.secret && p.key == v.key && p.has_value),
            value: if v.secret {
                String::new()
            } else {
                v.value.clone()
            },
            ..v.clone()
        })
        .collect();
    let ws = workspace_id.to_string();
    tx.execute(
        "INSERT INTO environments (id, workspace_id, name, variables_json, is_active, is_global)
         VALUES (?1, ?1, ?2, ?3, 0, 1)
         ON CONFLICT(id) DO UPDATE SET variables_json = excluded.variables_json",
        params![ws, NAME, serde_json::to_string(&variables)?],
    )?;
    Ok(previous
        .iter()
        .filter(|p| p.secret && !incoming.iter().any(|v| v.secret && v.key == p.key))
        .map(|p| env_account(workspace_id, &p.key))
        .collect())
}

/// Applies `pm.globals` changes made by a script.
pub fn apply_changes(
    conn: &Connection,
    workspace_id: Uuid,
    secrets: &dyn SecretStore,
    sets: &HashMap<String, String>,
    unsets: &[String],
) -> AppResult<()> {
    if sets.is_empty() && unsets.is_empty() {
        return Ok(());
    }
    let mut globals = get(conn, workspace_id)?;
    globals.variables.retain(|v| !unsets.contains(&v.key));
    for (key, value) in sets {
        match globals.variables.iter_mut().find(|v| &v.key == key) {
            Some(var) => var.value = value.clone(),
            None => globals.variables.push(Variable {
                key: key.clone(),
                value: value.clone(),
                enabled: true,
                secret: false,
                has_value: false,
            }),
        }
    }
    save(conn, workspace_id, globals, secrets)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_in_memory, workspace};
    use crate::services::secrets::MemoryStore;

    #[test]
    fn globals_are_hidden_from_environments_and_keep_secrets() {
        let conn = open_in_memory().unwrap();
        let ws = workspace::current(&conn).unwrap().id;
        let store = MemoryStore::default();
        let var = |key: &str, value: &str, secret| Variable {
            key: key.into(),
            value: value.into(),
            enabled: true,
            secret,
            has_value: false,
        };
        let globals = Environment {
            id: Uuid::nil(),
            name: "x".into(),
            variables: vec![var("app", "zorp", false), var("key", "s3cret", true)],
            is_active: true,
        };
        save(&conn, ws, globals, &store).unwrap();
        assert!(environments::list(&conn, ws).unwrap().is_empty());
        let loaded = get(&conn, ws).unwrap();
        assert_eq!((loaded.id, loaded.name.as_str()), (ws, NAME));
        assert_eq!(
            loaded.variables[1].value, "",
            "nilai rahasia tidak di SQLite"
        );
        let mut sets = HashMap::new();
        sets.insert("app".to_string(), "zorp2".to_string());
        apply_changes(&conn, ws, &store, &sets, &[]).unwrap();
        assert_eq!(
            values(&conn, ws, &store).unwrap(),
            [
                ("app".to_string(), "zorp2".to_string()),
                ("key".to_string(), "s3cret".to_string())
            ]
        );
    }
}
