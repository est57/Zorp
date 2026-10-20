use std::collections::{BTreeSet, HashMap};

use crate::error::{AppError, AppResult};
use crate::models::request::{Auth, Body, KeyValue, MultipartPart, RequestSpec};

const MAX_DEPTH: u8 = 5;

/// Variable values ordered from lowest to highest priority.
pub struct Variables(HashMap<String, String>);

impl Variables {
    pub fn from_layers<I: IntoIterator<Item = (String, String)>>(layers: Vec<I>) -> Self {
        let mut map = HashMap::new();
        for layer in layers {
            map.extend(layer);
        }
        Self(map)
    }

    fn lookup(&self, name: &str) -> Option<String> {
        match name {
            "$uuid" => Some(uuid::Uuid::new_v4().to_string()),
            "$timestamp" => Some((crate::db::now_ms() / 1000).to_string()),
            "$randomInt" => Some((uuid::Uuid::new_v4().as_u128() % 1000).to_string()),
            _ => self.0.get(name).cloned(),
        }
    }

    pub fn resolve(&self, input: &str, missing: &mut BTreeSet<String>) -> String {
        self.resolve_depth(input, missing, 0)
    }

    fn resolve_depth(&self, input: &str, missing: &mut BTreeSet<String>, depth: u8) -> String {
        let mut out = String::with_capacity(input.len());
        let mut rest = input;
        while let Some(start) = rest.find("{{") {
            let Some(len) = rest[start + 2..].find("}}") else {
                break;
            };
            let name = rest[start + 2..start + 2 + len].trim();
            out.push_str(&rest[..start]);
            match self.lookup(name) {
                Some(value) if depth < MAX_DEPTH => {
                    out.push_str(&self.resolve_depth(&value, missing, depth + 1))
                }
                Some(value) => out.push_str(&value),
                None => {
                    missing.insert(name.to_string());
                    out.push_str(&rest[start..start + len + 4]);
                }
            }
            rest = &rest[start + len + 4..];
        }
        out.push_str(rest);
        out
    }
}

fn resolve_pairs(
    vars: &Variables,
    items: &[KeyValue],
    missing: &mut BTreeSet<String>,
) -> Vec<KeyValue> {
    items
        .iter()
        .filter(|kv| kv.enabled && !kv.key.trim().is_empty())
        .map(|kv| KeyValue {
            key: vars.resolve(&kv.key, missing),
            value: vars.resolve(&kv.value, missing),
            enabled: true,
            description: String::new(),
        })
        .collect()
}

fn resolve_body(vars: &Variables, body: &Body, missing: &mut BTreeSet<String>) -> Body {
    match body {
        Body::None => Body::None,
        Body::Json { text } => Body::Json {
            text: vars.resolve(text, missing),
        },
        Body::Raw { text, content_type } => Body::Raw {
            text: vars.resolve(text, missing),
            content_type: content_type.clone(),
        },
        Body::FormUrlEncoded { fields } => Body::FormUrlEncoded {
            fields: resolve_pairs(vars, fields, missing),
        },
        Body::Multipart { parts } => Body::Multipart {
            parts: parts
                .iter()
                .filter_map(|part| match part {
                    MultipartPart::Text {
                        key,
                        value,
                        enabled: true,
                    } => Some(MultipartPart::Text {
                        key: vars.resolve(key, missing),
                        value: vars.resolve(value, missing),
                        enabled: true,
                    }),
                    MultipartPart::File {
                        key,
                        path,
                        enabled: true,
                    } => Some(MultipartPart::File {
                        key: vars.resolve(key, missing),
                        path: path.clone(),
                        enabled: true,
                    }),
                    _ => None,
                })
                .collect(),
        },
        Body::Binary { path } => Body::Binary { path: path.clone() },
        Body::Graphql { query, variables } => Body::Graphql {
            query: vars.resolve(query, missing),
            variables: vars.resolve(variables, missing),
        },
    }
}

fn resolve_auth(vars: &Variables, auth: &Auth, missing: &mut BTreeSet<String>) -> Auth {
    match auth {
        Auth::Bearer { token } => Auth::Bearer {
            token: vars.resolve(token, missing),
        },
        Auth::Basic { username, password } => Auth::Basic {
            username: vars.resolve(username, missing),
            password: vars.resolve(password, missing),
        },
        Auth::ApiKey {
            key,
            value,
            location,
        } => Auth::ApiKey {
            key: vars.resolve(key, missing),
            value: vars.resolve(value, missing),
            location: *location,
        },
        // OAuth settings are resolved separately when the token is requested.
        other => other.clone(),
    }
}

/// Returns a copy of the request with variables substituted and disabled rows removed.
pub fn resolve_spec(
    spec: &RequestSpec,
    auth: &Auth,
    vars: &Variables,
) -> AppResult<(RequestSpec, Auth)> {
    let mut missing = BTreeSet::new();
    let resolved = RequestSpec {
        url: vars.resolve(spec.url.trim(), &mut missing),
        params: resolve_pairs(vars, &spec.params, &mut missing),
        headers: resolve_pairs(vars, &spec.headers, &mut missing),
        body: resolve_body(vars, &spec.body, &mut missing),
        ..spec.clone()
    };
    let auth = resolve_auth(vars, auth, &mut missing);
    if missing.is_empty() {
        Ok((resolved, auth))
    } else {
        Err(AppError::UnresolvedVariables(missing.into_iter().collect()))
    }
}

#[cfg(test)]
#[path = "resolver_tests.rs"]
mod tests;
