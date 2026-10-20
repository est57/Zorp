use base64::Engine;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde_json::{Map, Value, json};

use crate::error::{AppError, AppResult};
use crate::models::auth::{JwtAlgorithm, JwtConfig};
use crate::models::request::{ApiKeyLocation, Auth};

fn invalid(what: &str, e: impl std::fmt::Display) -> AppError {
    AppError::Validation(format!("JWT: {what} tidak valid ({e})"))
}

fn object(text: &str, what: &str) -> AppResult<Map<String, Value>> {
    if text.trim().is_empty() {
        return Ok(Map::new());
    }
    match serde_json::from_str(text).map_err(|e| invalid(what, e))? {
        Value::Object(map) => Ok(map),
        _ => Err(AppError::Validation(format!(
            "JWT: {what} harus berupa objek JSON"
        ))),
    }
}

fn algorithm(a: JwtAlgorithm) -> Algorithm {
    match a {
        JwtAlgorithm::HS256 => Algorithm::HS256,
        JwtAlgorithm::HS384 => Algorithm::HS384,
        JwtAlgorithm::HS512 => Algorithm::HS512,
        JwtAlgorithm::RS256 => Algorithm::RS256,
        JwtAlgorithm::RS384 => Algorithm::RS384,
        JwtAlgorithm::RS512 => Algorithm::RS512,
        JwtAlgorithm::ES256 => Algorithm::ES256,
        JwtAlgorithm::ES384 => Algorithm::ES384,
    }
}

fn key(c: &JwtConfig) -> AppResult<EncodingKey> {
    let secret = c.secret.trim();
    match c.algorithm {
        JwtAlgorithm::HS256 | JwtAlgorithm::HS384 | JwtAlgorithm::HS512 => {
            if c.secret_base64 {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(secret)
                    .map_err(|e| invalid("secret base64", e))?;
                Ok(EncodingKey::from_secret(&bytes))
            } else {
                Ok(EncodingKey::from_secret(c.secret.as_bytes()))
            }
        }
        JwtAlgorithm::RS256 | JwtAlgorithm::RS384 | JwtAlgorithm::RS512 => {
            EncodingKey::from_rsa_pem(secret.as_bytes())
                .map_err(|e| invalid("private key RSA (PEM)", e))
        }
        JwtAlgorithm::ES256 | JwtAlgorithm::ES384 => EncodingKey::from_ec_pem(secret.as_bytes())
            .map_err(|e| invalid("private key EC (PEM)", e)),
    }
}

/// Signs the configured claims. `iat` is set to `now` when the payload has none.
pub fn token(c: &JwtConfig, now: i64) -> AppResult<String> {
    let mut claims = object(&c.payload, "payload")?;
    claims.entry("iat").or_insert(json!(now));
    let mut header = object(&c.headers, "header")?;
    header.insert("alg".into(), json!(format!("{:?}", c.algorithm)));
    header.entry("typ").or_insert(json!("JWT"));
    let header: Header =
        serde_json::from_value(Value::Object(header)).map_err(|e| invalid("header", e))?;
    debug_assert_eq!(header.alg, algorithm(c.algorithm));
    jsonwebtoken::encode(&header, &Value::Object(claims), &key(c)?)
        .map_err(|e| invalid("penandatanganan", e))
}

/// The signed token as a header or query parameter.
pub fn to_auth(c: &JwtConfig, now: i64) -> AppResult<Auth> {
    let token = token(c, now)?;
    let (key, value) = match c.location {
        ApiKeyLocation::Header => {
            let name = if c.param.trim().is_empty() {
                "Authorization"
            } else {
                c.param.trim()
            };
            (
                name.to_string(),
                format!("{} {token}", c.prefix.trim()).trim().to_string(),
            )
        }
        ApiKeyLocation::Query => (c.param.trim().to_string(), token),
    };
    if key.is_empty() {
        return Err(AppError::Validation(
            "JWT: isi nama query param untuk token".into(),
        ));
    }
    Ok(Auth::ApiKey {
        key,
        value,
        location: c.location,
    })
}
