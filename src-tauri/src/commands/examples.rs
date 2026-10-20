use tauri::State;
use uuid::Uuid;

use crate::db::examples;
use crate::error::AppResult;
use crate::models::example::{NewExample, ResponseExample};
use crate::state::AppState;

/// Saves a response as an example of a stored request.
#[tauri::command]
pub fn save_example(state: State<'_, AppState>, example: NewExample) -> AppResult<ResponseExample> {
    examples::create(&*state.db()?, example)
}

#[tauri::command]
pub fn get_example(state: State<'_, AppState>, id: Uuid) -> AppResult<ResponseExample> {
    examples::get(&*state.db()?, id)
}

#[tauri::command]
pub fn rename_example(state: State<'_, AppState>, id: Uuid, name: String) -> AppResult<()> {
    examples::rename(&*state.db()?, id, &name)
}

#[tauri::command]
pub fn delete_example(state: State<'_, AppState>, id: Uuid) -> AppResult<()> {
    examples::delete(&*state.db()?, id)
}
