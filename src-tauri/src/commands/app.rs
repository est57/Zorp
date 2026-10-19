use tauri::{AppHandle, State};

use crate::db;
use crate::error::AppResult;
use crate::models::app_info::AppInfo;
use crate::state::AppState;

#[tauri::command]
pub fn app_info(app: AppHandle, state: State<'_, AppState>) -> AppResult<AppInfo> {
    let conn = state.db()?;
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        schema_version: db::schema_version(&conn)?,
        workspace: db::workspace::ensure_default(&conn)?,
    })
}
