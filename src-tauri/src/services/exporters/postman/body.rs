use serde_json::{Value, json};

use super::fields::pairs;
use crate::models::request::{Body, MultipartPart};

fn raw_language(content_type: &str) -> &'static str {
    match content_type {
        t if t.contains("json") => "json",
        t if t.contains("xml") => "xml",
        t if t.contains("html") => "html",
        t if t.contains("javascript") => "javascript",
        _ => "text",
    }
}

/// Postman request body, or `None` when the request has no body.
pub fn body(body: &Body) -> Option<Value> {
    Some(match body {
        Body::None => return None,
        Body::Json { text } => {
            json!({ "mode": "raw", "raw": text, "options": { "raw": { "language": "json" } } })
        }
        Body::Raw { text, content_type } => {
            json!({ "mode": "raw", "raw": text, "options": { "raw": { "language": raw_language(content_type) } } })
        }
        Body::FormUrlEncoded { fields } => {
            json!({ "mode": "urlencoded", "urlencoded": pairs(fields) })
        }
        Body::Multipart { parts } => {
            let items: Vec<Value> = parts
                .iter()
                .map(|part| match part {
                    MultipartPart::Text {
                        key,
                        value,
                        enabled,
                    } => {
                        json!({ "key": key, "value": value, "type": "text", "disabled": !enabled })
                    }
                    MultipartPart::File { key, path, enabled } => {
                        json!({ "key": key, "src": path, "type": "file", "disabled": !enabled })
                    }
                })
                .collect();
            json!({ "mode": "formdata", "formdata": items })
        }
        Body::Binary { path } => json!({ "mode": "file", "file": { "src": path } }),
        Body::Graphql { query, variables } => {
            json!({ "mode": "graphql", "graphql": { "query": query, "variables": variables } })
        }
    })
}
