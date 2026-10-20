use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::mock::{MockRouteInfo, MockStatus};
use crate::services::mock::routes::{self, Route};
use crate::services::mock::server::{self, Running};
use crate::state::AppState;

const LOG_EVENT: &str = "mock://log";
const DEFAULT_PORT: u16 = 4010;

/// Mock servers that are running, one per collection.
#[derive(Default)]
pub struct MockRegistry(Mutex<HashMap<Uuid, Running>>);

fn lock(registry: &MockRegistry) -> AppResult<std::sync::MutexGuard<'_, HashMap<Uuid, Running>>> {
    registry
        .0
        .lock()
        .map_err(|_| AppError::Internal("Status mock server tidak konsisten".into()))
}

fn collection_routes(conn: &Connection, collection_id: Uuid) -> AppResult<Vec<Route>> {
    let workspace: String = conn
        .query_row(
            "SELECT workspace_id FROM collections WHERE id = ?1",
            [collection_id.to_string()],
            |r| r.get(0),
        )
        .map_err(|_| AppError::NotFound("Koleksi"))?;
    let workspace = Uuid::parse_str(&workspace).map_err(|e| AppError::Internal(e.to_string()))?;
    let requests: Vec<_> = db::requests::list(conn, workspace)?
        .into_iter()
        .filter(|r| r.collection_id == collection_id)
        .collect();
    Ok(routes::build(
        &requests,
        &db::examples::list(conn, workspace)?,
    ))
}

fn status(collection_id: Uuid, port: u16) -> MockStatus {
    MockStatus {
        collection_id,
        port,
        url: format!("http://127.0.0.1:{port}"),
    }
}

#[tauri::command]
pub fn mock_routes(
    state: State<'_, AppState>,
    collection_id: Uuid,
) -> AppResult<Vec<MockRouteInfo>> {
    Ok(routes::info(&collection_routes(
        &*state.db()?,
        collection_id,
    )?))
}

#[tauri::command]
pub fn list_mocks(registry: State<'_, MockRegistry>) -> AppResult<Vec<MockStatus>> {
    Ok(lock(&registry)?
        .iter()
        .map(|(id, r)| status(*id, r.port))
        .collect())
}

/// Starts serving the collection's examples on 127.0.0.1.
#[tauri::command]
pub async fn start_mock(
    app: AppHandle,
    registry: State<'_, MockRegistry>,
    collection_id: Uuid,
    port: Option<u16>,
) -> AppResult<MockStatus> {
    if let Some(running) = lock(&registry)?.get(&collection_id) {
        return Ok(status(collection_id, running.port));
    }
    let source_app = app.clone();
    let source =
        Arc::new(move || collection_routes(&*source_app.state::<AppState>().db()?, collection_id));
    let log_app = app.clone();
    let log = Arc::new(move |entry| {
        let _ = log_app.emit(LOG_EVENT, entry);
    });
    let running = server::start(collection_id, port.unwrap_or(DEFAULT_PORT), source, log).await?;
    let port = running.port;
    lock(&registry)?.insert(collection_id, running);
    Ok(status(collection_id, port))
}

#[tauri::command]
pub fn stop_mock(registry: State<'_, MockRegistry>, collection_id: Uuid) -> AppResult<()> {
    if let Some(running) = lock(&registry)?.remove(&collection_id) {
        running.stop();
    }
    Ok(())
}
