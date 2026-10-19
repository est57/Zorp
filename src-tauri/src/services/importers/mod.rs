pub mod postman;

use crate::models::request::{Auth, RequestSpec};

pub enum ImportedNode {
    Folder {
        name: String,
        auth: Option<Auth>,
        children: Vec<ImportedNode>,
    },
    Request(Box<RequestSpec>),
}

pub struct ImportedCollection {
    pub name: String,
    pub auth: Option<Auth>,
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
