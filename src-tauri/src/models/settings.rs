use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct ProxySettings {
    pub enabled: bool,
    /// http://, https://, socks5:// or socks5h:// followed by host and port.
    pub url: String,
    pub username: String,
    /// Write-only; empty keeps the password stored in the keychain.
    pub password: String,
    pub has_password: bool,
    /// Comma-separated hosts that bypass the proxy, for example `localhost, *.internal`.
    pub bypass: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaCertificate {
    pub id: Uuid,
    pub path: String,
    pub enabled: bool,
}

/// PEM client certificate used for hosts matching `host` (exact or `*.domain`).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ClientCertificate {
    pub id: Uuid,
    pub host: String,
    pub cert_path: String,
    pub key_path: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct WorkspaceSettings {
    pub proxy: ProxySettings,
    pub ca_certificates: Vec<CaCertificate>,
    pub client_certificates: Vec<ClientCertificate>,
}
