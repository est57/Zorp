//! Postman forms of the auth types that are signed per request.
use serde_json::{Value, json};

use crate::models::auth::{JwtConfig, OAuth1Method};
use crate::models::request::{ApiKeyLocation, Auth};

fn params(entries: &[(&str, Value)]) -> Value {
    Value::Array(
        entries
            .iter()
            .map(|(k, v)| json!({ "key": k, "value": v, "type": "string" }))
            .collect(),
    )
}

fn jwt(c: &JwtConfig) -> Value {
    let (add_to, query_key) = match c.location {
        ApiKeyLocation::Header => ("header", String::new()),
        ApiKeyLocation::Query => ("queryParam", c.param.clone()),
    };
    params(&[
        ("algorithm", json!(format!("{:?}", c.algorithm))),
        ("secret", json!(c.secret)),
        ("isSecretBase64Encoded", json!(c.secret_base64)),
        ("payload", json!(c.payload)),
        ("addTokenTo", json!(add_to)),
        ("headerPrefix", json!(c.prefix)),
        ("queryParamKey", json!(query_key)),
        ("header", json!(c.headers)),
    ])
}

/// Returns `None` for auth types handled elsewhere.
pub fn auth(auth: &Auth) -> Option<Value> {
    Some(match auth {
        Auth::Digest { username, password } => json!({ "type": "digest", "digest": params(&[
            ("username", json!(username)),
            ("password", json!(password)),
        ]) }),
        Auth::Jwt(c) => json!({ "type": "jwt", "jwt": jwt(c) }),
        Auth::AwsSigV4(c) => json!({ "type": "awsv4", "awsv4": params(&[
            ("accessKey", json!(c.access_key)),
            ("secretKey", json!(c.secret_key)),
            ("sessionToken", json!(c.session_token)),
            ("region", json!(c.region)),
            ("service", json!(c.service)),
        ]) }),
        Auth::OAuth1(c) => {
            let method = match c.signature_method {
                OAuth1Method::HmacSha1 => "HMAC-SHA1",
                OAuth1Method::HmacSha256 => "HMAC-SHA256",
                OAuth1Method::Plaintext => "PLAINTEXT",
            };
            json!({ "type": "oauth1", "oauth1": params(&[
                ("consumerKey", json!(c.consumer_key)),
                ("consumerSecret", json!(c.consumer_secret)),
                ("token", json!(c.token)),
                ("tokenSecret", json!(c.token_secret)),
                ("signatureMethod", json!(method)),
                ("realm", json!(c.realm)),
                ("addParamsToHeader", json!(true)),
            ]) })
        }
        _ => return None,
    })
}
