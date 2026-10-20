use serde_json::Value;

use super::operation::{Context, Operation};
use super::schema::{Doc, as_text};
use crate::models::request::{Body, KeyValue, MultipartPart};

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

pub fn json_body(ctx: &Context, schema: Option<&Value>, example: Option<&Value>) -> Body {
    let value = match (example, schema) {
        _ if !ctx.example_bodies => {
            return Body::Json {
                text: String::new(),
            };
        }
        (Some(e), _) => e.clone(),
        (None, Some(s)) => ctx.doc.example(s),
        (None, None) => Value::Null,
    };
    let text = if value.is_null() {
        String::new()
    } else {
        serde_json::to_string_pretty(&value).unwrap_or_default()
    };
    Body::Json { text }
}

fn media_example<'a>(doc: &Doc<'a>, media: &'a Value) -> Option<&'a Value> {
    media.get("example").or_else(|| {
        let first = media.get("examples")?.as_object()?.values().next()?;
        doc.resolve(first).get("value")
    })
}

pub fn request_body_of(ctx: &mut Context, request_body: &Value) -> Body {
    let Some(content) = request_body
        .get("content")
        .and_then(Value::as_object)
        .filter(|c| !c.is_empty())
    else {
        return Body::None;
    };
    let doc = ctx.doc;
    let pick = |want: &dyn Fn(&str) -> bool| content.iter().find(|(k, _)| want(k));
    if let Some((_, media)) = pick(&|k| k == "application/json" || k.ends_with("+json")) {
        return json_body(ctx, media.get("schema"), media_example(doc, media));
    }
    let fields = |media: &Value| -> Vec<(String, Value, bool)> {
        let schema = doc.resolve(media.get("schema").unwrap_or(&Value::Null));
        let example = doc.example(schema);
        schema
            .get("properties")
            .and_then(Value::as_object)
            .map(|props| {
                props
                    .iter()
                    .map(|(k, p)| {
                        let binary = matches!(text(doc.resolve(p), "format"), "binary" | "byte");
                        (
                            k.clone(),
                            example.get(k).cloned().unwrap_or(Value::Null),
                            binary,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    if let Some((_, media)) = pick(&|k| k == "application/x-www-form-urlencoded") {
        let rows = fields(media)
            .into_iter()
            .map(|(key, v, _)| KeyValue {
                key,
                value: as_text(&v),
                enabled: true,
                description: String::new(),
            })
            .collect();
        return Body::FormUrlEncoded { fields: rows };
    }
    if let Some((_, media)) = pick(&|k| k.starts_with("multipart/")) {
        let parts = fields(media)
            .into_iter()
            .map(|(key, v, binary)| match binary {
                true => MultipartPart::File {
                    key,
                    path: String::new(),
                    enabled: true,
                },
                false => MultipartPart::Text {
                    key,
                    value: as_text(&v),
                    enabled: true,
                },
            })
            .collect();
        return Body::Multipart { parts };
    }
    let Some((content_type, media)) = content.iter().next() else {
        return Body::None;
    };
    let text = media_example(doc, media).map(as_text).unwrap_or_default();
    Body::Raw {
        text,
        content_type: content_type.clone(),
    }
}

pub fn swagger_form(op: &Operation, form: Vec<(&Value, KeyValue)>) -> Body {
    let multipart = op
        .op
        .get("consumes")
        .and_then(Value::as_array)
        .is_some_and(|c| {
            c.iter()
                .any(|v| v.as_str().is_some_and(|s| s.starts_with("multipart/")))
        })
        || form.iter().any(|(p, _)| text(p, "type") == "file");
    if !multipart {
        return Body::FormUrlEncoded {
            fields: form.into_iter().map(|(_, r)| r).collect(),
        };
    }
    let parts = form
        .into_iter()
        .map(|(p, r)| match text(p, "type") {
            "file" => MultipartPart::File {
                key: r.key,
                path: String::new(),
                enabled: true,
            },
            _ => MultipartPart::Text {
                key: r.key,
                value: r.value,
                enabled: true,
            },
        })
        .collect();
    Body::Multipart { parts }
}
