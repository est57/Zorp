use std::path::PathBuf;

use tauri::{App, AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::db;
use crate::error::{AppError, AppResult};
use crate::services::cookies::CookieJar;
use crate::services::secrets::KeychainStore;
use crate::state::AppState;
use crate::sync;

/// Development builds keep their own database so they never upgrade the one used by
/// the installed app.
const DB_FILE: &str = if cfg!(debug_assertions) {
    "zorp-dev.db"
} else {
    "zorp.db"
};

fn path_error(e: tauri::Error) -> AppError {
    AppError::Internal(format!("Folder data aplikasi tidak ditemukan: {e}"))
}

/// Opens the database, loads the active workspace and starts folder sync.
pub fn init(app: &App) -> AppResult<()> {
    let data_dir = app.path().app_data_dir().map_err(path_error)?;
    std::fs::create_dir_all(&data_dir)?;
    let spill_dir = app
        .path()
        .app_cache_dir()
        .map_err(path_error)?
        .join("responses");
    // Large response files are only needed for the current session.
    let _ = std::fs::remove_dir_all(&spill_dir);
    let conn = db::open(&data_dir.join(DB_FILE))?;
    let (changes, flush) = std::sync::mpsc::channel();
    conn.update_hook(Some(sync::table_hook(changes)))?;
    let workspace = db::workspace::current(&conn)?;
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
    app.manage(sync::SyncEngine::default());
    sync::start_flusher(app.handle().clone(), flush);
    let handle = app.handle();
    let folder = workspace.folder.map(PathBuf::from);
    let watcher = folder.as_deref().and_then(|root| sync::watch(handle, root));
    handle
        .state::<sync::SyncEngine>()
        .attach(&handle.state::<AppState>(), folder, watcher)?;
    Ok(())
}

/// Explains a failed start in a dialog and quits once it is closed, instead of aborting.
pub fn report(app: &AppHandle, message: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    let handle = app.clone();
    app.dialog()
        .message(format!("Zorp tidak bisa dibuka.\n\n{message}"))
        .title("Zorp")
        .kind(MessageDialogKind::Error)
        .show(move |_| handle.exit(1));
}
