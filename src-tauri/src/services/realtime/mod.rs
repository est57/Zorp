pub mod sse;
pub mod sse_client;
pub mod tls;
pub mod transport;
pub mod websocket;

use std::collections::HashMap;
use std::sync::Mutex;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::realtime::RealtimeKind;

/// Instructions from the UI to a running connection.
pub enum Outgoing {
    Text(String),
    Close,
}

/// Emits one event to the UI: kind, data, SSE event name, binary flag.
pub type Emit = Box<dyn Fn(RealtimeKind, String, Option<String>, bool) + Send + Sync>;

/// Open connections keyed by the id chosen by the UI.
#[derive(Default)]
pub struct Registry(Mutex<HashMap<Uuid, mpsc::UnboundedSender<Outgoing>>>);

fn poisoned() -> AppError {
    AppError::Internal("Status koneksi tidak konsisten, mulai ulang aplikasi".into())
}

impl Registry {
    pub fn open(&self, id: Uuid) -> AppResult<mpsc::UnboundedReceiver<Outgoing>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut map = self.0.lock().map_err(|_| poisoned())?;
        if let Some(previous) = map.insert(id, tx) {
            let _ = previous.send(Outgoing::Close);
        }
        Ok(rx)
    }

    pub fn send(&self, id: Uuid, message: Outgoing) -> AppResult<()> {
        let map = self.0.lock().map_err(|_| poisoned())?;
        let tx = map
            .get(&id)
            .ok_or_else(|| AppError::Validation("Koneksi sudah tertutup".into()))?;
        tx.send(message)
            .map_err(|_| AppError::Validation("Koneksi sudah tertutup".into()))
    }

    pub fn remove(&self, id: Uuid) {
        if let Ok(mut map) = self.0.lock() {
            map.remove(&id);
        }
    }
}

#[cfg(test)]
mod network_tests;
#[cfg(test)]
mod test_servers;
#[cfg(test)]
mod tests;
