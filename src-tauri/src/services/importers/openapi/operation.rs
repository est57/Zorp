use std::collections::BTreeMap;

use serde_json::Value;
use uuid::Uuid;

use super::body::{json_body, request_body_of, swagger_form};
use super::schema::{Doc, as_text};
use super::security;
use crate::models::request::{Auth, Body, HttpMethod, KeyValue, RequestSpec};
use crate::services::importers::clean_name;

pub const METHODS: [(&str, HttpMethod); 7] = [
    ("get", HttpMethod::Get),
    ("post", HttpMethod::Post),
    ("put", HttpMethod::Put),
    ("patch", HttpMethod::Patch),
    ("delete", HttpMethod::Delete),
    ("head", HttpMethod::Head),
    ("options", HttpMethod::Options),
];

/// What every operation of one document shares while it is converted.
pub struct Context<'a, 'b> {
    pub doc: &'b Doc<'a>,
    pub example_bodies: bool,
    /// Collection variables collected along the way, such as path parameters.
    pub variables: BTreeMap<String, String>,
    pub warnings: Vec<String>,
}

pub struct Operation<'a> {
    pub path: &'a str,
    pub method: HttpMethod,
    pub op: &'a Value,
    pub item: &'a Value,
}

impl Operation<'_> {
    pub fn name(&self) -> String {
        let label = ["summary", "operationId"]
            .iter()
            .find_map(|k| {
                self.op
                    .get(*k)
                    .and_then(Value::as_str)
                    .filter(|s| !s.trim().is_empty())
            })
            .map(str::to_string)
            .unwrap_or_else(|| {
                format!(
                    "{} {}",
                    format!("{:?}", self.method).to_uppercase(),
                    self.path
                )
            });
        clean_name(&label, "Request")
    }

    pub fn tag(&self) -> Option<&str> {
        self.op.get("tags")?.as_array()?.first()?.as_str()
    }
}

/// Builds a request for one operation. Path parameters become `{{name}}` variables.
pub fn build(ctx: &mut Context, op: &Operation) -> RequestSpec {
    let doc = ctx.doc;
    let mut params: Vec<(&Value, String)> = Vec::new();
    for source in [op.item.get("parameters"), op.op.get("parameters")]
        .into_iter()
        .flatten()
    {
        for raw in source.as_array().into_iter().flatten() {
            let p = doc.resolve(raw);
            let key = format!("{}:{}", text(p, "in"), text(p, "name"));
            params.retain(|(_, k)| *k != key);
            params.push((p, key));
        }
    }
    let mut url = format!("{{{{baseUrl}}}}{}", path_template(op.path));
    let (mut query, mut headers, mut form) = (Vec::new(), Vec::new(), Vec::new());
    let mut body = Body::None;
    for (p, _) in &params {
        let name = text(p, "name").to_string();
        let row = || KeyValue {
            key: name.clone(),
            value: param_example(doc, p),
            enabled: p.get("required").and_then(Value::as_bool).unwrap_or(false),
            description: text(p, "description").chars().take(200).collect(),
        };
        match text(p, "in") {
            "path" => {
                ctx.variables
                    .entry(name.clone())
                    .or_insert_with(|| param_example(doc, p));
            }
            "query" => query.push(row()),
            "header"
                if !matches!(
                    name.to_ascii_lowercase().as_str(),
                    "accept" | "content-type" | "authorization"
                ) =>
            {
                headers.push(row())
            }
            "body" => body = json_body(ctx, p.get("schema"), None),
            "formData" => form.push((*p, row())),
            "cookie" => ctx
                .warnings
                .push(format!("{}: parameter cookie {name} dilewati", op.name())),
            _ => {}
        }
    }
    if !form.is_empty() {
        body = swagger_form(op, form);
    }
    if let Some(request_body) = op.op.get("requestBody") {
        body = request_body_of(ctx, doc.resolve(request_body));
    }
    let enabled: Vec<String> = query
        .iter()
        .filter(|q| q.enabled)
        .map(|q| format!("{}={}", q.key, q.value))
        .collect();
    if !enabled.is_empty() {
        url = format!("{url}?{}", enabled.join("&"));
    }
    let auth = match op.op.get("security") {
        Some(Value::Array(list)) if list.is_empty() => Auth::None,
        Some(req) if Some(req) != doc.root.get("security") => {
            security::requirement(doc, req, &mut ctx.warnings).unwrap_or(Auth::Inherit)
        }
        _ => Auth::Inherit,
    };
    RequestSpec {
        docs: String::new(),
        id: Uuid::now_v7(),
        name: op.name(),
        method: op.method,
        url,
        params: query,
        headers,
        body,
        auth,
        settings: Default::default(),
        scripts: Default::default(),
        mode: Default::default(),
    }
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

/// `/pets/{petId}` becomes `/pets/{{petId}}`.
fn path_template(path: &str) -> String {
    path.replace('{', "{{").replace('}', "}}")
}

fn param_example(doc: &Doc, p: &Value) -> String {
    if let Some(e) = p.get("example") {
        return as_text(e);
    }
    // OpenAPI 3 keeps the type under `schema`; Swagger 2 puts it on the parameter itself.
    as_text(&doc.example(p.get("schema").unwrap_or(p)))
}
