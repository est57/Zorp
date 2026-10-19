use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use reqwest::Client;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::settings::WorkspaceSettings;
use crate::services::cookies::CookieJar;
use crate::services::http_client::{ClientOptions, build_client};
use crate::services::network;
use crate::services::secrets::SecretStore;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientKey {
    pub verify_tls: bool,
    pub follow_redirects: bool,
    pub use_cookies: bool,
    /// Client certificate used for mutual TLS, if any.
    pub identity: Option<Uuid>,
}

impl ClientKey {
    /// Client for auxiliary calls such as OAuth token requests.
    pub fn plain() -> Self {
        Self {
            verify_tls: true,
            follow_redirects: true,
            use_cookies: false,
            identity: None,
        }
    }
}

pub fn proxy_account(workspace_id: Uuid) -> String {
    format!("proxy:{workspace_id}")
}

/// Pools clients per configuration so connections are reused across requests.
#[derive(Default)]
pub struct ClientPool {
    clients: Mutex<HashMap<ClientKey, Client>>,
    settings: RwLock<WorkspaceSettings>,
}

fn poisoned() -> AppError {
    AppError::Internal("Status koneksi tidak konsisten, mulai ulang aplikasi".into())
}

impl ClientPool {
    pub fn new(settings: WorkspaceSettings) -> Self {
        Self {
            clients: Mutex::default(),
            settings: RwLock::new(settings),
        }
    }

    pub fn settings(&self) -> AppResult<WorkspaceSettings> {
        Ok(self.settings.read().map_err(|_| poisoned())?.clone())
    }

    /// Replaces settings and drops pooled clients so new connections pick them up.
    pub fn apply(&self, settings: WorkspaceSettings) -> AppResult<()> {
        *self.settings.write().map_err(|_| poisoned())? = settings;
        self.clients.lock().map_err(|_| poisoned())?.clear();
        Ok(())
    }

    pub fn identity_for(&self, url: &str) -> AppResult<Option<Uuid>> {
        let settings = self.settings.read().map_err(|_| poisoned())?;
        Ok(network::certificate_for(&settings, url).map(|c| c.id))
    }

    pub fn get(
        &self,
        key: ClientKey,
        jar: &Arc<CookieJar>,
        secrets: &dyn SecretStore,
        workspace_id: Uuid,
    ) -> AppResult<Client> {
        let mut clients = self.clients.lock().map_err(|_| poisoned())?;
        if let Some(client) = clients.get(&key) {
            return Ok(client.clone());
        }
        let settings = self.settings.read().map_err(|_| poisoned())?;
        let password = if settings.proxy.has_password {
            secrets.get(&proxy_account(workspace_id))?
        } else {
            None
        };
        let identity = match key.identity {
            Some(id) => settings
                .client_certificates
                .iter()
                .find(|c| c.id == id)
                .map(network::load_identity)
                .transpose()?,
            None => None,
        };
        let client = build_client(ClientOptions {
            verify_tls: key.verify_tls,
            follow_redirects: key.follow_redirects,
            jar: key.use_cookies.then(|| jar.clone()),
            proxy: network::build_proxy(&settings.proxy, password.as_deref())?,
            roots: network::load_roots(&settings)?,
            identity,
        })?;
        clients.insert(key, client.clone());
        Ok(client)
    }
}
