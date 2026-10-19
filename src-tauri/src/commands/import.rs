use tauri::State;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::import::ImportReport;
use crate::services::importers::postman;
use crate::state::AppState;

const MAX_IMPORT_BYTES: u64 = 20 * 1024 * 1024;

#[tauri::command]
pub fn import_postman(state: State<'_, AppState>, path: String) -> AppResult<ImportReport> {
    let size = std::fs::metadata(&path)?.len();
    if size > MAX_IMPORT_BYTES {
        return Err(AppError::Validation("Ukuran file maksimal 20 MB".into()));
    }
    let json = std::fs::read_to_string(&path)?;
    let imported = postman::parse(&json)?;
    db::import::save(&*state.db()?, state.workspace_id, imported)
}
