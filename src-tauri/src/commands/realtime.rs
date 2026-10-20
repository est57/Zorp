use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

use crate::db::now_ms;
use crate::error::{AppError, AppResult};
use crate::models::realtime::{RealtimeEvent, RequestMode};
use crate::models::request::RequestSpec;
use crate::pipeline::{self, RequestContext};
use crate::services::realtime::{Emit, Outgoing, sse_client, websocket};
use crate::state::AppState;

const REALTIME_EVENT: &str = "realtime://event";

/// Opens a WebSocket or SSE connection; progress and messages arrive as `realtime://event`.
#[tauri::command]
pub async fn realtime_connect(
    app: AppHandle,
    state: State<'_, AppState>,
    connection_id: Uuid,
    spec: RequestSpec,
) -> AppResult<()> {
    let ctx = RequestContext::load(&state, &spec)?;
    let prepared = pipeline::prepare(&state, &spec, &ctx, &Default::default()).await?;
    let outgoing = state.realtime.open(connection_id)?;
    let handle = app.clone();
    let emit: Emit = Box::new(move |kind, data, event, binary| {
        let payload = RealtimeEvent {
            connection_id,
            kind,
            data,
            event,
            binary,
            at: now_ms(),
        };
        let _ = handle.emit(REALTIME_EVENT, payload);
    });
    let mode = spec.mode;
    tauri::async_runtime::spawn(async move {
        match mode {
            RequestMode::Websocket => {
                websocket::run(prepared.spec, prepared.auth, outgoing, emit).await
            }
            RequestMode::Sse => {
                sse_client::run(
                    prepared.client,
                    prepared.spec,
                    prepared.auth,
                    outgoing,
                    emit,
                )
                .await
            }
            RequestMode::Http => {}
        }
        let state = app.state::<AppState>();
        state.realtime.remove(connection_id);
        let _ = state.persist_cookies();
    });
    if mode == RequestMode::Http {
        return Err(AppError::Validation(
            "Request ini bukan WebSocket atau SSE".into(),
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn realtime_send(
    state: State<'_, AppState>,
    connection_id: Uuid,
    text: String,
) -> AppResult<()> {
    state.realtime.send(connection_id, Outgoing::Text(text))
}

#[tauri::command]
pub fn realtime_close(state: State<'_, AppState>, connection_id: Uuid) -> AppResult<()> {
    match state.realtime.send(connection_id, Outgoing::Close) {
        Ok(()) | Err(AppError::Validation(_)) => Ok(()),
        Err(e) => Err(e),
    }
}
