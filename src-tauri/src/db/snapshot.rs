use std::collections::HashMap;

use rusqlite::{Connection, params};
use uuid::Uuid;

use super::node_settings::scripts_json;
use super::{collections, environments, examples, folders, now_ms, requests, workspace};
use crate::error::{AppError, AppResult};
use crate::models::environment::Variable;
use crate::services::secrets::{SecretStore, env_account};
use crate::services::workspace_sync::{CollectionEntry, Snapshot};

/// The shared part of a workspace as stored in this app.
pub fn load(conn: &Connection, workspace_id: Uuid) -> AppResult<Snapshot> {
    let name = workspace::get(conn, workspace_id)?
        .ok_or(AppError::NotFound("Workspace"))?
        .name;
    let collections = collections::list_collections(conn, workspace_id)?
        .into_iter()
        .map(|collection| {
            let variables = collections::variables(conn, collection.id)?;
            Ok(CollectionEntry {
                collection,
                variables,
            })
        })
        .collect::<AppResult<_>>()?;
    Ok(Snapshot {
        name,
        collections,
        folders: folders::list_folders(conn, workspace_id)?,
        requests: requests::list(conn, workspace_id)?,
        examples: examples::list(conn, workspace_id)?,
        environments: environments::list(conn, workspace_id)?,
    })
}

fn conflict(e: rusqlite::Error) -> AppError {
    match e {
        rusqlite::Error::SqliteFailure(f, _)
            if f.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::Validation(
                "Folder ini berisi id yang sudah dipakai workspace lain di aplikasi ini".into(),
            )
        }
        other => other.into(),
    }
}

/// Replaces collections and environments with the snapshot in one transaction.
/// Environment ids, keychain secrets and the active environment are kept where they still apply.
pub fn replace(
    conn: &Connection,
    workspace_id: Uuid,
    snapshot: &Snapshot,
    secrets: &dyn SecretStore,
) -> AppResult<()> {
    let ws = workspace_id.to_string();
    let previous: HashMap<Uuid, (Vec<Variable>, bool)> = environments::list(conn, workspace_id)?
        .into_iter()
        .map(|e| (e.id, (e.variables, e.is_active)))
        .collect();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE workspaces SET name = ?1 WHERE id = ?2",
        params![snapshot.name, ws],
    )?;
    tx.execute("DELETE FROM collections WHERE workspace_id = ?1", [&ws])?;
    for entry in &snapshot.collections {
        let c = &entry.collection;
        tx.execute(
            "INSERT INTO collections
               (id, workspace_id, name, auth_json, variables_json, sort_order, scripts_json, docs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                c.id.to_string(),
                ws,
                c.name,
                c.auth.as_ref().map(serde_json::to_string).transpose()?,
                serde_json::to_string(&entry.variables)?,
                c.sort_order,
                scripts_json(&c.scripts)?,
                c.docs
            ],
        )
        .map_err(conflict)?;
    }
    // Parents first, so every folder's parent already exists when it is inserted.
    let mut pending: Vec<_> = snapshot.folders.iter().collect();
    let mut inserted = std::collections::HashSet::new();
    while !pending.is_empty() {
        let before = pending.len();
        let (ready, rest): (Vec<_>, Vec<_>) = pending
            .into_iter()
            .partition(|f| f.parent_id.is_none_or(|p| inserted.contains(&p)));
        for f in ready {
            tx.execute(
                "INSERT INTO folders
                   (id, collection_id, parent_id, name, auth_json, sort_order, scripts_json, docs)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    f.id.to_string(),
                    f.collection_id.to_string(),
                    f.parent_id.map(|p| p.to_string()),
                    f.name,
                    f.auth.as_ref().map(serde_json::to_string).transpose()?,
                    f.sort_order,
                    scripts_json(&f.scripts)?,
                    f.docs
                ],
            )
            .map_err(conflict)?;
            inserted.insert(f.id);
        }
        pending = rest;
        if pending.len() == before {
            return Err(AppError::Validation("Struktur folder tidak valid".into()));
        }
    }
    for r in &snapshot.requests {
        tx.execute(
            "INSERT INTO requests (id, collection_id, folder_id, spec_json, sort_order, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                r.spec.id.to_string(),
                r.collection_id.to_string(),
                r.folder_id.map(|f| f.to_string()),
                serde_json::to_string(&r.spec)?,
                r.sort_order,
                now_ms()
            ],
        )
        .map_err(conflict)?;
    }
    for example in &snapshot.examples {
        examples::insert(&tx, example).map_err(|e| match e {
            AppError::Db(inner) => conflict(inner),
            other => other,
        })?;
    }
    let stale = replace_environments(&tx, &ws, snapshot, &previous)?;
    tx.commit()?;
    for account in stale {
        secrets.delete(&account)?;
    }
    Ok(())
}

/// Upserts environments and returns keychain accounts that no longer belong to any variable.
fn replace_environments(
    tx: &Connection,
    ws: &str,
    snapshot: &Snapshot,
    previous: &HashMap<Uuid, (Vec<Variable>, bool)>,
) -> AppResult<Vec<String>> {
    let mut stale = Vec::new();
    for (id, (vars, _)) in previous {
        let kept = snapshot.environments.iter().find(|e| e.id == *id);
        for old in vars.iter().filter(|v| v.secret) {
            let still =
                kept.is_some_and(|e| e.variables.iter().any(|v| v.secret && v.key == old.key));
            if !still {
                stale.push(env_account(*id, &old.key));
            }
        }
        if kept.is_none() {
            tx.execute("DELETE FROM environments WHERE id = ?1", [id.to_string()])?;
        }
    }
    for env in &snapshot.environments {
        let (old_vars, is_active) = previous.get(&env.id).cloned().unwrap_or_default();
        let variables: Vec<Variable> = env
            .variables
            .iter()
            .map(|v| Variable {
                has_value: v.secret
                    && old_vars
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
        let changed = tx.execute(
            "INSERT INTO environments (id, workspace_id, name, variables_json, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, variables_json = excluded.variables_json
             WHERE environments.workspace_id = excluded.workspace_id",
            params![env.id.to_string(), ws, env.name, serde_json::to_string(&variables)?, is_active],
        )
        .map_err(conflict)?;
        if changed == 0 {
            return Err(conflict(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                None,
            )));
        }
    }
    Ok(stale)
}

#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod tests;
