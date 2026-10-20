use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::runner::{RunDataPreview, RunSummary, RunnerEvent, RunnerOptions};
use crate::pipeline::runner::run_items;
use crate::services::run_data;
use crate::state::AppState;

const RUNNER_EVENT: &str = "runner://item";
const PREVIEW_ROWS: usize = 5;

/// Runs every request of a collection or folder in order, emitting a result per request.
#[tauri::command]
pub async fn run_collection(
    app: AppHandle,
    state: State<'_, AppState>,
    run_id: Uuid,
    collection_id: Uuid,
    folder_id: Option<Uuid>,
    options: RunnerOptions,
) -> AppResult<RunSummary> {
    let items = db::run_order::ordered_requests(
        &*state.db()?,
        state.workspace_id(),
        collection_id,
        folder_id,
    )?;
    if items.is_empty() {
        return Err(AppError::Validation(
            "Tidak ada request untuk dijalankan".into(),
        ));
    }
    let data = match &options.data_path {
        Some(path) if !path.trim().is_empty() => run_data::load(path)?.rows,
        _ => Vec::new(),
    };
    let cancelled = state.register(run_id)?;
    let emit = |item| {
        let _ = app.emit(RUNNER_EVENT, RunnerEvent { run_id, item });
    };
    let summary = run_items(&state, &items, &data, &options, cancelled, emit).await;
    state.finish(run_id);
    Ok(summary)
}

/// Columns and first rows of an iteration data file.
#[tauri::command]
pub fn preview_run_data(path: String) -> AppResult<RunDataPreview> {
    let data = run_data::load(&path)?;
    Ok(RunDataPreview {
        rows: data
            .rows
            .iter()
            .take(PREVIEW_ROWS)
            .map(|row| {
                data.columns
                    .iter()
                    .map(|c| {
                        row.iter()
                            .find(|(k, _)| k == c)
                            .map(|(_, v)| v.clone())
                            .unwrap_or_default()
                    })
                    .collect()
            })
            .collect(),
        total: data.rows.len() as u32,
        columns: data.columns,
    })
}

/// Writes a run report chosen through the save dialog; only .json and .html are accepted.
#[tauri::command]
pub fn save_run_report(path: String, contents: String) -> AppResult<()> {
    let ext = std::path::Path::new(&path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if ext != "json" && ext != "html" {
        return Err(AppError::Validation(
            "Laporan hanya bisa disimpan sebagai .json atau .html".into(),
        ));
    }
    std::fs::write(path, contents)?;
    Ok(())
}

#[tauri::command]
pub fn cancel_run(state: State<'_, AppState>, run_id: Uuid) -> AppResult<()> {
    state.cancel(run_id)
}
