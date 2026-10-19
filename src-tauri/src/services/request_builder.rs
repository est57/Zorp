use std::path::Path;
use std::time::Duration;

use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use reqwest::multipart::{Form, Part};
use reqwest::{Client, RequestBuilder};
use url::Url;

use crate::error::{AppError, AppResult};
use crate::models::request::{ApiKeyLocation, Auth, Body, MultipartPart, RequestSpec};

const MAX_UPLOAD_BYTES: u64 = 100 * 1024 * 1024;

pub fn parse_url(raw: &str) -> AppResult<Url> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(AppError::InvalidUrl("URL masih kosong".into()));
    }
    let with_scheme = if raw.contains("://") {
        raw.to_string()
    } else {
        format!("http://{raw}")
    };
    let url = Url::parse(&with_scheme).map_err(|e| AppError::InvalidUrl(e.to_string()))?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        other => Err(AppError::InvalidUrl(format!(
            "protokol {other} tidak didukung"
        ))),
    }
}

fn header_pair(key: &str, value: &str) -> AppResult<(HeaderName, HeaderValue)> {
    let name = HeaderName::from_bytes(key.trim().as_bytes())
        .map_err(|_| AppError::Validation(format!("Nama header tidak valid: {key}")))?;
    let value = HeaderValue::from_str(value)
        .map_err(|_| AppError::Validation(format!("Nilai header {key} tidak valid")))?;
    Ok((name, value))
}

fn read_file(path: &str) -> AppResult<(Vec<u8>, String)> {
    let path = Path::new(path);
    let size = std::fs::metadata(path)?.len();
    if size > MAX_UPLOAD_BYTES {
        return Err(AppError::Validation("Ukuran file maksimal 100 MB".into()));
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok((std::fs::read(path)?, name))
}

fn apply_body(
    builder: RequestBuilder,
    body: &Body,
    has_content_type: bool,
) -> AppResult<RequestBuilder> {
    let with_type = |b: RequestBuilder, ct: &str| {
        if has_content_type {
            b
        } else {
            b.header(CONTENT_TYPE, ct)
        }
    };
    Ok(match body {
        Body::None => builder,
        Body::Json { text } => with_type(builder, "application/json").body(text.clone()),
        Body::Raw { text, content_type } => with_type(builder, content_type).body(text.clone()),
        Body::FormUrlEncoded { fields } => {
            let pairs: Vec<(&str, &str)> = fields
                .iter()
                .map(|f| (f.key.as_str(), f.value.as_str()))
                .collect();
            builder.form(&pairs)
        }
        Body::Multipart { parts } => {
            let mut form = Form::new();
            for part in parts {
                form = match part {
                    MultipartPart::Text { key, value, .. } => form.text(key.clone(), value.clone()),
                    MultipartPart::File { key, path, .. } => {
                        let (bytes, name) = read_file(path)?;
                        form.part(key.clone(), Part::bytes(bytes).file_name(name))
                    }
                };
            }
            builder.multipart(form)
        }
        Body::Binary { path } => {
            let (bytes, _) = read_file(path)?;
            with_type(builder, "application/octet-stream").body(bytes)
        }
    })
}

/// Builds a ready-to-send request from an already resolved spec.
pub fn build(client: &Client, spec: &RequestSpec, auth: &Auth) -> AppResult<RequestBuilder> {
    let mut url = parse_url(&spec.url)?;
    {
        let mut query = url.query_pairs_mut();
        for p in &spec.params {
            query.append_pair(&p.key, &p.value);
        }
        if let Auth::ApiKey {
            key,
            value,
            location: ApiKeyLocation::Query,
        } = auth
        {
            query.append_pair(key, value);
        }
    }
    if url.query() == Some("") {
        url.set_query(None);
    }

    let mut headers = HeaderMap::new();
    for h in &spec.headers {
        let (name, value) = header_pair(&h.key, &h.value)?;
        headers.append(name, value);
    }
    let has_content_type = headers.contains_key(CONTENT_TYPE);

    let mut builder = client
        .request(spec.method.as_reqwest(), url)
        .timeout(Duration::from_millis(spec.settings.timeout_ms.max(1) as u64));
    builder = match auth {
        Auth::Bearer { token } => builder.bearer_auth(token),
        Auth::Basic { username, password } => builder.basic_auth(username, Some(password)),
        Auth::ApiKey {
            key,
            value,
            location: ApiKeyLocation::Header,
        } => {
            let (name, value) = header_pair(key, value)?;
            builder.header(name, value)
        }
        _ => builder,
    };
    apply_body(builder.headers(headers), &spec.body, has_content_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_adds_scheme_and_rejects_others() {
        assert_eq!(
            parse_url("localhost:3000/a").unwrap().as_str(),
            "http://localhost:3000/a"
        );
        assert!(parse_url("file:///etc/passwd").is_err());
        assert!(parse_url("  ").is_err());
    }
}
