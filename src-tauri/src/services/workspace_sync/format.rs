//! File shapes of a workspace folder. Fields that hold default values are left out
//! so files stay short and diffs stay small.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::realtime::RequestMode;
use crate::models::request::{Auth, Body, HttpMethod, KeyValue, RequestSettings};
use crate::models::rules::{Assertion, VarRule};
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
    #[serde(default, skip_serializing_if = "is_empty_scripts")]
    pub scripts: Scripts,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub docs: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderFile {
    pub id: Uuid,
    pub name: String,
    pub order: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
    #[serde(default, skip_serializing_if = "is_empty_scripts")]
    pub scripts: Scripts,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub docs: String,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub asserts: Vec<Assertion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vars: Vec<VarRule>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub docs: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<ExampleFile>,
}

/// A saved response, kept in the request file so it is reviewed together with it.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleFile {
    pub id: Uuid,
    pub name: String,
    pub status: u16,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub status_text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<Pair>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentFile {
    pub id: Uuid,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<EnvVariable>,
}

/// Workspace-wide variables, kept beside the environments as `_global.yaml`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalFile {
    #[serde(default)]
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
