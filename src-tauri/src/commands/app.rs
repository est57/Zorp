use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::app_info::AppInfo;
use crate::state::AppState;

#[tauri::command]
pub fn app_info(app: AppHandle, state: State<'_, AppState>) -> AppResult<AppInfo> {
    let conn = state.db()?;
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        schema_version: db::schema_version(&conn)?,
        workspace: db::workspace::get(&conn, state.workspace_id())?
            .ok_or(AppError::NotFound("Workspace"))?,
    })
}

/// Opens a web link from rendered docs in the default browser. Only http and https are
/// accepted, so a document cannot launch other applications.
#[tauri::command]
pub fn open_link(app: AppHandle, url: String) -> AppResult<()> {
    let parsed = url::Url::parse(&url).map_err(|e| AppError::InvalidUrl(e.to_string()))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::Validation(
            "Hanya link http dan https yang bisa dibuka".into(),
        ));
    }
    app.opener()
        .open_url(parsed.as_str(), None::<&str>)
        .map_err(|e| AppError::Internal(format!("Browser tidak bisa dibuka: {e}")))
}
