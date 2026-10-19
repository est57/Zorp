use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum OAuthGrant {
    AuthorizationCode,
    ClientCredentials,
}

/// How client credentials are sent to the token endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ClientAuth {
    Basic,
    Body,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OAuth2Config {
    pub grant: OAuthGrant,
    pub auth_url: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub scope: String,
    pub use_pkce: bool,
    pub client_auth: ClientAuth,
    /// Loopback port for the redirect URI; 0 picks a free port.
    pub callback_port: u16,
    /// Identifies the stored token in the keychain.
    pub token_key: String,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TokenStatus {
    pub has_token: bool,
    pub has_refresh_token: bool,
    #[ts(type = "number | null")]
    pub expires_at: Option<i64>,
    pub scope: Option<String>,
}
