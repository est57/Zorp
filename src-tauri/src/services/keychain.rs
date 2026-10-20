//! Secrets in the operating system keychain, used by the desktop app.
use super::secrets::SecretStore;
use crate::error::{AppError, AppResult};

const SERVICE: &str = "dev.zorp.app";

pub struct KeychainStore;

fn entry(account: &str) -> AppResult<keyring::Entry> {
    keyring::Entry::new(SERVICE, account).map_err(|e| AppError::Secret(e.to_string()))
}

impl SecretStore for KeychainStore {
    fn get(&self, account: &str) -> AppResult<Option<String>> {
        match entry(account)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Secret(e.to_string())),
        }
    }

    fn set(&self, account: &str, value: &str) -> AppResult<()> {
        entry(account)?
            .set_password(value)
            .map_err(|e| AppError::Secret(e.to_string()))
    }

    fn delete(&self, account: &str) -> AppResult<()> {
        match entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Secret(e.to_string())),
        }
    }
}
