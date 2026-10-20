//! File shapes of a workspace folder. Fields that hold default values are left out
//! so files stay short and diffs stay small.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::realtime::RequestMode;
use crate::models::request::{Auth, Body, HttpMethod, KeyValue, RequestSettings, RequestSpec};
use crate::models::script::Scripts;

pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceFile {
    pub zorp: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionFile {
    pub id: Uuid,
    pub name: String,
    pub order: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<Pair>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderFile {
    pub id: Uuid,
    pub name: String,
    pub order: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestFile {
    pub id: Uuid,
    pub name: String,
    pub order: f64,
    pub method: HttpMethod,
    pub url: String,
    #[serde(default, skip_serializing_if = "is_http")]
    pub mode: RequestMode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<KeyValue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<KeyValue>,
    #[serde(default = "no_body", skip_serializing_if = "is_no_body")]
    pub body: Body,
    #[serde(default = "no_auth", skip_serializing_if = "is_no_auth")]
    pub auth: Auth,
    #[serde(default, skip_serializing_if = "is_default_settings")]
    pub settings: RequestSettings,
    #[serde(default, skip_serializing_if = "is_empty_scripts")]
    pub scripts: Scripts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentFile {
    pub id: Uuid,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<EnvVariable>,
}

/// Secret variables are listed by key only; their values stay in the local keychain.
#[derive(Debug, Serialize, Deserialize)]
pub struct EnvVariable {
    pub key: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub value: String,
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub secret: bool,
}

impl RequestFile {
    pub fn from_spec(spec: RequestSpec, order: f64) -> Self {
        Self {
            id: spec.id,
            name: spec.name,
            order,
            method: spec.method,
            url: spec.url,
            mode: spec.mode,
            params: spec.params,
            headers: spec.headers,
            body: spec.body,
            auth: spec.auth,
            settings: spec.settings,
            scripts: spec.scripts,
        }
    }

    pub fn into_spec(self) -> (RequestSpec, f64) {
        let spec = RequestSpec {
            id: self.id,
            name: self.name,
            method: self.method,
            url: self.url,
            params: self.params,
            headers: self.headers,
            body: self.body,
            auth: self.auth,
            settings: self.settings,
            scripts: self.scripts,
            mode: self.mode,
        };
        (spec, self.order)
    }
}

fn is_http(mode: &RequestMode) -> bool {
    *mode == RequestMode::Http
}
fn no_body() -> Body {
    Body::None
}
fn is_no_body(body: &Body) -> bool {
    matches!(body, Body::None)
}
fn no_auth() -> Auth {
    Auth::None
}
fn is_no_auth(auth: &Auth) -> bool {
    matches!(auth, Auth::None)
}
fn is_default_settings(s: &RequestSettings) -> bool {
    let d = RequestSettings::default();
    (
        s.timeout_ms,
        s.follow_redirects,
        s.verify_tls,
        s.use_cookies,
    ) == (
        d.timeout_ms,
        d.follow_redirects,
        d.verify_tls,
        d.use_cookies,
    )
}
fn is_empty_scripts(s: &Scripts) -> bool {
    s.pre_request.is_empty() && s.test.is_empty()
}
fn yes() -> bool {
    true
}
fn is_true(v: &bool) -> bool {
    *v
}
fn is_false(v: &bool) -> bool {
    !*v
}
