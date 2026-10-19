use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::db;
use crate::db::history::NewEntry;
use crate::error::{AppError, AppResult};
use crate::models::request::{Auth, RequestSpec};
use crate::models::response::{ProgressEvent, ResponseData};
use crate::services::resolver::{Variables, resolve_spec};
use crate::services::{http_engine, request_builder};
use crate::state::AppState;

const PROGRESS_EVENT: &str = "request://progress";

fn effective_auth(state: &AppState, spec: &RequestSpec) -> AppResult<(Auth, Variables)> {
    let conn = state.db()?;
    let env = db::environments::active_values(&conn, state.workspace_id, state.secrets.as_ref())?;
    let auth = match &spec.auth {
        Auth::Inherit => match db::requests::location(&conn, spec.id)? {
            Some((collection_id, folder_id)) => {
                db::collections::inherited_auth(&conn, collection_id, folder_id)?
                    .unwrap_or(Auth::None)
            }
            None => Auth::None,
        },
        other => other.clone(),
    };
    Ok((auth, Variables::from_layers(vec![env])))
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
) -> AppResult<ResponseData> {
    let (auth, vars) = effective_auth(&state, &spec)?;
    let (resolved, auth) = resolve_spec(&spec, &auth, &vars)?;
    let client = state.client(spec.settings.verify_tls, spec.settings.follow_redirects)?;
    let builder = request_builder::build(&client, &resolved, &auth)?;

    let cancelled = state.register(request_id)?;
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
    let result = tokio::select! {
        result = http_engine::execute(builder, &state.spill_dir, progress) => result,
        _ = cancelled => Err(AppError::Cancelled),
    };
    state.finish(request_id);
    record(&state, &spec, &result)?;
    result
}

#[tauri::command]
pub fn cancel_request(state: State<'_, AppState>, request_id: Uuid) -> AppResult<()> {
    state.cancel(request_id)
}
