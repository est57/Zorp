// Without the desktop app, parts of the core used only by its commands go unused.
#![cfg_attr(not(feature = "gui"), allow(dead_code, unused_imports))]

pub mod cli;
#[cfg(feature = "gui")]
mod commands;
mod db;
mod error;
mod models;
mod pipeline;
mod services;
#[cfg(feature = "gui")]
mod startup;
mod state;
#[cfg(feature = "gui")]
mod sync;

#[cfg(feature = "gui")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Some(code) = cli::try_run() {
        std::process::exit(code);
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            if let Err(e) = startup::init(app) {
                startup::report(app.handle(), &e.to_string());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::app_info,
            commands::app::open_link,
            commands::mock::mock_routes,
            commands::mock::list_mocks,
            commands::mock::start_mock,
            commands::mock::stop_mock,
            commands::workspaces::list_workspaces,
            commands::workspaces::create_workspace,
            commands::workspaces::switch_workspace,
            commands::workspaces::rename_workspace,
            commands::workspaces::remove_workspace,
            commands::workspace_folder::sync_status,
            commands::workspace_folder::open_folder,
            commands::workspace_folder::link_folder,
            commands::workspace_folder::unlink_folder,
            commands::workspace_folder::reveal_folder,
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
            commands::collections::set_node_settings,
            commands::collections::get_collection_variables,
            commands::collections::set_collection_variables,
            commands::environments::list_environments,
            commands::environments::save_environment,
            commands::environments::delete_environment,
            commands::environments::set_active_environment,
            commands::environments::get_globals,
            commands::environments::save_globals,
            commands::history::list_history,
            commands::history::clear_history,
            commands::import::import_postman,
            commands::examples::save_example,
            commands::examples::get_example,
            commands::examples::rename_example,
            commands::examples::delete_example,
            commands::openapi::preview_openapi,
            commands::openapi::import_openapi,
            commands::export::export_collection,
            commands::export::export_environment,
            commands::export::import_environment,
            commands::oauth::oauth_get_token,
            commands::oauth::oauth_cancel,
            commands::oauth::oauth_token_status,
            commands::oauth::oauth_clear_token,
            commands::cookies::list_cookies,
            commands::cookies::delete_cookie,
            commands::cookies::clear_cookies,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::runner::run_collection,
            commands::runner::cancel_run,
            commands::runner::preview_run_data,
            commands::runner::save_run_report,
            commands::realtime::realtime_connect,
            commands::realtime::realtime_send,
            commands::realtime::realtime_close,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Zorp");
}
