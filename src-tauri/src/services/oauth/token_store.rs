use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::models::oauth::TokenStatus;
use crate::services::secrets::SecretStore;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredToken {
    pub access_token: String,
    pub token_type: String,
    pub refresh_token: Option<String>,
    /// Unix time in milliseconds.
    pub expires_at: Option<i64>,
    pub scope: Option<String>,
}

const EXPIRY_SKEW_MS: i64 = 30_000;

fn account(token_key: &str) -> String {
    format!("oauth:{token_key}")
}

impl StoredToken {
    pub fn is_expired(&self, now_ms: i64) -> bool {
        self.expires_at
            .is_some_and(|at| at - EXPIRY_SKEW_MS <= now_ms)
    }

    pub fn status(&self) -> TokenStatus {
        TokenStatus {
            has_token: true,
            has_refresh_token: self.refresh_token.is_some(),
            expires_at: self.expires_at,
            scope: self.scope.clone(),
        }
    }
}

pub fn load(store: &dyn SecretStore, token_key: &str) -> AppResult<Option<StoredToken>> {
    Ok(store
        .get(&account(token_key))?
        .and_then(|json| serde_json::from_str(&json).ok()))
}

pub fn save(store: &dyn SecretStore, token_key: &str, token: &StoredToken) -> AppResult<()> {
    store.set(&account(token_key), &serde_json::to_string(token)?)
}

pub fn clear(store: &dyn SecretStore, token_key: &str) -> AppResult<()> {
    store.delete(&account(token_key))
}
