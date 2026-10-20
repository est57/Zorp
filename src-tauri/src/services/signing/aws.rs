use hmac::{Hmac, KeyInit, Mac};
use reqwest::Request;
use reqwest::header::{AUTHORIZATION, HeaderName, HeaderValue};
use sha2::{Digest, Sha256};

use super::{encode, utc_parts};
use crate::error::{AppError, AppResult};
use crate::models::auth::AwsSigV4Config;

type HmacSha256 = Hmac<Sha256>;

fn hmac(key: &[u8], data: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

fn header(name: &str, value: &str) -> AppResult<(HeaderName, HeaderValue)> {
    let value = HeaderValue::from_str(value)
        .map_err(|_| AppError::Validation(format!("Nilai {name} tidak valid")))?;
    Ok((
        HeaderName::from_bytes(name.as_bytes()).expect("static header name"),
        value,
    ))
}

/// Adds AWS Signature Version 4 headers to a built request.
pub fn sign(req: &mut Request, c: &AwsSigV4Config, now: i64) -> AppResult<()> {
    if c.access_key.trim().is_empty()
        || c.secret_key.trim().is_empty()
        || c.region.trim().is_empty()
        || c.service.trim().is_empty()
    {
        return Err(AppError::Validation(
            "AWS Signature: access key, secret key, region, dan service wajib diisi".into(),
        ));
    }
    let (y, mo, d, h, mi, s) = utc_parts(now);
    let date = format!("{y:04}{mo:02}{d:02}");
    let amz_date = format!("{date}T{h:02}{mi:02}{s:02}Z");
    let payload_hash = match req.body() {
        None => hex::encode(Sha256::digest(b"")),
        Some(body) => match body.as_bytes() {
            Some(bytes) => hex::encode(Sha256::digest(bytes)),
            None => "UNSIGNED-PAYLOAD".into(),
        },
    };
    let url = req.url().clone();
    let host = match url.port() {
        Some(port) => format!("{}:{port}", url.host_str().unwrap_or_default()),
        None => url.host_str().unwrap_or_default().to_string(),
    };
    let mut signed: Vec<(String, String)> = vec![
        ("host".into(), host),
        ("x-amz-date".into(), amz_date.clone()),
    ];
    // S3 requires the payload hash header; other services accept the request without it.
    if c.service == "s3" {
        signed.push(("x-amz-content-sha256".into(), payload_hash.clone()));
    }
    if !c.session_token.is_empty() {
        signed.push(("x-amz-security-token".into(), c.session_token.clone()));
    }
    signed.sort();
    let mut query: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| (encode(&k), encode(&v)))
        .collect();
    query.sort();
    let canonical = [
        req.method().as_str().to_string(),
        if url.path().is_empty() {
            "/".into()
        } else {
            url.path().to_string()
        },
        query
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&"),
        signed
            .iter()
            .map(|(k, v)| format!("{k}:{}\n", v.trim()))
            .collect::<String>(),
        signed
            .iter()
            .map(|(k, _)| k.as_str())
            .collect::<Vec<_>>()
            .join(";"),
        payload_hash,
    ]
    .join("\n");
    let scope = format!(
        "{date}/{}/{}/aws4_request",
        c.region.trim(),
        c.service.trim()
    );
    let to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
        hex::encode(Sha256::digest(canonical.as_bytes()))
    );
    let mut key = hmac(format!("AWS4{}", c.secret_key).as_bytes(), &date);
    for part in [c.region.trim(), c.service.trim(), "aws4_request"] {
        key = hmac(&key, part);
    }
    let signature = hex::encode(hmac(&key, &to_sign));
    let names: Vec<&str> = signed.iter().map(|(k, _)| k.as_str()).collect();
    let headers = req.headers_mut();
    for (name, value) in signed.iter().filter(|(k, _)| k != "host") {
        let (n, v) = header(name, value)?;
        headers.insert(n, v);
    }
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={}, Signature={signature}",
        c.access_key.trim(),
        names.join(";")
    );
    headers.insert(AUTHORIZATION, header("Authorization", &authorization)?.1);
    Ok(())
}
