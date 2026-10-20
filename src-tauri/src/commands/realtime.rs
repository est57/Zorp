use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

use crate::db::now_ms;
use crate::error::{AppError, AppResult};
use crate::models::realtime::{RealtimeEvent, RequestMode};
use crate::models::request::RequestSpec;
use crate::pipeline::{self, RequestContext};
use std::time::Duration;

use crate::services::client_pool::proxy_account;
use crate::services::network;
use crate::services::realtime::{Emit, Outgoing, sse_client, tls, transport, websocket};
use crate::state::AppState;

const REALTIME_EVENT: &str = "realtime://event";

/// Proxy, TLS roots and client certificate from the connection settings, as used for HTTP.
fn websocket_network(state: &AppState, spec: &RequestSpec) -> AppResult<websocket::Network> {
    let settings = state.pool.settings()?;
    let url = url::Url::parse(spec.url.trim()).map_err(|e| AppError::InvalidUrl(e.to_string()))?;
    let host = url.host_str().unwrap_or_default().to_string();
    let password = if settings.proxy.has_password {
        state.secrets.get(&proxy_account(state.workspace_id()))?
    } else {
        None
    };
    let client_cert = network::certificate_for_host(&settings, &host);
    Ok(websocket::Network {
        proxy: transport::select(&settings.proxy, password, &host)?,
        tls: tls::client_config(&settings, spec.settings.verify_tls, client_cert)?,
        timeout: Duration::from_millis(spec.settings.timeout_ms.into()),
    })
}

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
    let network = match spec.mode {
        RequestMode::Websocket => Some(websocket_network(&state, &prepared.spec)?),
        _ => None,
    };
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
                if let Some(network) = network {
                    websocket::run(prepared.spec, prepared.auth, network, outgoing, emit).await
                }
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
