use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::db;
use crate::db::history::NewEntry;
use crate::error::{AppError, AppResult};
use crate::models::request::RequestSpec;
use crate::models::response::{ProgressEvent, ResponseData};
use crate::pipeline::{self, RequestContext};
use crate::services::resolver::Variables;
use crate::state::AppState;

const PROGRESS_EVENT: &str = "request://progress";

/// Variables for flows outside a request, such as fetching an OAuth token.
pub(crate) fn context_variables(
    state: &AppState,
    collection_id: Option<Uuid>,
) -> AppResult<Variables> {
    Ok(RequestContext::for_location(state, collection_id, None)?.variables(&Default::default()))
}

fn record(state: &AppState, spec: &RequestSpec, result: &AppResult<ResponseData>) -> AppResult<()> {
    let entry = match result {
        Ok(res) => NewEntry {
            request: spec,
            status: Some(res.status),
            duration_ms: Some(res.timing.total_ms),
            size_bytes: Some(res.size_bytes),
        },
        Err(AppError::Network { .. }) => NewEntry {
            request: spec,
            status: None,
            duration_ms: None,
            size_bytes: None,
        },
        Err(_) => return Ok(()),
    };
    db::history::insert(&*state.db()?, state.workspace_id, entry)
}

#[tauri::command]
pub async fn send_request(
    app: AppHandle,
    state: State<'_, AppState>,
    request_id: Uuid,
    spec: RequestSpec,
    record_history: Option<bool>,
) -> AppResult<ResponseData> {
    let progress = |phase, bytes| {
        let _ = app.emit(
            PROGRESS_EVENT,
            ProgressEvent {
                request_id,
                phase,
                bytes,
            },
        );
    };
    // History keeps the request as written, so values injected by scripts (often tokens) are not stored.
    let original = spec.clone();
    let (_, result) = pipeline::execute(&state, spec, request_id, progress).await;
    // Background calls such as GraphQL schema introspection opt out of history.
    if record_history.unwrap_or(true) {
        record(&state, &original, &result)?;
    }
    result
}
#[tauri::command]
pub fn cancel_request(state: State<'_, AppState>, request_id: Uuid) -> AppResult<()> {
    state.cancel(request_id)
}
