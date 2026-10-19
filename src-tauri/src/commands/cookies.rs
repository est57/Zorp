use tauri::State;

use crate::error::AppResult;
use crate::models::cookie::CookieInfo;
use crate::state::AppState;

#[tauri::command]
pub fn list_cookies(state: State<'_, AppState>) -> Vec<CookieInfo> {
    state.cookies.list()
}

#[tauri::command]
pub fn delete_cookie(
    state: State<'_, AppState>,
    domain: String,
    path: String,
    name: String,
) -> AppResult<()> {
    state.cookies.remove(&domain, &path, &name);
    state.persist_cookies()
}

/// Clears every cookie, or only those of `domain`.
#[tauri::command]
pub fn clear_cookies(state: State<'_, AppState>, domain: Option<String>) -> AppResult<()> {
    state.cookies.clear(domain.as_deref());
    state.persist_cookies()
}
