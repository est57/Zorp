use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use reqwest::Client;
use rusqlite::Connection;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::settings::WorkspaceSettings;
use crate::services::client_pool::{ClientKey, ClientPool};
use crate::services::cookies::CookieJar;
use crate::services::realtime::Registry;
use crate::services::secrets::SecretStore;

pub struct AppState {
    db: Mutex<Connection>,
    pub workspace_id: Uuid,
    pub secrets: Box<dyn SecretStore>,
    pub spill_dir: PathBuf,
    pub cookies: Arc<CookieJar>,
    pub pool: ClientPool,
    pub realtime: Registry,
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
        settings: WorkspaceSettings,
    ) -> Self {
        Self {
            db: Mutex::new(conn),
            workspace_id,
            secrets,
            spill_dir,
            cookies: Arc::new(cookies),
            pool: ClientPool::new(settings),
            realtime: Registry::default(),
            inflight: Mutex::default(),
        }
    }

    pub fn db(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.db.lock().map_err(|_| poisoned())
    }

    pub fn client(&self, key: ClientKey) -> AppResult<Client> {
        self.pool
            .get(key, &self.cookies, self.secrets.as_ref(), self.workspace_id)
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
