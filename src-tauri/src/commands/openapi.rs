use std::time::Duration;

use tauri::State;

use crate::db;
use crate::error::{AppError, AppResult, ErrorKind};
use crate::models::import::ImportReport;
use crate::models::openapi::{OpenApiOptions, OpenApiSummary};
use crate::services::client_pool::ClientKey;
use crate::services::importers::openapi;
use crate::state::AppState;

const MAX_BYTES: usize = 20 * 1024 * 1024;
const FETCH_TIMEOUT: Duration = Duration::from_secs(30);

fn too_large() -> AppError {
    AppError::Validation("Ukuran dokumen maksimal 20 MB".into())
}

fn network(message: String) -> AppError {
    AppError::Network {
        kind: ErrorKind::Network,
        message,
    }
}

/// Reads a document from a local file or an http(s) URL. URLs are fetched here, with the
/// workspace proxy and certificate settings, so the webview never makes the request.
async fn read_source(state: &AppState, source: &str) -> AppResult<String> {
    let source = source.trim();
    if !(source.starts_with("http://") || source.starts_with("https://")) {
        if std::fs::metadata(source)?.len() > MAX_BYTES as u64 {
            return Err(too_large());
        }
        return Ok(std::fs::read_to_string(source)?);
    }
    let url = url::Url::parse(source).map_err(|e| AppError::InvalidUrl(e.to_string()))?;
    let client = state.client(ClientKey::plain())?;
    let mut response = client
        .get(url)
        .timeout(FETCH_TIMEOUT)
        .send()
        .await
        .map_err(|e| network(format!("Dokumen tidak bisa diunduh: {e}")))?;
    if !response.status().is_success() {
        return Err(network(format!("Server membalas {}", response.status())));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| network(e.to_string()))? {
        if bytes.len() + chunk.len() > MAX_BYTES {
            return Err(too_large());
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| AppError::Validation("Dokumen bukan teks UTF-8".into()))
}

#[tauri::command]
pub async fn preview_openapi(
    state: State<'_, AppState>,
    source: String,
) -> AppResult<OpenApiSummary> {
    openapi::summarize(&openapi::load(&read_source(&state, &source).await?)?)
}

#[tauri::command]
pub async fn import_openapi(
    state: State<'_, AppState>,
    source: String,
    options: OpenApiOptions,
) -> AppResult<ImportReport> {
    let root = openapi::load(&read_source(&state, &source).await?)?;
    let imported = openapi::convert(&root, &options)?;
    db::import::save(&*state.db()?, state.workspace_id(), imported)
}
