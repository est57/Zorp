use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use reqwest::Client;
use rusqlite::Connection;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::http_engine;
use crate::services::secrets::SecretStore;

pub struct AppState {
    db: Mutex<Connection>,
    pub workspace_id: Uuid,
    pub secrets: Box<dyn SecretStore>,
    pub spill_dir: PathBuf,
    clients: Mutex<HashMap<(bool, bool), Client>>,
    inflight: Mutex<HashMap<Uuid, oneshot::Sender<()>>>,
}

fn poisoned() -> AppError {
    AppError::Internal("Status aplikasi tidak konsisten, mulai ulang aplikasi".into())
}

impl AppState {
    pub fn new(
        conn: Connection,
        workspace_id: Uuid,
        secrets: Box<dyn SecretStore>,
        spill_dir: PathBuf,
    ) -> Self {
        Self {
            db: Mutex::new(conn),
            workspace_id,
            secrets,
            spill_dir,
            clients: Mutex::default(),
            inflight: Mutex::default(),
        }
    }

    pub fn db(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.db.lock().map_err(|_| poisoned())
    }

    /// Returns a pooled client so connections are reused across requests.
    pub fn client(&self, verify_tls: bool, follow_redirects: bool) -> AppResult<Client> {
        let mut clients = self.clients.lock().map_err(|_| poisoned())?;
        if let Some(client) = clients.get(&(verify_tls, follow_redirects)) {
            return Ok(client.clone());
        }
        let client = http_engine::new_client(verify_tls, follow_redirects)?;
        clients.insert((verify_tls, follow_redirects), client.clone());
        Ok(client)
    }

    pub fn register(&self, request_id: Uuid) -> AppResult<oneshot::Receiver<()>> {
        let (tx, rx) = oneshot::channel();
        self.inflight
            .lock()
            .map_err(|_| poisoned())?
            .insert(request_id, tx);
        Ok(rx)
    }

    pub fn finish(&self, request_id: Uuid) {
        if let Ok(mut map) = self.inflight.lock() {
            map.remove(&request_id);
        }
    }

    pub fn cancel(&self, request_id: Uuid) -> AppResult<()> {
        if let Some(tx) = self
            .inflight
            .lock()
            .map_err(|_| poisoned())?
            .remove(&request_id)
        {
            let _ = tx.send(());
        }
        Ok(())
    }
}
