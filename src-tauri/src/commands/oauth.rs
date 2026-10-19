use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

use super::http::context_variables;
use crate::error::{AppError, AppResult};
use crate::models::oauth::{OAuth2Config, TokenStatus};
use crate::services::oauth::{self, token_store};
use crate::state::AppState;

/// Only one browser login runs at a time; it is tracked under this id for cancellation.
const LOGIN_FLOW_ID: Uuid = Uuid::nil();

#[tauri::command]
pub async fn oauth_get_token(
    app: AppHandle,
    state: State<'_, AppState>,
    config: OAuth2Config,
    collection_id: Option<Uuid>,
) -> AppResult<TokenStatus> {
    let config = oauth::resolve_config(&config, &context_variables(&state, collection_id)?)?;
    let client = state.client(true, true, false)?;
    state.cancel(LOGIN_FLOW_ID)?;
    let cancelled = state.register(LOGIN_FLOW_ID)?;
    let open_browser = |url: &str| {
        app.opener()
            .open_url(url, None::<&str>)
            .map_err(|e| AppError::OAuth(format!("Browser tidak bisa dibuka: {e}")))
    };
    let result = oauth::fetch_token(&client, &config, open_browser, cancelled).await;
    state.finish(LOGIN_FLOW_ID);
    let token = result?;
    token_store::save(state.secrets.as_ref(), &config.token_key, &token)?;
    Ok(token.status())
}

#[tauri::command]
pub fn oauth_cancel(state: State<'_, AppState>) -> AppResult<()> {
    state.cancel(LOGIN_FLOW_ID)
}

#[tauri::command]
pub fn oauth_token_status(state: State<'_, AppState>, token_key: String) -> AppResult<TokenStatus> {
    Ok(token_store::load(state.secrets.as_ref(), &token_key)?
        .map(|t| t.status())
        .unwrap_or_default())
}

#[tauri::command]
pub fn oauth_clear_token(state: State<'_, AppState>, token_key: String) -> AppResult<()> {
    token_store::clear(state.secrets.as_ref(), &token_key)
}
