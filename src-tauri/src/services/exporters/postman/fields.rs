use serde_json::{Value, json};

use crate::models::oauth::{ClientAuth, OAuthGrant};
use crate::models::request::{ApiKeyLocation, Auth, KeyValue};

pub fn pairs(items: &[KeyValue]) -> Value {
    Value::Array(
        items
            .iter()
            .filter(|kv| !kv.key.is_empty() || !kv.value.is_empty())
            .map(|kv| {
                let mut item = json!({ "key": kv.key, "value": kv.value });
                if !kv.enabled {
                    item["disabled"] = json!(true);
                }
                if !kv.description.is_empty() {
                    item["description"] = json!(kv.description);
                }
                item
            })
            .collect(),
    )
}

fn params(entries: &[(&str, &str)]) -> Value {
    Value::Array(
        entries
            .iter()
            .map(|(k, v)| json!({ "key": k, "value": v, "type": "string" }))
            .collect(),
    )
}

/// Postman auth object; `None` means the item inherits from its parent.
pub fn auth(auth: &Auth) -> Option<Value> {
    Some(match auth {
        Auth::Inherit => return None,
        Auth::None => json!({ "type": "noauth" }),
        Auth::Bearer { token } => {
            json!({ "type": "bearer", "bearer": params(&[("token", token)]) })
        }
        Auth::Basic { username, password } => {
            json!({ "type": "basic", "basic": params(&[("username", username), ("password", password)]) })
        }
        Auth::ApiKey {
            key,
            value,
            location,
        } => {
            let place = if matches!(location, ApiKeyLocation::Query) {
                "query"
            } else {
                "header"
            };
            json!({ "type": "apikey", "apikey": params(&[("key", key), ("value", value), ("in", place)]) })
        }
        Auth::OAuth2(config) => {
            let grant = match config.grant {
                OAuthGrant::AuthorizationCode if config.use_pkce => "authorization_code_with_pkce",
                OAuthGrant::AuthorizationCode => "authorization_code",
                OAuthGrant::ClientCredentials => "client_credentials",
            };
            let client_auth = if config.client_auth == ClientAuth::Body {
                "body"
            } else {
                "header"
            };
            json!({ "type": "oauth2", "oauth2": params(&[
                ("grant_type", grant),
                ("authUrl", &config.auth_url),
                ("accessTokenUrl", &config.token_url),
                ("clientId", &config.client_id),
                ("clientSecret", &config.client_secret),
                ("scope", &config.scope),
                ("client_authentication", client_auth),
            ]) })
        }
    })
}
