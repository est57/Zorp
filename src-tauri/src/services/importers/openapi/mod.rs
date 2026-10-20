//! Converts OpenAPI 3.x and Swagger 2.0 documents (JSON or YAML) into a collection.
mod body;
mod operation;
mod schema;
mod security;
mod summary;

pub use summary::summarize;

use std::collections::BTreeMap;

use serde_json::Value;

use super::{ImportedCollection, ImportedNode, clean_name};
use crate::error::{AppError, AppResult};
use crate::models::openapi::{GroupBy, OpenApiOptions};
use operation::{Context, METHODS, Operation};
use schema::Doc;

const MAX_OPERATIONS: usize = 5_000;

/// Parses JSON or YAML and checks that it is an OpenAPI or Swagger document.
pub fn load(text: &str) -> AppResult<Value> {
    let value: Value = if text.trim_start().starts_with('{') {
        serde_json::from_str(text).map_err(|e| invalid(&e.to_string()))?
    } else {
        serde_saphyr::from_str_with_options(text, yaml_options())
            .map_err(|e| invalid(&e.to_string()))?
    };
    format_of(&value)?;
    Ok(value)
}

/// Large public specs (GitHub's has about a million nodes) need more than the parser's
/// default budget; the 20 MB input cap still bounds the work.
fn yaml_options() -> serde_saphyr::Options {
    let mut budget = serde_saphyr::Budget::default();
    budget.max_nodes = 5_000_000;
    budget.max_events = 20_000_000;
    let mut options = serde_saphyr::Options::default();
    options.budget = Some(budget);
    options
}

fn invalid(reason: &str) -> AppError {
    AppError::Validation(format!("File bukan JSON atau YAML yang valid: {reason}"))
}

pub(super) fn format_of(root: &Value) -> AppResult<String> {
    let text = |k: &str| root.get(k).and_then(Value::as_str);
    match (text("openapi"), text("swagger")) {
        (Some(v), _) if v.starts_with('3') => Ok(format!("OpenAPI {}", v.get(..3).unwrap_or(v))),
        (None, Some("2.0")) => Ok("Swagger 2.0".into()),
        _ => Err(AppError::Validation(
            "Dokumen bukan OpenAPI 3.x atau Swagger 2.0 (field openapi atau swagger tidak ditemukan)".into(),
        )),
    }
}

pub(super) fn operations(root: &Value) -> Vec<Operation<'_>> {
    let mut ops = Vec::new();
    for (path, item) in root
        .get("paths")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        let item = Doc { root }.resolve(item);
        for (key, method) in METHODS {
            if let Some(op) = item.get(key).filter(|o| o.is_object()) {
                ops.push(Operation {
                    path,
                    method,
                    op,
                    item,
                });
            }
        }
    }
    ops.truncate(MAX_OPERATIONS);
    ops
}

pub(super) fn group_of(op: &Operation, by: GroupBy) -> Option<String> {
    match by {
        GroupBy::Tag => op.tag().map(str::to_string),
        GroupBy::Path => op
            .path
            .split('/')
            .find(|s| !s.is_empty() && !s.starts_with('{'))
            .map(str::to_string),
    }
}

pub fn convert(root: &Value, options: &OpenApiOptions) -> AppResult<ImportedCollection> {
    format_of(root)?;
    let doc = Doc { root };
    let mut ctx = Context {
        doc: &doc,
        example_bodies: options.example_bodies,
        variables: BTreeMap::new(),
        warnings: Vec::new(),
    };
    let ops = operations(root);
    if ops.is_empty() {
        return Err(AppError::Validation("Dokumen tidak berisi endpoint".into()));
    }
    // Groups follow the order of the document's `tags` list, then first appearance.
    let mut order: Vec<String> = root
        .get("tags")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|t| t.get("name").and_then(Value::as_str).map(str::to_string))
        .collect();
    let mut grouped: BTreeMap<String, Vec<ImportedNode>> = BTreeMap::new();
    let mut loose = Vec::new();
    for op in &ops {
        let spec = operation::build(&mut ctx, op);
        match group_of(op, options.group_by) {
            Some(group) => {
                if !order.contains(&group) {
                    order.push(group.clone());
                }
                grouped
                    .entry(group)
                    .or_default()
                    .push(ImportedNode::Request(Box::new(spec)));
            }
            None => loose.push(ImportedNode::Request(Box::new(spec))),
        }
    }
    let mut items: Vec<ImportedNode> = order
        .into_iter()
        .filter_map(|name| {
            let children = grouped.remove(&name)?;
            Some(ImportedNode::Folder {
                name: clean_name(&name, "Folder"),
                auth: None,
                scripts: Default::default(),
                children,
            })
        })
        .collect();
    items.extend(loose);

    let auth = match (options.collection_auth, root.get("security")) {
        (true, Some(req)) => security::requirement(&doc, req, &mut ctx.warnings),
        _ => None,
    };
    let mut variables = vec![(
        "baseUrl".to_string(),
        options.base_url.trim().trim_end_matches('/').to_string(),
    )];
    variables.extend(ctx.variables);
    if options.base_url.trim().is_empty() {
        ctx.warnings
            .push("Base URL kosong; isi variabel baseUrl di koleksi".into());
    }
    Ok(ImportedCollection {
        scripts: Default::default(),
        name: clean_name(&summary::info(root, "title"), "API"),
        auth,
        variables,
        items,
        warnings: unique(ctx.warnings),
    })
}

/// The same scheme warning would otherwise repeat for every operation that uses it.
fn unique(warnings: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    warnings
        .into_iter()
        .filter(|w| seen.insert(w.clone()))
        .collect()
}
