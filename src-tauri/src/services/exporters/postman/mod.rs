mod body;
mod fields;
mod signed_auth;

#[cfg(test)]
pub fn fields_auth(auth: &crate::models::request::Auth) -> Option<serde_json::Value> {
    fields::auth(auth)
}

use serde_json::{Value, json};
use uuid::Uuid;

use crate::models::collection::{Collection, Folder, SavedRequest};
use crate::models::realtime::RequestMode;
use crate::models::request::RequestSpec;
use crate::models::script::Scripts;

const SCHEMA: &str = "https://schema.getpostman.com/json/collection/v2.1.0/collection.json";

/// A collection as stored in Zorp, ready to convert.
pub struct CollectionData<'a> {
    pub collection: &'a Collection,
    pub variables: &'a [(String, String)],
    pub folders: &'a [Folder],
    pub requests: &'a [SavedRequest],
}

pub struct Exported {
    pub json: Value,
    pub requests: u32,
    /// Names of requests Postman collections cannot represent, such as WebSocket requests.
    pub skipped: Vec<String>,
}

fn script_events(scripts: &Scripts) -> Value {
    let event = |listen: &str, source: &str| json!({ "listen": listen, "script": { "type": "text/javascript", "exec": source.lines().collect::<Vec<_>>() } });
    let mut events = Vec::new();
    if !scripts.pre_request.trim().is_empty() {
        events.push(event("prerequest", &scripts.pre_request));
    }
    if !scripts.test.trim().is_empty() {
        events.push(event("test", &scripts.test));
    }
    Value::Array(events)
}

fn request_item(spec: &RequestSpec) -> Value {
    let mut request = json!({
        "method": format!("{:?}", spec.method).to_uppercase(),
        "header": fields::pairs(&spec.headers),
        "url": { "raw": spec.url, "query": fields::pairs(&spec.params) },
    });
    if let Some(body) = body::body(&spec.body) {
        request["body"] = body;
    }
    if let Some(auth) = fields::auth(&spec.auth) {
        request["auth"] = auth;
    }
    let mut item = json!({ "name": spec.name, "request": request });
    add_events(&mut item, &spec.scripts);
    item
}

fn add_events(item: &mut Value, scripts: &Scripts) {
    let events = script_events(scripts);
    if events.as_array().is_some_and(|e| !e.is_empty()) {
        item["event"] = events;
    }
}

fn items(data: &CollectionData<'_>, parent: Option<Uuid>, out: &mut Exported) -> Vec<Value> {
    let mut folders: Vec<&Folder> = data
        .folders
        .iter()
        .filter(|f| f.parent_id == parent)
        .collect();
    folders.sort_by(|a, b| a.sort_order.total_cmp(&b.sort_order));
    let mut requests: Vec<&SavedRequest> = data
        .requests
        .iter()
        .filter(|r| r.folder_id == parent)
        .collect();
    requests.sort_by(|a, b| a.sort_order.total_cmp(&b.sort_order));

    let mut list = Vec::new();
    for folder in folders {
        let mut item = json!({ "name": folder.name, "item": items(data, Some(folder.id), out) });
        if let Some(auth) = folder.auth.as_ref().and_then(fields::auth) {
            item["auth"] = auth;
        }
        add_events(&mut item, &folder.scripts);
        list.push(item);
    }
    for saved in requests {
        if saved.spec.mode == RequestMode::Websocket {
            out.skipped.push(saved.spec.name.clone());
            continue;
        }
        out.requests += 1;
        list.push(request_item(&saved.spec));
    }
    list
}

/// Converts a collection to Postman Collection v2.1 JSON.
pub fn export(data: &CollectionData<'_>) -> Exported {
    let mut out = Exported {
        json: Value::Null,
        requests: 0,
        skipped: Vec::new(),
    };
    let item = items(data, None, &mut out);
    let mut root = json!({
        "info": { "_postman_id": data.collection.id, "name": data.collection.name, "schema": SCHEMA },
        "item": item,
    });
    if let Some(auth) = data.collection.auth.as_ref().and_then(fields::auth) {
        root["auth"] = auth;
    }
    add_events(&mut root, &data.collection.scripts);
    if !data.variables.is_empty() {
        root["variable"] = Value::Array(
            data.variables
                .iter()
                .map(|(k, v)| json!({ "key": k, "value": v }))
                .collect(),
        );
    }
    out.json = root;
    out
}

#[cfg(test)]
#[path = "../postman_tests.rs"]
mod tests;
