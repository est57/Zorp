mod body;
mod fields;
mod signed_auth;

#[cfg(test)]
pub fn parse_auth_value(value: &Value, warnings: &mut Vec<String>) -> Option<Auth> {
    fields::parse_auth(Some(value), warnings, "tes")
}

use serde_json::Value;
use uuid::Uuid;

use super::{ImportedCollection, ImportedNode, clean_name};
use crate::error::{AppError, AppResult};
use crate::models::request::{Auth, HttpMethod, RequestSettings, RequestSpec};
use crate::models::script::Scripts;
use body::parse_body;
use fields::{pairs, parse_auth, parse_url, str_of};

const MAX_DEPTH: usize = 32;

/// Reads `event` entries; Postman stores script source as an array of lines.
fn parse_scripts(events: Option<&Value>) -> Scripts {
    let mut scripts = Scripts::default();
    for event in events.and_then(Value::as_array).into_iter().flatten() {
        let source = match event.pointer("/script/exec") {
            Some(Value::Array(lines)) => lines
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("\n"),
            Some(Value::String(text)) => text.clone(),
            _ => continue,
        };
        match str_of(event, "listen") {
            "prerequest" => scripts.pre_request = source,
            "test" => scripts.test = source,
            _ => {}
        }
    }
    scripts
}

fn parse_method(value: &str, warnings: &mut Vec<String>, owner: &str) -> HttpMethod {
    serde_json::from_value(Value::String(value.to_ascii_uppercase())).unwrap_or_else(|_| {
        warnings.push(format!(
            "{owner}: method {value} belum didukung, diganti GET"
        ));
        HttpMethod::Get
    })
}

fn parse_request(name: String, request: &Value, warnings: &mut Vec<String>) -> RequestSpec {
    // Postman v2 allows a request to be just a URL string.
    let request = if request.is_string() {
        &serde_json::json!({ "url": request })
    } else {
        request
    };
    let (url, params) = parse_url(request.get("url"));
    let method = match str_of(request, "method") {
        "" => "GET",
        m => m,
    };
    RequestSpec {
        docs: String::new(),
        id: Uuid::now_v7(),
        method: parse_method(method, warnings, &name),
        url,
        params,
        headers: pairs(request.get("header")),
        body: parse_body(request.get("body"), warnings, &name),
        auth: parse_auth(request.get("auth"), warnings, &name).unwrap_or(Auth::Inherit),
        settings: RequestSettings::default(),
        mode: Default::default(),
        scripts: Default::default(),
        name,
    }
}

fn parse_items(
    items: Option<&Value>,
    depth: usize,
    warnings: &mut Vec<String>,
) -> AppResult<Vec<ImportedNode>> {
    if depth > MAX_DEPTH {
        return Err(AppError::Validation("Struktur folder terlalu dalam".into()));
    }
    let mut nodes = Vec::new();
    for item in items.and_then(Value::as_array).into_iter().flatten() {
        if let Some(request) = item.get("request") {
            let name = clean_name(str_of(item, "name"), "Request tanpa nama");
            let mut spec = parse_request(name, request, warnings);
            spec.scripts = parse_scripts(item.get("event"));
            nodes.push(ImportedNode::Request(Box::new(spec)));
        } else {
            let name = clean_name(str_of(item, "name"), "Folder tanpa nama");
            let auth = parse_auth(item.get("auth"), warnings, &name);
            let children = parse_items(item.get("item"), depth + 1, warnings)?;
            nodes.push(ImportedNode::Folder {
                name,
                auth,
                children,
            });
        }
    }
    Ok(nodes)
}

pub fn parse(json: &str) -> AppResult<ImportedCollection> {
    let root: Value = serde_json::from_str(json)
        .map_err(|_| AppError::Validation("File bukan JSON yang valid".into()))?;
    let schema = root
        .pointer("/info/schema")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !schema.contains("collection/v2") || root.get("item").is_none() {
        return Err(AppError::Validation(
            "Format tidak dikenali. Ekspor koleksi dari Postman sebagai Collection v2.1".into(),
        ));
    }
    let mut warnings = Vec::new();
    let name = clean_name(
        root.pointer("/info/name")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        "Koleksi impor",
    );
    let auth = parse_auth(root.get("auth"), &mut warnings, &name);
    let variables = pairs(root.get("variable"))
        .into_iter()
        .filter(|kv| kv.enabled && !kv.key.is_empty())
        .map(|kv| (kv.key, kv.value))
        .collect();
    let items = parse_items(root.get("item"), 0, &mut warnings)?;
    Ok(ImportedCollection {
        name,
        auth,
        variables,
        items,
        warnings,
    })
}

#[cfg(test)]
#[path = "../postman_tests.rs"]
mod tests;
