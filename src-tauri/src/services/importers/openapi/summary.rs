use serde_json::Value;

use super::{format_of, group_of, operations, schema};
use crate::error::AppResult;
use crate::models::openapi::{GroupBy, OpenApiGroup, OpenApiSample, OpenApiSummary};
use crate::services::importers::clean_name;

const SAMPLE_COUNT: usize = 6;

/// Server URLs with `{variable}` placeholders filled from their defaults.
pub fn servers(root: &Value) -> Vec<String> {
    if let Some(host) = root.get("host").and_then(Value::as_str) {
        let scheme = root
            .pointer("/schemes/0")
            .and_then(Value::as_str)
            .unwrap_or("https");
        let base = root.get("basePath").and_then(Value::as_str).unwrap_or("");
        return vec![format!("{scheme}://{host}{}", base.trim_end_matches('/'))];
    }
    let mut urls = Vec::new();
    for server in root
        .get("servers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(mut url) = server
            .get("url")
            .and_then(Value::as_str)
            .map(str::to_string)
        else {
            continue;
        };
        for (name, var) in server
            .get("variables")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            let default = var.get("default").map(schema::as_text).unwrap_or_default();
            url = url.replace(&format!("{{{name}}}"), &default);
        }
        urls.push(url.trim_end_matches('/').to_string());
    }
    urls
}

pub fn info(root: &Value, key: &str) -> String {
    root.pointer(&format!("/info/{key}"))
        .map(schema::as_text)
        .unwrap_or_default()
}

fn scheme_names(root: &Value) -> Vec<String> {
    let schemes = root
        .pointer("/components/securitySchemes")
        .or_else(|| root.get("securityDefinitions"));
    schemes
        .and_then(Value::as_object)
        .map(|s| s.keys().cloned().collect())
        .unwrap_or_default()
}

pub fn summarize(root: &Value) -> AppResult<OpenApiSummary> {
    let ops = operations(root);
    let mut groups: Vec<OpenApiGroup> = Vec::new();
    for op in &ops {
        let name = group_of(op, GroupBy::Tag).unwrap_or_else(|| "Tanpa tag".into());
        match groups.iter_mut().find(|g| g.name == name) {
            Some(g) => g.count += 1,
            None => groups.push(OpenApiGroup { name, count: 1 }),
        }
    }
    Ok(OpenApiSummary {
        title: clean_name(&info(root, "title"), "API"),
        version: info(root, "version"),
        format: format_of(root)?,
        endpoints: ops.len() as u32,
        groups,
        auth: scheme_names(root),
        servers: servers(root),
        samples: ops
            .iter()
            .take(SAMPLE_COUNT)
            .map(|op| OpenApiSample {
                method: format!("{:?}", op.method).to_uppercase(),
                name: op.name(),
                group: group_of(op, GroupBy::Tag),
            })
            .collect(),
    })
}
