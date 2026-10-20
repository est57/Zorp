use std::path::Path;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{DebounceEventResult, Debouncer, new_debouncer};
use tauri::{AppHandle, Emitter, Manager};

use super::{CHANGED_EVENT, STATUS_EVENT, SyncEngine};
use crate::state::AppState;

const SHARED_TABLES: [&str; 5] = [
    "workspaces",
    "collections",
    "folders",
    "requests",
    "environments",
];
const QUIET: Duration = Duration::from_millis(300);

/// SQLite update hook that signals the flusher when shared data changed.
pub fn table_hook(
    tx: Sender<()>,
) -> impl FnMut(rusqlite::hooks::Action, &str, &str, i64) + Send + 'static {
    move |_, _, table, _| {
        if SHARED_TABLES.contains(&table) {
            let _ = tx.send(());
        }
    }
}

/// Writes app changes to the folder once edits have been quiet for a moment.
pub fn start_flusher(app: AppHandle, rx: Receiver<()>) {
    std::thread::spawn(move || {
        while rx.recv().is_ok() {
            loop {
                match rx.recv_timeout(QUIET) {
                    Ok(()) => continue,
                    Err(RecvTimeoutError::Timeout) => break,
                    Err(RecvTimeoutError::Disconnected) => return,
                }
            }
            let engine = app.state::<SyncEngine>();
            let before = engine.status().problems;
            if let Err(e) = engine.push(&app.state::<AppState>()) {
                engine.report(format!("Menulis folder gagal: {e}"));
            }
            if engine.status().problems != before {
                let _ = app.emit(STATUS_EVENT, engine.status());
            }
        }
    });
}

fn is_hidden(path: &Path, root: &Path) -> bool {
    path.strip_prefix(root)
        .map(|rel| {
            rel.components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
        })
        .unwrap_or(true)
}

/// Watches a folder and reads outside edits back into the app.
pub fn watch(app: &AppHandle, root: &Path) -> Option<Debouncer<RecommendedWatcher>> {
    let handle = app.clone();
    let base = root.to_path_buf();
    let mut debouncer = new_debouncer(
        Duration::from_millis(400),
        move |res: DebounceEventResult| {
            let Ok(events) = res else { return };
            if events.iter().all(|e| is_hidden(&e.path, &base)) {
                return;
            }
            let engine = handle.state::<SyncEngine>();
            match engine.pull(&handle.state::<AppState>()) {
                Ok(true) => {
                    let _ = handle.emit(CHANGED_EVENT, ());
                }
                Ok(false) => {}
                Err(e) => engine.report(format!("Membaca folder gagal: {e}")),
            }
            let _ = handle.emit(STATUS_EVENT, engine.status());
        },
    )
    .ok()?;
    debouncer
        .watcher()
        .watch(root, RecursiveMode::Recursive)
        .ok()?;
    Some(debouncer)
}
