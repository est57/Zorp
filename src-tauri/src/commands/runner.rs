use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::collection::SavedRequest;
use crate::models::runner::{
    RunDataPreview, RunItemResult, RunSummary, RunnerEvent, RunnerOptions,
};
use crate::pipeline;
use crate::services::run_data;
use crate::state::AppState;

const RUNNER_EVENT: &str = "runner://item";
const PREVIEW_ROWS: usize = 5;

async fn run_one(
    state: &AppState,
    item: &SavedRequest,
    data: &[(String, String)],
    (index, total, iteration): (u32, u32, u32),
) -> RunItemResult {
    let request_id = Uuid::now_v7();
    let (_, result) = pipeline::execute_with_data(
        state,
        item.spec.clone(),
        data.to_vec(),
        request_id,
        |_, _| {},
    )
    .await;
    let (status, duration_ms, tests, error) = match result {
        Ok(response) => {
            let errors = response.script.errors.join("; ");
            let error = (!errors.is_empty()).then_some(errors);
            (
                Some(response.status),
                Some(response.timing.total_ms),
                response.script.tests,
                error,
            )
        }
        Err(e) => (None, None, Vec::new(), Some(e.to_string())),
    };
    RunItemResult {
        index,
        total,
        iteration,
        iteration_label: data.first().map(|(_, v)| v.clone()),
        request_id: item.spec.id,
        name: item.spec.name.clone(),
        method: item.spec.method,
        status,
        duration_ms,
        passed: error.is_none() && tests.iter().all(|t| t.passed),
        tests,
        error,
    }
}

/// Runs the items once per data row (once without data) until done, cancelled, or a
/// failure when `stop_on_failure` is set.
pub(crate) async fn run_items(
    state: &AppState,
    items: &[SavedRequest],
    data: &[Vec<(String, String)>],
    options: &RunnerOptions,
    mut cancelled: oneshot::Receiver<()>,
    emit: impl Fn(RunItemResult),
) -> RunSummary {
    let no_data = [Vec::new()];
    let rows = if data.is_empty() { &no_data[..] } else { data };
    let total = (items.len() * rows.len()) as u32;
    let started = Instant::now();
    let mut summary = RunSummary {
        total,
        iterations: rows.len() as u32,
        passed: 0,
        failed: 0,
        duration_ms: 0,
        cancelled: false,
    };
    let mut index = 0;
    'run: for (iteration, row) in rows.iter().enumerate() {
        for item in items {
            if index > 0 && options.delay_ms > 0 {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(options.delay_ms.into())) => {}
                    _ = &mut cancelled => { summary.cancelled = true; break 'run; }
                }
            }
            index += 1;
            let position = (index, total, iteration as u32 + 1);
            let result = tokio::select! {
                result = run_one(state, item, row, position) => result,
                _ = &mut cancelled => { summary.cancelled = true; break 'run; }
            };
            if result.passed {
                summary.passed += 1;
            } else {
                summary.failed += 1;
            }
            let stop = !result.passed && options.stop_on_failure;
            emit(result);
            if stop {
                break 'run;
            }
        }
    }
    summary.duration_ms = started.elapsed().as_millis() as u32;
    summary
}

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

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
