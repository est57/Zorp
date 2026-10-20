pub mod openapi;
pub mod postman;

#[cfg(test)]
mod openapi_fixtures;
#[cfg(test)]
mod openapi_tests;

use crate::models::request::{Auth, RequestSpec};
use crate::models::script::Scripts;

// Import trees are built once and saved right away, so variant size does not matter here.
#[allow(clippy::large_enum_variant)]
pub enum ImportedNode {
    Folder {
        name: String,
        auth: Option<Auth>,
        scripts: Scripts,
        children: Vec<ImportedNode>,
    },
    Request(Box<RequestSpec>),
}

pub struct ImportedCollection {
    pub name: String,
    pub auth: Option<Auth>,
    pub scripts: Scripts,
    pub variables: Vec<(String, String)>,
    pub items: Vec<ImportedNode>,
    pub warnings: Vec<String>,
}

const MAX_NAME_CHARS: usize = 200;

/// Trims and truncates names so they always pass storage validation.
pub fn clean_name(name: &str, fallback: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return fallback.to_string();
    }
    trimmed.chars().take(MAX_NAME_CHARS).collect()
}
