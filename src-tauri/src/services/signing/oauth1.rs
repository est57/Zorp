use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use reqwest::Request;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue};
use sha1::Sha1;
use sha2::Sha256;

use super::encode;
use crate::error::{AppError, AppResult};
use crate::models::auth::{OAuth1Config, OAuth1Method};

fn sign_with<M: Mac + KeyInit>(key: &str, text: &str) -> String {
    let mut mac =
        <M as KeyInit>::new_from_slice(key.as_bytes()).expect("HMAC accepts any key length");
    mac.update(text.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}

/// Adds an OAuth 1.0a `Authorization` header to a built request (RFC 5849).
pub fn sign(req: &mut Request, c: &OAuth1Config, nonce: &str, timestamp: i64) -> AppResult<()> {
    if c.consumer_key.trim().is_empty() {
        return Err(AppError::Validation(
            "OAuth 1.0: consumer key wajib diisi".into(),
        ));
    }
    let method_name = match c.signature_method {
        OAuth1Method::HmacSha1 => "HMAC-SHA1",
        OAuth1Method::HmacSha256 => "HMAC-SHA256",
        OAuth1Method::Plaintext => "PLAINTEXT",
    };
    let mut oauth: Vec<(String, String)> = vec![
        ("oauth_consumer_key".into(), c.consumer_key.clone()),
        ("oauth_nonce".into(), nonce.to_string()),
        ("oauth_signature_method".into(), method_name.into()),
        ("oauth_timestamp".into(), timestamp.to_string()),
        ("oauth_version".into(), "1.0".into()),
    ];
    if !c.token.is_empty() {
        oauth.push(("oauth_token".into(), c.token.clone()));
    }
    let url = req.url().clone();
    let mut params: Vec<(String, String)> = oauth.clone();
    params.extend(
        url.query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned())),
    );
    let form = req
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/x-www-form-urlencoded"));
    if form && let Some(bytes) = req.body().and_then(|b| b.as_bytes()) {
        params.extend(
            url::form_urlencoded::parse(bytes).map(|(k, v)| (k.into_owned(), v.into_owned())),
        );
    }
    let mut encoded: Vec<(String, String)> =
        params.iter().map(|(k, v)| (encode(k), encode(v))).collect();
    encoded.sort();
    let normalized = encoded
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");
    let mut base_url = url.clone();
    base_url.set_query(None);
    base_url.set_fragment(None);
    let base = format!(
        "{}&{}&{}",
        req.method().as_str(),
        encode(base_url.as_str()),
        encode(&normalized)
    );
    let key = format!("{}&{}", encode(&c.consumer_secret), encode(&c.token_secret));
    let signature = match c.signature_method {
        OAuth1Method::HmacSha1 => sign_with::<Hmac<Sha1>>(&key, &base),
        OAuth1Method::HmacSha256 => sign_with::<Hmac<Sha256>>(&key, &base),
        OAuth1Method::Plaintext => key,
    };
    oauth.push(("oauth_signature".into(), signature));
    let mut fields: Vec<String> = Vec::new();
    if !c.realm.is_empty() {
        fields.push(format!("realm=\"{}\"", encode(&c.realm)));
    }
    fields.extend(oauth.iter().map(|(k, v)| format!("{k}=\"{}\"", encode(v))));
    let value = HeaderValue::from_str(&format!("OAuth {}", fields.join(", ")))
        .map_err(|_| AppError::Validation("OAuth 1.0: nilai header tidak valid".into()))?;
    req.headers_mut().insert(AUTHORIZATION, value);
    Ok(())
}
