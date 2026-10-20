use std::fs;

use super::*;
use crate::db;
use crate::services::cookies::CookieJar;
use crate::services::secrets::MemoryStore;
use crate::state::AppState;

fn state() -> AppState {
    let conn = db::open_in_memory().unwrap();
    let ws = db::workspace::current(&conn).unwrap().id;
    AppState::new(
        conn,
        ws,
        Box::new(MemoryStore::default()),
        std::env::temp_dir(),
        CookieJar::default(),
        Default::default(),
    )
}

fn names(state: &AppState) -> Vec<String> {
    db::collections::list_collections(&state.db().unwrap(), state.workspace_id())
        .unwrap()
        .into_iter()
        .map(|c| c.name)
        .collect()
}

#[test]
fn empty_folder_receives_data_and_outside_edits_are_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let (state, engine) = (state(), SyncEngine::default());
    db::collections::create_collection(&state.db().unwrap(), state.workspace_id(), "Users")
        .unwrap();
    assert!(
        !engine
            .attach(&state, Some(dir.path().into()), None)
            .unwrap()
    );
    let file = dir.path().join("collections/users/collection.yaml");
    assert!(file.is_file() && dir.path().join("zorp.yaml").is_file());
    let ignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(ignore.contains(".DS_Store"));

    db::collections::create_collection(&state.db().unwrap(), state.workspace_id(), "Orders")
        .unwrap();
    engine.push(&state).unwrap();
    assert!(
        dir.path()
            .join("collections/orders/collection.yaml")
            .is_file()
    );

    let edited = fs::read_to_string(&file)
        .unwrap()
        .replace("name: Users", "name: Customers");
    fs::write(&file, edited).unwrap();
    assert!(engine.pull(&state).unwrap());
    assert_eq!(names(&state), ["Customers", "Orders"]);
    assert!(!engine.pull(&state).unwrap());
}

#[test]
fn broken_files_pause_sync_without_overwriting() {
    let dir = tempfile::tempdir().unwrap();
    let (state, engine) = (state(), SyncEngine::default());
    db::collections::create_collection(&state.db().unwrap(), state.workspace_id(), "Users")
        .unwrap();
    engine
        .attach(&state, Some(dir.path().into()), None)
        .unwrap();
    let file = dir.path().join("collections/users/collection.yaml");
    let good = fs::read_to_string(&file).unwrap();

    fs::write(&file, "id: [broken").unwrap();
    assert!(!engine.pull(&state).unwrap());
    assert_eq!(engine.status().problems.len(), 1);
    db::collections::create_collection(&state.db().unwrap(), state.workspace_id(), "Orders")
        .unwrap();
    engine.push(&state).unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "id: [broken");
    assert!(!dir.path().join("collections/orders").exists());

    fs::write(&file, good).unwrap();
    assert!(!engine.pull(&state).unwrap());
    assert!(engine.status().problems.is_empty());
    assert_eq!(names(&state), ["Users", "Orders"]);
    assert!(
        dir.path()
            .join("collections/orders/collection.yaml")
            .is_file()
    );
}

#[test]
fn folder_with_stray_workspace_files_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("collections/x")).unwrap();
    fs::write(dir.path().join("collections/x/collection.yaml"), "keep").unwrap();
    let (state, engine) = (state(), SyncEngine::default());
    engine
        .attach(&state, Some(dir.path().into()), None)
        .unwrap();
    assert_eq!(engine.status().problems.len(), 1);
    assert_eq!(
        fs::read_to_string(dir.path().join("collections/x/collection.yaml")).unwrap(),
        "keep"
    );
}

#[test]
fn hook_signals_only_shared_tables() {
    let conn = db::open_in_memory().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    conn.update_hook(Some(table_hook(tx))).unwrap();
    let ws = db::workspace::current(&conn).unwrap().id;
    while rx.try_recv().is_ok() {}
    db::settings::save(&conn, ws, &Default::default()).unwrap();
    assert!(rx.try_recv().is_err());
    db::collections::create_collection(&conn, ws, "Users").unwrap();
    assert!(rx.try_recv().is_ok());
}
