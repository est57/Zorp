use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use super::{ensure_changed, uuid_at, validate_name};
use crate::error::AppResult;
use crate::models::environment::{Environment, Variable};
use crate::services::secrets::{SecretStore, env_account};

pub fn list(conn: &Connection, workspace_id: Uuid) -> AppResult<Vec<Environment>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, variables_json, is_active FROM environments
         WHERE workspace_id = ?1 AND is_global = 0 ORDER BY name",
    )?;
    let rows = stmt.query_map([workspace_id.to_string()], |row| {
        Ok((
            uuid_at(row, 0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get(3)?,
        ))
    })?;
    let mut items = Vec::new();
    for row in rows {
        let (id, name, json, is_active) = row?;
        let variables = serde_json::from_str(&json).unwrap_or_default();
        items.push(Environment {
            id,
            name,
            variables,
            is_active,
        });
    }
    Ok(items)
}

pub(super) fn stored_variables(conn: &Connection, id: Uuid) -> AppResult<Vec<Variable>> {
    let json: Option<String> = conn
        .query_row(
            "SELECT variables_json FROM environments WHERE id = ?1",
            [id.to_string()],
            |row| row.get(0),
        )
        .optional()?;
    Ok(json
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default())
}

/// Saves an environment. Secret values go to the keychain; an empty secret keeps the stored value.
pub fn save(
    conn: &Connection,
    workspace_id: Uuid,
    mut env: Environment,
    secrets: &dyn SecretStore,
) -> AppResult<Environment> {
    env.name = validate_name(&env.name)?;
    let previous = stored_variables(conn, env.id)?;
    env.variables.retain(|v| !v.key.trim().is_empty());

    for var in &mut env.variables {
        var.key = var.key.trim().to_string();
        if !var.secret {
            var.has_value = false;
            continue;
        }
        let account = env_account(env.id, &var.key);
        if !var.value.is_empty() {
            secrets.set(&account, &var.value)?;
            var.has_value = true;
        } else {
            var.has_value = previous
                .iter()
                .any(|p| p.secret && p.key == var.key && p.has_value);
        }
        var.value.clear();
    }

    for old in previous.iter().filter(|p| p.secret) {
        let still_secret = env.variables.iter().any(|v| v.secret && v.key == old.key);
        if !still_secret {
            secrets.delete(&env_account(env.id, &old.key))?;
        }
    }

    conn.execute(
        "INSERT INTO environments (id, workspace_id, name, variables_json, is_active)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, variables_json = excluded.variables_json",
        params![
            env.id.to_string(),
            workspace_id.to_string(),
            env.name,
            serde_json::to_string(&env.variables)?,
            env.is_active
        ],
    )?;
    Ok(env)
}

pub fn delete(conn: &Connection, id: Uuid, secrets: &dyn SecretStore) -> AppResult<()> {
    for var in stored_variables(conn, id)?.iter().filter(|v| v.secret) {
        secrets.delete(&env_account(id, &var.key))?;
    }
    let n = conn.execute("DELETE FROM environments WHERE id = ?1", [id.to_string()])?;
    ensure_changed(n, "Environment")
}

pub fn set_active(conn: &Connection, workspace_id: Uuid, id: Option<Uuid>) -> AppResult<()> {
    conn.execute(
        "UPDATE environments SET is_active = (id IS ?1) WHERE workspace_id = ?2",
        params![id.map(|i| i.to_string()), workspace_id.to_string()],
    )?;
    Ok(())
}

/// Enabled variables of the active environment with secrets loaded from the keychain.
pub fn active_values(
    conn: &Connection,
    workspace_id: Uuid,
    secrets: &dyn SecretStore,
) -> AppResult<Vec<(String, String)>> {
    match list(conn, workspace_id)?.into_iter().find(|e| e.is_active) {
        Some(env) => values_of(env, secrets),
        None => Ok(Vec::new()),
    }
}

/// Enabled variables of an environment, with secrets loaded from the keychain.
pub fn values_of(env: Environment, secrets: &dyn SecretStore) -> AppResult<Vec<(String, String)>> {
    let mut values = Vec::new();
    for var in env.variables.into_iter().filter(|v| v.enabled) {
        let value = if var.secret {
            match secrets.get(&env_account(env.id, &var.key))? {
                Some(value) => value,
                None => continue,
            }
        } else {
            var.value
        };
        values.push((var.key, value));
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_in_memory, workspace};
    use crate::services::secrets::MemoryStore;

    fn var(key: &str, value: &str, secret: bool) -> Variable {
        Variable {
            key: key.into(),
            value: value.into(),
            enabled: true,
            secret,
            has_value: false,
        }
    }

    #[test]
    fn secrets_stay_out_of_sqlite_and_resolve_when_active() {
        let conn = open_in_memory().unwrap();
        let ws = workspace::current(&conn).unwrap().id;
        let store = MemoryStore::default();
        let env = Environment {
            id: Uuid::now_v7(),
            name: "Prod".into(),
            variables: vec![
                var("base", "https://api", false),
                var("token", "s3cret", true),
            ],
            is_active: false,
        };
        let saved = save(&conn, ws, env.clone(), &store).unwrap();
        assert!(saved.variables[1].value.is_empty() && saved.variables[1].has_value);

        let raw: String = conn
            .query_row("SELECT variables_json FROM environments", [], |r| r.get(0))
            .unwrap();
        assert!(!raw.contains("s3cret"));

        // Saving again with an empty secret keeps the stored value.
        save(&conn, ws, saved, &store).unwrap();
        set_active(&conn, ws, Some(env.id)).unwrap();
        let values = active_values(&conn, ws, &store).unwrap();
        assert!(values.contains(&("token".into(), "s3cret".into())));

        delete(&conn, env.id, &store).unwrap();
        assert!(store.get(&env_account(env.id, "token")).unwrap().is_none());
    }
}
