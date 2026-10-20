use std::time::{Duration, Instant};

use tokio::sync::oneshot;
use uuid::Uuid;

use crate::models::collection::SavedRequest;
use crate::models::runner::{RunItemResult, RunSummary, RunnerOptions};
use crate::pipeline;
use crate::state::AppState;

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
pub async fn run_items(
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

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
