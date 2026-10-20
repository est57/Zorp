use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::db::{environments, workspace};
use crate::error::{AppError, AppResult};
use crate::models::workspace::{Workspace, WorkspaceList};
use crate::services::client_pool::proxy_account;
use crate::services::secrets::{SecretStore, env_account};
use crate::state::AppState;
use crate::sync::{self, SyncEngine};

/// Makes a workspace active and binds sync to its folder, if it has one.
/// Returns true when data was read from the folder.
pub(super) fn open(
    app: &AppHandle,
    state: &AppState,
    engine: &SyncEngine,
    id: Uuid,
) -> AppResult<bool> {
    let folder = workspace::get(&*state.db()?, id)?
        .ok_or(AppError::NotFound("Workspace"))?
        .folder
        .map(std::path::PathBuf::from);
    state.activate(id)?;
    let watcher = folder.as_deref().and_then(|root| sync::watch(app, root));
    engine.attach(state, folder, watcher)
}

#[tauri::command]
pub fn list_workspaces(state: State<'_, AppState>) -> AppResult<WorkspaceList> {
    Ok(WorkspaceList {
        active_id: state.workspace_id(),
        workspaces: workspace::list(&*state.db()?)?,
    })
}

/// Creates an empty workspace stored in this app and makes it active.
#[tauri::command]
pub fn create_workspace(
    app: AppHandle,
    state: State<'_, AppState>,
    engine: State<'_, SyncEngine>,
    name: String,
) -> AppResult<Workspace> {
    let created = workspace::create(&*state.db()?, &name, None)?;
    open(&app, &state, &engine, created.id)?;
    Ok(created)
}

#[tauri::command]
pub fn switch_workspace(
    app: AppHandle,
    state: State<'_, AppState>,
    engine: State<'_, SyncEngine>,
    id: Uuid,
) -> AppResult<()> {
    open(&app, &state, &engine, id).map(|_| ())
}

#[tauri::command]
pub fn rename_workspace(state: State<'_, AppState>, id: Uuid, name: String) -> AppResult<()> {
    workspace::rename(&*state.db()?, id, &name)
}

/// Removes an inactive workspace with its keychain secrets; a linked folder is not touched.
#[tauri::command]
pub fn remove_workspace(state: State<'_, AppState>, id: Uuid) -> AppResult<()> {
    if id == state.workspace_id() {
        return Err(AppError::Validation(
            "Pindah ke workspace lain sebelum menghapus workspace ini".into(),
        ));
    }
    let conn = state.db()?;
    remove_secrets(&conn, state.secrets.as_ref(), id)?;
    workspace::delete(&conn, id)
}

fn remove_secrets(
    conn: &rusqlite::Connection,
    secrets: &dyn SecretStore,
    id: Uuid,
) -> AppResult<()> {
    let globals = crate::db::globals::get(conn, id)?;
    for env in environments::list(conn, id)?.into_iter().chain([globals]) {
        for var in env.variables.iter().filter(|v| v.secret) {
            secrets.delete(&env_account(env.id, &var.key))?;
        }
    }
    secrets.delete(&proxy_account(id))
}
