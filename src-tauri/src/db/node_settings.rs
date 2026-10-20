//! Scripts, docs and variables that a collection or folder applies to everything inside it.
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use super::ensure_changed;
use crate::error::AppResult;
use crate::models::collection::NodeKind;
use crate::models::script::Scripts;

pub(super) fn scripts_from(json: Option<String>) -> Scripts {
    json.and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default()
}

pub(super) fn scripts_json(scripts: &Scripts) -> AppResult<Option<String>> {
    let empty = scripts.pre_request.trim().is_empty() && scripts.test.trim().is_empty();
    Ok(if empty {
        None
    } else {
        Some(serde_json::to_string(scripts)?)
    })
}

pub fn set(
    conn: &Connection,
    kind: NodeKind,
    id: Uuid,
    scripts: &Scripts,
    docs: &str,
) -> AppResult<()> {
    let sql = match kind {
        NodeKind::Collection => "UPDATE collections SET scripts_json = ?1, docs = ?2 WHERE id = ?3",
        NodeKind::Folder => "UPDATE folders SET scripts_json = ?1, docs = ?2 WHERE id = ?3",
        NodeKind::Request => return Ok(()),
    };
    let n = conn.execute(sql, params![scripts_json(scripts)?, docs, id.to_string()])?;
    ensure_changed(n, "Koleksi atau folder")
}

/// Replaces the collection variables, kept as `[key, value]` pairs sorted by key.
pub fn set_variables(
    conn: &Connection,
    collection_id: Uuid,
    mut variables: Vec<(String, String)>,
) -> AppResult<()> {
    variables.retain(|(k, _)| !k.trim().is_empty());
    for (k, _) in variables.iter_mut() {
        *k = k.trim().to_string();
    }
    variables.sort();
    variables.dedup_by(|a, b| a.0 == b.0);
    let n = conn.execute(
        "UPDATE collections SET variables_json = ?1 WHERE id = ?2",
        params![
            serde_json::to_string(&variables)?,
            collection_id.to_string()
        ],
    )?;
    ensure_changed(n, "Koleksi")
}

/// Scripts that run around a request, outermost first: the collection, then each folder
/// from the top down. Each entry carries a label for error messages.
pub fn chain(
    conn: &Connection,
    collection_id: Uuid,
    folder_id: Option<Uuid>,
) -> AppResult<Vec<(String, Scripts)>> {
    let mut folders = Vec::new();
    let mut current = folder_id;
    // The depth limit guards against a corrupt parent cycle.
    for _ in 0..64 {
        let Some(id) = current else { break };
        let row: Option<(String, Option<String>, Option<String>)> = conn
            .query_row(
                "SELECT name, scripts_json, parent_id FROM folders WHERE id = ?1",
                [id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let Some((name, scripts, parent)) = row else {
            break;
        };
        folders.push((format!("folder {name}"), scripts_from(scripts)));
        current = parent.and_then(|p| Uuid::parse_str(&p).ok());
    }
    let collection: Option<(String, Option<String>)> = conn
        .query_row(
            "SELECT name, scripts_json FROM collections WHERE id = ?1",
            [collection_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let mut out: Vec<(String, Scripts)> = collection
        .map(|(name, scripts)| (format!("koleksi {name}"), scripts_from(scripts)))
        .into_iter()
        .collect();
    out.extend(folders.into_iter().rev());
    Ok(out)
}
