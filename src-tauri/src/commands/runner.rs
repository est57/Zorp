use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, State};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::collection::SavedRequest;
use crate::models::runner::{RunItemResult, RunSummary, RunnerEvent, RunnerOptions};
use crate::pipeline;
use crate::state::AppState;

const RUNNER_EVENT: &str = "runner://item";

async fn run_one(state: &AppState, item: &SavedRequest, index: u32, total: u32) -> RunItemResult {
    let request_id = Uuid::now_v7();
    let (_, result) = pipeline::execute(state, item.spec.clone(), request_id, |_, _| {}).await;
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

/// Runs items in order until done, cancelled, or a failure when `stop_on_failure` is set.
pub(crate) async fn run_items(
    state: &AppState,
    items: &[SavedRequest],
    options: &RunnerOptions,
    mut cancelled: oneshot::Receiver<()>,
    emit: impl Fn(RunItemResult),
) -> RunSummary {
    let total = items.len() as u32;
    let started = Instant::now();
    let mut summary = RunSummary {
        total,
        passed: 0,
        failed: 0,
        duration_ms: 0,
        cancelled: false,
    };
    for (i, item) in items.iter().enumerate() {
        if i > 0 && options.delay_ms > 0 {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(options.delay_ms.into())) => {}
                _ = &mut cancelled => { summary.cancelled = true; break; }
            }
        }
        let result = tokio::select! {
            result = run_one(state, item, i as u32 + 1, total) => result,
            _ = &mut cancelled => { summary.cancelled = true; break; }
        };
        if result.passed {
            summary.passed += 1;
        } else {
            summary.failed += 1;
        }
        let stop = !result.passed && options.stop_on_failure;
        emit(result);
        if stop {
            break;
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
    let cancelled = state.register(run_id)?;
    let emit = |item| {
        let _ = app.emit(RUNNER_EVENT, RunnerEvent { run_id, item });
    };
    let summary = run_items(&state, &items, &options, cancelled, emit).await;
    state.finish(run_id);
    Ok(summary)
}

#[tauri::command]
pub fn cancel_run(state: State<'_, AppState>, run_id: Uuid) -> AppResult<()> {
    state.cancel(run_id)
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
