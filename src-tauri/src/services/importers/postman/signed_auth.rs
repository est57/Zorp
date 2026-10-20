//! Reads the Postman auth types that Zorp signs per request.
use serde_json::Value;

use super::fields::auth_param;
use crate::models::auth::{AwsSigV4Config, JwtAlgorithm, JwtConfig, OAuth1Config, OAuth1Method};
use crate::models::request::{ApiKeyLocation, Auth};

pub fn parse(kind: &str, value: &Value) -> Option<Auth> {
    let list = value.get(kind);
    let get = |key: &str| auth_param(list, key);
    Some(match kind {
        "digest" => Auth::Digest {
            username: get("username"),
            password: get("password"),
        },
        "awsv4" => Auth::AwsSigV4(AwsSigV4Config {
            access_key: get("accessKey"),
            secret_key: get("secretKey"),
            session_token: get("sessionToken"),
            region: get("region"),
            service: get("service"),
        }),
        "oauth1" => Auth::OAuth1(OAuth1Config {
            consumer_key: get("consumerKey"),
            consumer_secret: get("consumerSecret"),
            token: get("token"),
            token_secret: get("tokenSecret"),
            signature_method: match get("signatureMethod").as_str() {
                "HMAC-SHA256" => OAuth1Method::HmacSha256,
                "PLAINTEXT" => OAuth1Method::Plaintext,
                _ => OAuth1Method::HmacSha1,
            },
            realm: get("realm"),
        }),
        "jwt" => {
            let query = get("addTokenTo") == "queryParam";
            Auth::Jwt(JwtConfig {
                algorithm: serde_json::from_value(Value::String(get("algorithm")))
                    .unwrap_or(JwtAlgorithm::HS256),
                secret: get("secret"),
                secret_base64: get("isSecretBase64Encoded") == "true",
                payload: get("payload"),
                headers: get("header"),
                location: if query {
                    ApiKeyLocation::Query
                } else {
                    ApiKeyLocation::Header
                },
                param: if query {
                    get("queryParamKey")
                } else {
                    "Authorization".into()
                },
                prefix: if query {
                    String::new()
                } else {
                    get("headerPrefix")
                },
            })
        }
        _ => return None,
    })
}
