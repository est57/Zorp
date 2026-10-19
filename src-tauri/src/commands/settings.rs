use tauri::State;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::settings::WorkspaceSettings;
use crate::services::client_pool::proxy_account;
use crate::services::network;
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<WorkspaceSettings> {
    state.pool.settings()
}

/// Validates certificate files and proxy settings before applying them to new connections.
#[tauri::command]
pub fn save_settings(
    state: State<'_, AppState>,
    mut settings: WorkspaceSettings,
) -> AppResult<WorkspaceSettings> {
    let proxy = &mut settings.proxy;
    proxy.url = proxy.url.trim().to_string();
    if proxy.enabled {
        network::validate_proxy_url(&proxy.url)?;
    }
    let account = proxy_account(state.workspace_id);
    if proxy.username.is_empty() {
        state.secrets.delete(&account)?;
        proxy.has_password = false;
    } else if !proxy.password.is_empty() {
        state.secrets.set(&account, &proxy.password)?;
        proxy.has_password = true;
    } else {
        proxy.has_password = state.pool.settings()?.proxy.has_password;
    }
    proxy.password.clear();

    network::load_roots(&settings)?;
    for cert in settings.client_certificates.iter_mut() {
        cert.host = cert.host.trim().to_ascii_lowercase();
        if cert.host.is_empty() {
            return Err(AppError::Validation(
                "Host sertifikat klien wajib diisi".into(),
            ));
        }
        if cert.enabled {
            network::load_identity(cert)?;
        }
    }

    db::settings::save(&*state.db()?, state.workspace_id, &settings)?;
    state.pool.apply(settings.clone())?;
    Ok(settings)
}
