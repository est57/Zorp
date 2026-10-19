use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use reqwest::Client;
use rusqlite::Connection;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::cookies::CookieJar;
use crate::services::http_engine;
use crate::services::secrets::SecretStore;

pub struct AppState {
    db: Mutex<Connection>,
    pub workspace_id: Uuid,
    pub secrets: Box<dyn SecretStore>,
    pub spill_dir: PathBuf,
    pub cookies: Arc<CookieJar>,
    clients: Mutex<HashMap<(bool, bool, bool), Client>>,
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
        cookies: CookieJar,
    ) -> Self {
        Self {
            db: Mutex::new(conn),
            workspace_id,
            secrets,
            spill_dir,
            cookies: Arc::new(cookies),
            clients: Mutex::default(),
            inflight: Mutex::default(),
        }
    }

    pub fn db(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.db.lock().map_err(|_| poisoned())
    }

    /// Returns a pooled client so connections are reused across requests.
    pub fn client(
        &self,
        verify_tls: bool,
        follow_redirects: bool,
        use_cookies: bool,
    ) -> AppResult<Client> {
        let key = (verify_tls, follow_redirects, use_cookies);
        let mut clients = self.clients.lock().map_err(|_| poisoned())?;
        if let Some(client) = clients.get(&key) {
            return Ok(client.clone());
        }
        let jar = use_cookies.then(|| self.cookies.clone());
        let client = http_engine::new_client(verify_tls, follow_redirects, jar)?;
        clients.insert(key, client.clone());
        Ok(client)
    }

    /// Writes the cookie jar to SQLite when it changed.
    pub fn persist_cookies(&self) -> AppResult<()> {
        if self.cookies.take_dirty() {
            crate::db::cookies::save(&*self.db()?, self.workspace_id, &self.cookies.to_json()?)?;
        }
        Ok(())
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
