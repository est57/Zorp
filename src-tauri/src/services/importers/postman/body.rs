use serde_json::Value;

use super::fields::{enabled, pairs, str_of, text_of};
use crate::models::request::{Body, MultipartPart};

fn raw_content_type(language: &str) -> &'static str {
    match language {
        "xml" => "application/xml",
        "html" => "text/html",
        "javascript" => "application/javascript",
        _ => "text/plain",
    }
}

fn file_src(value: &Value) -> String {
    match value.get("src") {
        Some(Value::Array(items)) => items
            .first()
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    }
}

pub fn parse_body(value: Option<&Value>, warnings: &mut Vec<String>, owner: &str) -> Body {
    let Some(body) = value else { return Body::None };
    match str_of(body, "mode") {
        "raw" => {
            let text = text_of(body, "raw");
            let language = body
                .pointer("/options/raw/language")
                .and_then(Value::as_str)
                .unwrap_or("text");
            if language == "json" {
                Body::Json { text }
            } else {
                Body::Raw {
                    text,
                    content_type: raw_content_type(language).into(),
                }
            }
        }
        "urlencoded" => Body::FormUrlEncoded {
            fields: pairs(body.get("urlencoded")),
        },
        "formdata" => Body::Multipart {
            parts: body
                .get("formdata")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .map(|item| {
                            let (key, enabled) = (text_of(item, "key"), enabled(item));
                            if str_of(item, "type") == "file" {
                                MultipartPart::File {
                                    key,
                                    path: file_src(item),
                                    enabled,
                                }
                            } else {
                                MultipartPart::Text {
                                    key,
                                    value: text_of(item, "value"),
                                    enabled,
                                }
                            }
                        })
                        .collect()
                })
                .unwrap_or_default(),
        },
        "file" => Body::Binary {
            path: body.get("file").map(file_src).unwrap_or_default(),
        },
        "graphql" => {
            let graphql = body.get("graphql").cloned().unwrap_or(Value::Null);
            let variables = match graphql.get("variables") {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Null) | None => String::new(),
                Some(other) => serde_json::to_string_pretty(other).unwrap_or_default(),
            };
            Body::Graphql {
                query: text_of(&graphql, "query"),
                variables,
            }
        }
        "" => Body::None,
        other => {
            warnings.push(format!("{owner}: body {other} belum didukung"));
            Body::None
        }
    }
}
