use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

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
    workspace: RwLock<Uuid>,
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
            workspace: RwLock::new(workspace_id),
            secrets,
            spill_dir,
            cookies: Arc::new(cookies),
            pool: ClientPool::new(settings),
            realtime: Registry::default(),
            inflight: Mutex::default(),
        }
    }

    /// The workspace every command currently operates on.
    pub fn workspace_id(&self) -> Uuid {
        *self.workspace.read().unwrap_or_else(|e| e.into_inner())
    }

    /// Makes another workspace active, saving the current cookies and loading its own
    /// cookies and connection settings.
    pub fn activate(&self, workspace_id: Uuid) -> AppResult<()> {
        self.persist_cookies()?;
        let conn = self.db()?;
        crate::db::workspace::touch(&conn, workspace_id)?;
        let jar = crate::db::cookies::load(&conn, workspace_id)?;
        let settings = crate::db::settings::load(&conn, workspace_id)?;
        self.cookies.replace(jar.as_deref())?;
        self.pool.apply(settings)?;
        *self.workspace.write().map_err(|_| poisoned())? = workspace_id;
        Ok(())
    }

    pub fn db(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.db.lock().map_err(|_| poisoned())
    }

    pub fn client(&self, key: ClientKey) -> AppResult<Client> {
        self.pool.get(
            key,
            &self.cookies,
            self.secrets.as_ref(),
            self.workspace_id(),
        )
    }

    /// Writes the cookie jar to SQLite when it changed.
    pub fn persist_cookies(&self) -> AppResult<()> {
        if self.cookies.take_dirty() {
            crate::db::cookies::save(&*self.db()?, self.workspace_id(), &self.cookies.to_json()?)?;
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

#[cfg(test)]
mod tests {
    use reqwest::cookie::CookieStore;

    use super::*;
    use crate::db;
    use crate::services::secrets::MemoryStore;

    #[test]
    fn activate_swaps_cookies_and_settings_per_workspace() {
        let conn = db::open_in_memory().unwrap();
        let first = db::workspace::current(&conn).unwrap().id;
        let second = db::workspace::create(&conn, "Tim", None).unwrap().id;
        let mut proxy = WorkspaceSettings::default();
        proxy.proxy.enabled = true;
        db::settings::save(&conn, second, &proxy).unwrap();
        let state = AppState::new(
            conn,
            first,
            Box::new(MemoryStore::default()),
            std::env::temp_dir(),
            CookieJar::default(),
            WorkspaceSettings::default(),
        );
        let url = url::Url::parse("https://api.example.com/").unwrap();
        let header = reqwest::header::HeaderValue::from_static("sid=one");
        state
            .cookies
            .set_cookies(&mut std::iter::once(&header), &url);

        state.activate(second).unwrap();
        assert_eq!(state.workspace_id(), second);
        assert!(state.cookies.list().is_empty());
        assert!(state.pool.settings().unwrap().proxy.enabled);

        state.activate(first).unwrap();
        assert_eq!(state.cookies.list()[0].value, "one");
        assert!(!state.pool.settings().unwrap().proxy.enabled);
        assert_eq!(
            db::workspace::current(&state.db().unwrap()).unwrap().id,
            first
        );
    }
}
