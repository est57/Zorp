use std::path::Path;
use std::time::Instant;

use base64::Engine;
use reqwest::{RequestBuilder, Response};

use crate::error::{AppError, AppResult, ErrorKind};
use crate::models::response::{ProgressPhase, ResponseBody, ResponseData, Timing};
use crate::services::timing::{self, Phases};

const INLINE_LIMIT: usize = 5 * 1024 * 1024;
const PREVIEW_LIMIT: usize = 1024 * 1024;

pub fn classify(error: reqwest::Error) -> AppError {
    let detail = format!("{error:?}").to_lowercase();
    let (kind, message) = if error.is_timeout() {
        (
            ErrorKind::Timeout,
            "Server tidak merespons sebelum batas waktu habis.".to_string(),
        )
    } else if detail.contains("dns") || detail.contains("failed to lookup") {
        (
            ErrorKind::Dns,
            "Nama host tidak ditemukan. Periksa penulisan URL atau koneksi jaringan.".into(),
        )
    } else if detail.contains("certificate")
        || detail.contains("tls")
        || detail.contains("handshake")
    {
        (ErrorKind::Tls, "Sertifikat TLS tidak valid. Periksa sertifikat server atau nonaktifkan verifikasi TLS.".into())
    } else if error.is_connect() {
        (
            ErrorKind::Connect,
            "Tidak bisa terhubung ke server. Pastikan server berjalan dan port benar.".into(),
        )
    } else {
        (ErrorKind::Network, format!("Request gagal: {error}"))
    };
    AppError::Network { kind, message }
}

fn is_textual(mime: &str) -> bool {
    let mime = mime.to_ascii_lowercase();
    mime.starts_with("text/")
        || [
            "json",
            "xml",
            "javascript",
            "x-www-form-urlencoded",
            "html",
            "yaml",
            "csv",
        ]
        .iter()
        .any(|t| mime.contains(t))
}

fn to_body(bytes: Vec<u8>, mime: &str) -> ResponseBody {
    if is_textual(mime) || mime.is_empty() {
        match String::from_utf8(bytes) {
            Ok(text) => ResponseBody::Text { text },
            Err(e) => to_base64(e.into_bytes(), mime),
        }
    } else {
        to_base64(bytes, mime)
    }
}

fn to_base64(bytes: Vec<u8>, mime: &str) -> ResponseBody {
    let data = base64::engine::general_purpose::STANDARD.encode(bytes);
    ResponseBody::Base64 {
        data,
        mime: mime.to_string(),
    }
}

fn spill_to_file(dir: &Path, bytes: &[u8]) -> AppResult<String> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("{}.bin", uuid::Uuid::now_v7()));
    std::fs::write(&path, bytes)?;
    Ok(path.to_string_lossy().into_owned())
}

/// Sends the request and reads the full response, spilling large bodies to `spill_dir`.
pub async fn execute(
    builder: RequestBuilder,
    spill_dir: &Path,
    progress: impl Fn(ProgressPhase, u64),
) -> AppResult<ResponseData> {
    let started = Instant::now();
    progress(ProgressPhase::Sending, 0);
    let recorder = timing::Recorder::default();
    let mut response: Response = timing::measure(recorder.clone(), builder.send())
        .await
        .map_err(classify)?;
    let ttfb = started.elapsed();
    let phases: Phases = *recorder.lock().unwrap_or_else(|e| e.into_inner());
    progress(ProgressPhase::Downloading, 0);

    let status = response.status();
    let mime = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let headers = response
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                String::from_utf8_lossy(v.as_bytes()).into_owned(),
            )
        })
        .collect();
    let http_version = format!("{:?}", response.version());
    let url = response.url().to_string();

    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(classify)? {
        bytes.extend_from_slice(&chunk);
        progress(ProgressPhase::Downloading, bytes.len() as u64);
    }
    let total = started.elapsed();
    let size_bytes = bytes.len() as u64;

    let body = if bytes.len() > INLINE_LIMIT {
        let path = spill_to_file(spill_dir, &bytes)?;
        let preview = String::from_utf8_lossy(&bytes[..PREVIEW_LIMIT]).into_owned();
        ResponseBody::TooLarge { preview, path }
    } else {
        to_body(bytes, &mime)
    };

    Ok(ResponseData {
        status: status.as_u16(),
        status_text: status.canonical_reason().unwrap_or_default().to_string(),
        headers,
        body,
        size_bytes,
        timing: Timing {
            dns_ms: phases.dns.map(|d| d.as_millis() as u32),
            // Connector time includes the DNS lookup, so subtract it to get TCP + TLS.
            connect_ms: phases
                .connect
                .map(|c| c.saturating_sub(phases.dns.unwrap_or_default()).as_millis() as u32),
            ttfb_ms: ttfb.as_millis() as u32,
            download_ms: (total - ttfb).as_millis() as u32,
            total_ms: total.as_millis() as u32,
        },
        http_version,
        url,
    })
}
