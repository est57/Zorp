use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue};

use super::{Emit, Outgoing};
use crate::error::{AppError, AppResult};
use crate::models::realtime::RealtimeKind;
use crate::models::request::{ApiKeyLocation, Auth, RequestSpec};

/// URL and headers for the WebSocket handshake, including auth.
pub fn handshake_request(
    spec: &RequestSpec,
    auth: &Auth,
) -> AppResult<tokio_tungstenite::tungstenite::handshake::client::Request> {
    let mut url =
        url::Url::parse(spec.url.trim()).map_err(|e| AppError::InvalidUrl(e.to_string()))?;
    if !matches!(url.scheme(), "ws" | "wss") {
        let hint = if matches!(url.scheme(), "http" | "https") {
            " Untuk stream http:// atau https://, pilih SSE."
        } else {
            ""
        };
        return Err(AppError::InvalidUrl(format!(
            "URL WebSocket harus diawali ws:// atau wss://.{hint}"
        )));
    }
    if let Auth::ApiKey {
        key,
        value,
        location: ApiKeyLocation::Query,
    } = auth
    {
        url.query_pairs_mut().append_pair(key, value);
    }
    let mut request = url
        .as_str()
        .into_client_request()
        .map_err(|e| AppError::InvalidUrl(e.to_string()))?;
    let mut headers: Vec<(String, String)> = spec
        .headers
        .iter()
        .filter(|h| h.enabled && !h.key.is_empty())
        .map(|h| (h.key.clone(), h.value.clone()))
        .collect();
    match auth {
        Auth::Bearer { token } => headers.push(("Authorization".into(), format!("Bearer {token}"))),
        Auth::Basic { username, password } => {
            let encoded =
                base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
            headers.push(("Authorization".into(), format!("Basic {encoded}")));
        }
        Auth::ApiKey {
            key,
            value,
            location: ApiKeyLocation::Header,
        } => headers.push((key.clone(), value.clone())),
        _ => {}
    }
    for (key, value) in headers {
        let name = HeaderName::from_bytes(key.as_bytes())
            .map_err(|_| AppError::Validation(format!("Nama header tidak valid: {key}")))?;
        let value = HeaderValue::from_str(&value)
            .map_err(|_| AppError::Validation(format!("Nilai header {key} tidak valid")))?;
        request.headers_mut().insert(name, value);
    }
    Ok(request)
}

/// tokio-tungstenite asks rustls for the process default; use the same provider as reqwest.
fn ensure_crypto_provider() {
    static INSTALLED: std::sync::Once = std::sync::Once::new();
    INSTALLED.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

/// Runs a WebSocket connection until either side closes it.
pub async fn run(
    spec: RequestSpec,
    auth: Auth,
    mut outgoing: UnboundedReceiver<Outgoing>,
    emit: Emit,
) {
    let request = match handshake_request(&spec, &auth) {
        Ok(request) => request,
        Err(e) => return emit(RealtimeKind::Error, e.to_string(), None, false),
    };
    ensure_crypto_provider();
    let (stream, response) = match connect_async(request).await {
        Ok(ok) => ok,
        Err(e) => {
            return emit(
                RealtimeKind::Error,
                format!("Gagal terhubung: {e}"),
                None,
                false,
            );
        }
    };
    emit(
        RealtimeKind::Open,
        format!("Terhubung (HTTP {})", response.status().as_u16()),
        None,
        false,
    );
    let (mut write, mut read) = stream.split();
    loop {
        tokio::select! {
            incoming = read.next() => match incoming {
                Some(Ok(Message::Text(text))) => emit(RealtimeKind::Message, text.to_string(), None, false),
                Some(Ok(Message::Binary(bytes))) => emit(RealtimeKind::Message, format!("[data biner {} byte]", bytes.len()), None, true),
                Some(Ok(Message::Close(frame))) => {
                    let reason = frame.map(|f| format!("kode {}: {}", u16::from(f.code), f.reason)).unwrap_or_default();
                    return emit(RealtimeKind::Close, format!("Ditutup oleh server {reason}").trim().to_string(), None, false);
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => return emit(RealtimeKind::Error, format!("Koneksi terputus: {e}"), None, false),
                None => return emit(RealtimeKind::Close, "Koneksi ditutup".into(), None, false),
            },
            command = outgoing.recv() => match command {
                Some(Outgoing::Text(text)) => match write.send(Message::Text(text.clone().into())).await {
                    Ok(()) => emit(RealtimeKind::Sent, text, None, false),
                    Err(e) => return emit(RealtimeKind::Error, format!("Pesan gagal dikirim: {e}"), None, false),
                },
                Some(Outgoing::Close) | None => {
                    let _ = write.send(Message::Close(None)).await;
                    return emit(RealtimeKind::Close, "Koneksi ditutup".into(), None, false);
                }
            },
        }
    }
}
