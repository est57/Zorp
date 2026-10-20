use std::path::Path;

use tauri::{AppHandle, State};

use super::workspaces::open;
use crate::db::workspace;
use crate::error::{AppError, AppResult};
use crate::models::sync::SyncStatus;
use crate::models::workspace::Workspace;
use crate::services::workspace_sync::{ROOT_FILE, Snapshot, parse};
use crate::state::AppState;
use crate::sync::{self, SyncEngine};

#[tauri::command]
pub fn sync_status(engine: State<'_, SyncEngine>) -> SyncStatus {
    engine.status()
}

/// Opens a folder as a workspace. A folder already known to the app is switched to; a
/// workspace folder (with `zorp.yaml`) is added; an empty folder starts a new workspace.
#[tauri::command]
pub fn open_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    engine: State<'_, SyncEngine>,
    path: String,
) -> AppResult<Workspace> {
    let root = sync::canonical(&path)?;
    let folder = root.to_string_lossy().into_owned();
    let existing = workspace::find_by_folder(&*state.db()?, &folder)?;
    if let Some(known) = existing {
        open(&app, &state, &engine, known.id)?;
        return Ok(known);
    }
    let name = if root.join(ROOT_FILE).is_file() {
        // Check the files before anything is created, so a broken folder adds nothing.
        let snapshot = parse::read(&root).map_err(|p| {
            AppError::Validation(format!("Folder tidak bisa dibaca:\n{}", p.0.join("\n")))
        })?;
        if let Some(owner) = owner(&state, &snapshot)? {
            return reconnect(&app, &state, &engine, owner, &folder);
        }
        snapshot.name
    } else {
        folder_name(&root)
    };
    let previous = state.workspace_id();
    let created = workspace::create(&*state.db()?, &name, Some(&folder))?;
    let outcome = open(&app, &state, &engine, created.id).and_then(|_| {
        let problems = engine.status().problems;
        if problems.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation(format!(
                "Folder tidak bisa dibuka:\n{}",
                problems.join("\n")
            )))
        }
    });
    if let Err(e) = outcome {
        open(&app, &state, &engine, previous)?;
        workspace::delete(&*state.db()?, created.id)?;
        return Err(e);
    }
    Ok(created)
}

/// Stores the active workspace in an empty folder and keeps the two in sync from now on.
#[tauri::command]
pub fn link_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    engine: State<'_, SyncEngine>,
    path: String,
) -> AppResult<()> {
    let root = sync::canonical(&path)?;
    let folder = root.to_string_lossy().into_owned();
    let id = state.workspace_id();
    {
        let conn = state.db()?;
        if workspace::find_by_folder(&conn, &folder)?.is_some() {
            return Err(AppError::Validation(
                "Folder ini sudah dipakai workspace lain".into(),
            ));
        }
        if root.join(ROOT_FILE).exists() {
            return Err(AppError::Validation(
                "Folder ini sudah berisi workspace; gunakan Buka folder".into(),
            ));
        }
        workspace::set_folder(&conn, id, Some(&folder))?;
    }
    open(&app, &state, &engine, id)?;
    let problems = engine.status().problems;
    if !problems.is_empty() {
        workspace::set_folder(&*state.db()?, id, None)?;
        open(&app, &state, &engine, id)?;
        return Err(AppError::Validation(problems.join("\n")));
    }
    Ok(())
}

/// Shows the active workspace folder in the system file manager.
#[tauri::command]
pub fn reveal_folder(engine: State<'_, SyncEngine>) -> AppResult<()> {
    let folder = engine
        .status()
        .folder
        .ok_or_else(|| AppError::Validation("Workspace ini belum disimpan ke folder".into()))?;
    let root = Path::new(&folder);
    let target = if root.join(ROOT_FILE).is_file() {
        root.join(ROOT_FILE)
    } else {
        root.to_path_buf()
    };
    tauri_plugin_opener::reveal_item_in_dir(target)
        .map_err(|e| AppError::Internal(format!("Folder tidak bisa ditampilkan: {e}")))
}

/// Stops syncing the active workspace; the files in the folder are kept.
#[tauri::command]
pub fn unlink_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    engine: State<'_, SyncEngine>,
) -> AppResult<()> {
    let id = state.workspace_id();
    workspace::set_folder(&*state.db()?, id, None)?;
    open(&app, &state, &engine, id).map(|_| ())
}

fn owner(state: &AppState, snapshot: &Snapshot) -> AppResult<Option<Workspace>> {
    let ids: Vec<_> = snapshot
        .collections
        .iter()
        .map(|c| c.collection.id)
        .chain(snapshot.environments.iter().map(|e| e.id))
        .collect();
    workspace::owner_of(&*state.db()?, &ids)
}

/// Links a folder back to the workspace its files came from. The folder is read as the
/// shared copy, so it replaces that workspace's collections and environments.
fn reconnect(
    app: &AppHandle,
    state: &AppState,
    engine: &SyncEngine,
    owner: Workspace,
    folder: &str,
) -> AppResult<Workspace> {
    if let Some(linked) = &owner.folder {
        return Err(AppError::Validation(format!(
            "Isi folder ini milik workspace {} yang tersambung ke {linked}",
            owner.name
        )));
    }
    workspace::set_folder(&*state.db()?, owner.id, Some(folder))?;
    open(app, state, engine, owner.id)?;
    Ok(Workspace {
        folder: Some(folder.to_string()),
        ..owner
    })
}

fn folder_name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "Workspace".into())
}
