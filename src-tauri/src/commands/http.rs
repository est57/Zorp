use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::db;
use crate::db::history::NewEntry;
use crate::error::{AppError, AppResult};
use crate::models::request::{Auth, RequestSpec};
use crate::models::response::{ProgressEvent, ResponseData};
use crate::services::resolver::{Variables, resolve_spec};
use crate::services::{http_engine, oauth, request_builder};
use crate::state::AppState;

const PROGRESS_EVENT: &str = "request://progress";

/// Collection variables overridden by the active environment, matching Postman precedence.
pub(crate) fn context_variables(
    state: &AppState,
    collection_id: Option<Uuid>,
) -> AppResult<Variables> {
    let conn = state.db()?;
    let env = db::environments::active_values(&conn, state.workspace_id, state.secrets.as_ref())?;
    let collection_vars = match collection_id {
        Some(id) => db::collections::variables(&conn, id)?,
        None => Vec::new(),
    };
    Ok(Variables::from_layers(vec![collection_vars, env]))
}

fn effective_auth(state: &AppState, spec: &RequestSpec) -> AppResult<(Auth, Variables)> {
    let location = db::requests::location(&*state.db()?, spec.id)?;
    let auth = match (&spec.auth, location) {
        (Auth::Inherit, Some((collection_id, folder_id))) => {
            db::collections::inherited_auth(&*state.db()?, collection_id, folder_id)?
                .unwrap_or(Auth::None)
        }
        (Auth::Inherit, None) => Auth::None,
        (other, _) => other.clone(),
    };
    Ok((auth, context_variables(state, location.map(|(c, _)| c))?))
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
    let client = state.client(
        spec.settings.verify_tls,
        spec.settings.follow_redirects,
        spec.settings.use_cookies,
    )?;
    let auth = match auth {
        Auth::OAuth2(config) => {
            let config = oauth::resolve_config(&config, &vars)?;
            let token_client = state.client(true, true, false)?;
            Auth::Bearer {
                token: oauth::access_token(state.secrets.as_ref(), &token_client, &config).await?,
            }
        }
        other => other,
    };
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
    state.persist_cookies()?;
    record(&state, &spec, &result)?;
    result
}

#[tauri::command]
pub fn cancel_request(state: State<'_, AppState>, request_id: Uuid) -> AppResult<()> {
    state.cancel(request_id)
}
