//! Settings for auth types that sign or compute their credentials per request.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::request::ApiKeyLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum JwtAlgorithm {
    HS256,
    HS384,
    HS512,
    RS256,
    RS384,
    RS512,
    ES256,
    ES384,
}

/// Signs a JWT on every send and attaches it to the request.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct JwtConfig {
    pub algorithm: JwtAlgorithm,
    /// HMAC secret, or a PEM private key for RSA and ECDSA.
    pub secret: String,
    #[serde(default)]
    pub secret_base64: bool,
    /// Claims as a JSON object; `iat` is added when missing.
    pub payload: String,
    /// Extra header fields as a JSON object, may be empty.
    #[serde(default)]
    pub headers: String,
    pub location: ApiKeyLocation,
    /// Header name or query parameter that carries the token.
    pub param: String,
    /// Text before the token in a header, such as `Bearer`.
    #[serde(default)]
    pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AwsSigV4Config {
    pub access_key: String,
    pub secret_key: String,
    #[serde(default)]
    pub session_token: String,
    pub region: String,
    pub service: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
#[ts(export)]
pub enum OAuth1Method {
    HmacSha1,
    HmacSha256,
    Plaintext,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OAuth1Config {
    pub consumer_key: String,
    pub consumer_secret: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub token_secret: String,
    pub signature_method: OAuth1Method,
    #[serde(default)]
    pub realm: String,
}
