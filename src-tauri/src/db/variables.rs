use std::collections::HashMap;

use rusqlite::{Connection, params};
use uuid::Uuid;

use super::{collections, environments};
use crate::error::AppResult;
use crate::models::environment::Variable;
use crate::services::secrets::SecretStore;

/// Applies script changes to the active environment; secret values stay in the keychain.
pub fn apply_env_changes(
    conn: &Connection,
    workspace_id: Uuid,
    secrets: &dyn SecretStore,
    sets: &HashMap<String, String>,
    unsets: &[String],
) -> AppResult<()> {
    let Some(mut env) = environments::list(conn, workspace_id)?
        .into_iter()
        .find(|e| e.is_active)
    else {
        return Ok(());
    };
    env.variables.retain(|v| !unsets.contains(&v.key));
    for (key, value) in sets {
        match env.variables.iter_mut().find(|v| &v.key == key) {
            Some(var) => var.value = value.clone(),
            None => env.variables.push(Variable {
                key: key.clone(),
                value: value.clone(),
                enabled: true,
                secret: false,
                has_value: false,
            }),
        }
    }
    environments::save(conn, workspace_id, env, secrets)?;
    Ok(())
}

pub fn apply_collection_changes(
    conn: &Connection,
    collection_id: Uuid,
    sets: &HashMap<String, String>,
    unsets: &[String],
) -> AppResult<()> {
    let mut vars = collections::variables(conn, collection_id)?;
    vars.retain(|(k, _)| !unsets.contains(k) && !sets.contains_key(k));
    vars.extend(sets.iter().map(|(k, v)| (k.clone(), v.clone())));
    vars.sort();
    conn.execute(
        "UPDATE collections SET variables_json = ?1 WHERE id = ?2",
        params![serde_json::to_string(&vars)?, collection_id.to_string()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_in_memory, workspace};
    use crate::models::environment::Environment;
    use crate::services::secrets::{MemoryStore, env_account};

    #[test]
    fn script_changes_update_active_environment_and_keep_secrets_out_of_sqlite() {
        let conn = open_in_memory().unwrap();
        let ws = workspace::current(&conn).unwrap().id;
        let store = MemoryStore::default();
        let env = Environment {
            id: Uuid::now_v7(),
            name: "Dev".into(),
            variables: vec![
                Variable {
                    key: "token".into(),
                    value: "old".into(),
                    enabled: true,
                    secret: true,
                    has_value: false,
                },
                Variable {
                    key: "gone".into(),
                    value: "x".into(),
                    enabled: true,
                    secret: false,
                    has_value: false,
                },
            ],
            is_active: true,
        };
        environments::save(&conn, ws, env.clone(), &store).unwrap();
        environments::set_active(&conn, ws, Some(env.id)).unwrap();

        let sets = HashMap::from([
            ("token".to_string(), "fresh".to_string()),
            ("user".to_string(), "42".to_string()),
        ]);
        apply_env_changes(&conn, ws, &store, &sets, &["gone".to_string()]).unwrap();

        let values = environments::active_values(&conn, ws, &store).unwrap();
        assert!(values.contains(&("token".into(), "fresh".into())));
        assert!(values.contains(&("user".into(), "42".into())));
        assert!(!values.iter().any(|(k, _)| k == "gone"));
        assert_eq!(
            store.get(&env_account(env.id, "token")).unwrap().as_deref(),
            Some("fresh")
        );
        let raw: String = conn
            .query_row("SELECT variables_json FROM environments", [], |r| r.get(0))
            .unwrap();
        assert!(!raw.contains("fresh"));
    }
}
