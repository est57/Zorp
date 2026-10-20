use serde_json::Value;
use uuid::Uuid;

use super::schema::Doc;
use crate::models::oauth::{ClientAuth, OAuth2Config, OAuthGrant};
use crate::models::request::{ApiKeyLocation, Auth};

/// Converts a security requirement list into auth. Credentials are always variables, so
/// nothing secret ends up in the collection; the variables belong in an environment.
pub fn requirement(doc: &Doc, requirements: &Value, warnings: &mut Vec<String>) -> Option<Auth> {
    let first = requirements.as_array()?.first()?.as_object()?;
    let (name, scopes) = first.iter().next()?;
    let schemes = doc
        .root
        .pointer("/components/securitySchemes")
        .or_else(|| doc.root.get("securityDefinitions"))?;
    let scheme = doc.resolve(schemes.get(name)?);
    let scopes = scopes
        .as_array()
        .map(|s| {
            s.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    scheme_auth(scheme, name, scopes, warnings)
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn scheme_auth(
    scheme: &Value,
    name: &str,
    scope: String,
    warnings: &mut Vec<String>,
) -> Option<Auth> {
    match text(scheme, "type") {
        "http" if text(scheme, "scheme").eq_ignore_ascii_case("basic") => Some(Auth::Basic {
            username: "{{username}}".into(),
            password: "{{password}}".into(),
        }),
        "http" if text(scheme, "scheme").eq_ignore_ascii_case("bearer") => bearer(),
        "basic" => Some(Auth::Basic {
            username: "{{username}}".into(),
            password: "{{password}}".into(),
        }),
        "apiKey" => {
            let location = match text(scheme, "in") {
                "header" => ApiKeyLocation::Header,
                "query" => ApiKeyLocation::Query,
                other => {
                    warnings.push(format!(
                        "Skema auth {name}: API key di {other} belum didukung"
                    ));
                    return None;
                }
            };
            Some(Auth::ApiKey {
                key: text(scheme, "name").to_string(),
                value: "{{apiKey}}".into(),
                location,
            })
        }
        "oauth2" => oauth2(scheme, name, scope, warnings),
        other => {
            warnings.push(format!(
                "Skema auth {name} ({other}) belum didukung, gunakan Bearer"
            ));
            bearer()
        }
    }
}

fn bearer() -> Option<Auth> {
    Some(Auth::Bearer {
        token: "{{token}}".into(),
    })
}

fn oauth2(scheme: &Value, name: &str, scope: String, warnings: &mut Vec<String>) -> Option<Auth> {
    // OpenAPI 3 lists flows by name; Swagger 2 has one flow with the URLs beside it.
    let (grant, flow) = match scheme.get("flows") {
        Some(flows) => match (
            flows.get("authorizationCode"),
            flows.get("clientCredentials"),
        ) {
            (Some(f), _) => (OAuthGrant::AuthorizationCode, f),
            (None, Some(f)) => (OAuthGrant::ClientCredentials, f),
            _ => return unsupported(name, warnings),
        },
        None => match text(scheme, "flow") {
            "accessCode" => (OAuthGrant::AuthorizationCode, scheme),
            "application" => (OAuthGrant::ClientCredentials, scheme),
            _ => return unsupported(name, warnings),
        },
    };
    Some(Auth::OAuth2(OAuth2Config {
        grant,
        auth_url: text(flow, "authorizationUrl").to_string(),
        token_url: text(flow, "tokenUrl").to_string(),
        client_id: "{{clientId}}".into(),
        client_secret: "{{clientSecret}}".into(),
        scope,
        use_pkce: grant == OAuthGrant::AuthorizationCode,
        client_auth: ClientAuth::Basic,
        callback_port: 0,
        token_key: Uuid::now_v7().to_string(),
    }))
}

fn unsupported(name: &str, warnings: &mut Vec<String>) -> Option<Auth> {
    warnings.push(format!(
        "Skema auth {name}: alur OAuth 2.0 ini belum didukung, diganti Bearer {{{{token}}}}"
    ));
    bearer()
}
