use serde_json::Value;

use crate::models::oauth::{ClientAuth, OAuth2Config, OAuthGrant};
use crate::models::request::{ApiKeyLocation, Auth, KeyValue};

pub fn str_of<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}

pub fn text_of(value: &Value, key: &str) -> String {
    match value.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

pub fn enabled(value: &Value) -> bool {
    !value
        .get("disabled")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub fn pairs(value: Option<&Value>) -> Vec<KeyValue> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| KeyValue {
                    key: text_of(item, "key"),
                    value: text_of(item, "value"),
                    enabled: enabled(item),
                    description: item
                        .get("description")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .into(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Reads a value from Postman's `[{ key, value }]` auth parameter lists.
fn auth_param(list: Option<&Value>, key: &str) -> String {
    list.and_then(Value::as_array)
        .and_then(|items| items.iter().find(|i| str_of(i, "key") == key))
        .map(|i| text_of(i, "value"))
        .unwrap_or_default()
}

pub fn parse_auth(value: Option<&Value>, warnings: &mut Vec<String>, owner: &str) -> Option<Auth> {
    let value = value?;
    let kind = str_of(value, "type");
    Some(match kind {
        "noauth" => Auth::None,
        "bearer" => Auth::Bearer {
            token: auth_param(value.get("bearer"), "token"),
        },
        "basic" => Auth::Basic {
            username: auth_param(value.get("basic"), "username"),
            password: auth_param(value.get("basic"), "password"),
        },
        "apikey" => Auth::ApiKey {
            key: auth_param(value.get("apikey"), "key"),
            value: auth_param(value.get("apikey"), "value"),
            location: if auth_param(value.get("apikey"), "in") == "query" {
                ApiKeyLocation::Query
            } else {
                ApiKeyLocation::Header
            },
        },
        "oauth2" => return oauth2(value.get("oauth2"), warnings, owner),
        "" => return None,
        other => {
            warnings.push(format!(
                "{owner}: auth {other} belum didukung, diganti tanpa auth"
            ));
            Auth::None
        }
    })
}

pub fn parse_url(value: Option<&Value>) -> (String, Vec<KeyValue>) {
    match value {
        Some(Value::String(raw)) => (raw.clone(), Vec::new()),
        Some(url @ Value::Object(_)) => {
            let raw = str_of(url, "raw").to_string();
            (raw, pairs(url.get("query")))
        }
        _ => (String::new(), Vec::new()),
    }
}

fn oauth2(params: Option<&Value>, warnings: &mut Vec<String>, owner: &str) -> Option<Auth> {
    let get = |key: &str| auth_param(params, key);
    let grant = match get("grant_type").as_str() {
        "client_credentials" => OAuthGrant::ClientCredentials,
        "" | "authorization_code" | "authorization_code_with_pkce" => OAuthGrant::AuthorizationCode,
        other => {
            warnings.push(format!(
                "{owner}: grant OAuth {other} belum didukung, diganti tanpa auth"
            ));
            return Some(Auth::None);
        }
    };
    Some(Auth::OAuth2(OAuth2Config {
        grant,
        auth_url: get("authUrl"),
        token_url: get("accessTokenUrl"),
        client_id: get("clientId"),
        client_secret: get("clientSecret"),
        scope: get("scope"),
        use_pkce: true,
        client_auth: if get("client_authentication") == "body" {
            ClientAuth::Body
        } else {
            ClientAuth::Basic
        },
        callback_port: 0,
        token_key: uuid::Uuid::now_v7().to_string(),
    }))
}
