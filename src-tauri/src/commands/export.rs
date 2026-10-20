use tauri::State;
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::environment::Environment;
use crate::models::import::ExportReport;
use crate::services::exporters::{environment, postman};
use crate::state::AppState;

const MAX_IMPORT_BYTES: u64 = 5 * 1024 * 1024;

fn write_json(path: &str, value: &serde_json::Value) -> AppResult<()> {
    std::fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

#[tauri::command]
pub fn export_collection(
    state: State<'_, AppState>,
    collection_id: Uuid,
    path: String,
) -> AppResult<ExportReport> {
    let conn = state.db()?;
    let collection = db::collections::list_collections(&conn, state.workspace_id)?
        .into_iter()
        .find(|c| c.id == collection_id)
        .ok_or(AppError::NotFound("Koleksi"))?;
    let folders: Vec<_> = db::folders::list_folders(&conn, state.workspace_id)?
        .into_iter()
        .filter(|f| f.collection_id == collection_id)
        .collect();
    let requests: Vec<_> = db::requests::list(&conn, state.workspace_id)?
        .into_iter()
        .filter(|r| r.collection_id == collection_id)
        .collect();
    let variables = db::collections::variables(&conn, collection_id)?;
    let exported = postman::export(&postman::CollectionData {
        collection: &collection,
        variables: &variables,
        folders: &folders,
        requests: &requests,
    });
    write_json(&path, &exported.json)?;
    Ok(ExportReport {
        requests: exported.requests,
        skipped: exported.skipped,
    })
}

#[tauri::command]
pub fn export_environment(
    state: State<'_, AppState>,
    environment_id: Uuid,
    path: String,
) -> AppResult<()> {
    let env = db::environments::list(&*state.db()?, state.workspace_id)?
        .into_iter()
        .find(|e| e.id == environment_id)
        .ok_or(AppError::NotFound("Environment"))?;
    write_json(&path, &environment::to_postman(&env))
}

/// Imports a Postman environment; secret values are moved to the keychain when saved.
#[tauri::command]
pub fn import_environment(state: State<'_, AppState>, path: String) -> AppResult<Environment> {
    if std::fs::metadata(&path)?.len() > MAX_IMPORT_BYTES {
        return Err(AppError::Validation("Ukuran file maksimal 5 MB".into()));
    }
    let env = environment::from_postman(&std::fs::read_to_string(&path)?)?;
    db::environments::save(
        &*state.db()?,
        state.workspace_id,
        env,
        state.secrets.as_ref(),
    )
}
