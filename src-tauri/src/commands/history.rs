use tauri::State;

use crate::db::history;
use crate::error::AppResult;
use crate::models::history::HistoryPage;
use crate::state::AppState;

#[tauri::command]
pub fn list_history(
    state: State<'_, AppState>,
    cursor: Option<i64>,
    limit: u32,
) -> AppResult<HistoryPage> {
    history::list(&*state.db()?, state.workspace_id(), cursor, limit)
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> AppResult<()> {
    history::clear(&*state.db()?, state.workspace_id())
}
