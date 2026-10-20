use crate::error::AppResult;

/// Storage for secret values that must never be written to SQLite.
pub trait SecretStore: Send + Sync {
    fn get(&self, account: &str) -> AppResult<Option<String>>;
    fn set(&self, account: &str, value: &str) -> AppResult<()>;
    fn delete(&self, account: &str) -> AppResult<()>;
}

pub fn env_account(env_id: uuid::Uuid, key: &str) -> String {
    format!("env:{env_id}:{key}")
}

/// Secrets kept only in memory, for tests and the CLI, where no keychain is available.
#[derive(Default)]
pub struct MemoryStore(std::sync::Mutex<std::collections::HashMap<String, String>>);

impl MemoryStore {
    fn map(&self) -> std::sync::MutexGuard<'_, std::collections::HashMap<String, String>> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl SecretStore for MemoryStore {
    fn get(&self, account: &str) -> AppResult<Option<String>> {
        Ok(self.map().get(account).cloned())
    }

    fn set(&self, account: &str, value: &str) -> AppResult<()> {
        self.map().insert(account.into(), value.into());
        Ok(())
    }

    fn delete(&self, account: &str) -> AppResult<()> {
        self.map().remove(account);
        Ok(())
    }
}
