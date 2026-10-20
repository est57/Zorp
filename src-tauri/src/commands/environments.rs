use tauri::State;
use uuid::Uuid;

use crate::db::{self, environments};
use crate::error::AppResult;
use crate::models::environment::Environment;
use crate::state::AppState;

#[tauri::command]
pub fn list_environments(state: State<'_, AppState>) -> AppResult<Vec<Environment>> {
    environments::list(&*state.db()?, state.workspace_id())
}

#[tauri::command]
pub fn save_environment(
    state: State<'_, AppState>,
    environment: Environment,
) -> AppResult<Environment> {
    environments::save(
        &*state.db()?,
        state.workspace_id(),
        environment,
        state.secrets.as_ref(),
    )
}

#[tauri::command]
pub fn delete_environment(state: State<'_, AppState>, id: Uuid) -> AppResult<()> {
    environments::delete(&*state.db()?, id, state.secrets.as_ref())
}

#[tauri::command]
pub fn set_active_environment(state: State<'_, AppState>, id: Option<Uuid>) -> AppResult<()> {
    environments::set_active(&*state.db()?, state.workspace_id(), id)
}

#[tauri::command]
pub fn get_globals(state: State<'_, AppState>) -> AppResult<Environment> {
    db::globals::get(&*state.db()?, state.workspace_id())
}

/// Saves workspace-wide variables; secret values go to the keychain as for environments.
#[tauri::command]
pub fn save_globals(state: State<'_, AppState>, globals: Environment) -> AppResult<Environment> {
    db::globals::save(
        &*state.db()?,
        state.workspace_id(),
        globals,
        state.secrets.as_ref(),
    )
}
