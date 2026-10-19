mod commands;
mod db;
mod error;
mod models;
mod pipeline;
mod services;
mod state;

use tauri::Manager;

use crate::services::cookies::CookieJar;
use crate::services::secrets::KeychainStore;
use crate::state::AppState;

const DB_FILE: &str = "zorp.db";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let spill_dir = app.path().app_cache_dir()?.join("responses");
            // Large response files are only needed for the current session.
            let _ = std::fs::remove_dir_all(&spill_dir);
            let conn = db::open(&data_dir.join(DB_FILE))?;
            let workspace = db::workspace::ensure_default(&conn)?;
            let jar = db::cookies::load(&conn, workspace.id)?
                .map(|json| CookieJar::from_json(&json))
                .unwrap_or_default();
            let settings = db::settings::load(&conn, workspace.id)?;
            app.manage(AppState::new(
                conn,
                workspace.id,
                Box::new(KeychainStore),
                spill_dir,
                jar,
                settings,
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_info,
            commands::http::send_request,
            commands::http::cancel_request,
            commands::collections::list_tree,
            commands::collections::create_collection,
            commands::collections::create_folder,
            commands::collections::save_request,
            commands::collections::rename_node,
            commands::collections::delete_node,
            commands::collections::move_node,
            commands::collections::set_node_auth,
            commands::environments::list_environments,
            commands::environments::save_environment,
            commands::environments::delete_environment,
            commands::environments::set_active_environment,
            commands::history::list_history,
            commands::history::clear_history,
            commands::import::import_postman,
            commands::oauth::oauth_get_token,
            commands::oauth::oauth_cancel,
            commands::oauth::oauth_token_status,
            commands::oauth::oauth_clear_token,
            commands::cookies::list_cookies,
            commands::cookies::delete_cookie,
            commands::cookies::clear_cookies,
            commands::settings::get_settings,
            commands::settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Zorp");
}
