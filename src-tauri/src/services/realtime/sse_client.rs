use reqwest::header::ACCEPT;
use tokio::sync::mpsc::UnboundedReceiver;

use super::sse::SseParser;
use super::{Emit, Outgoing};
use crate::models::realtime::RealtimeKind;
use crate::models::request::{Auth, RequestSpec};
use crate::services::request_builder;

const ERROR_BODY_LIMIT: usize = 500;

/// Streams Server-Sent Events until the server ends the stream or the user disconnects.
pub async fn run(
    client: reqwest::Client,
    mut spec: RequestSpec,
    auth: Auth,
    mut outgoing: UnboundedReceiver<Outgoing>,
    emit: Emit,
) {
    let scheme = spec
        .url
        .trim()
        .split("://")
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if scheme == "ws" || scheme == "wss" {
        let message =
            "SSE memakai URL http:// atau https://. Untuk alamat ws:// atau wss://, pilih WS.";
        return emit(RealtimeKind::Error, message.into(), None, false);
    }
    // A stream stays open indefinitely, so the request timeout must not apply.
    spec.settings.timeout_ms = u32::MAX;
    let builder = match request_builder::build(&client, &spec, &auth) {
        Ok(builder) => builder.header(ACCEPT, "text/event-stream"),
        Err(e) => return emit(RealtimeKind::Error, e.to_string(), None, false),
    };
    let mut response = tokio::select! {
        response = builder.send() => match response {
            Ok(response) => response,
            Err(e) => return emit(RealtimeKind::Error, format!("Gagal terhubung: {e}"), None, false),
        },
        _ = outgoing.recv() => return emit(RealtimeKind::Close, "Koneksi ditutup".into(), None, false),
    };
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let snippet: String = body.chars().take(ERROR_BODY_LIMIT).collect();
        return emit(
            RealtimeKind::Error,
            format!("Server membalas HTTP {}: {snippet}", status.as_u16()),
            None,
            false,
        );
    }
    emit(
        RealtimeKind::Open,
        format!("Terhubung (HTTP {})", status.as_u16()),
        None,
        false,
    );
    let mut parser = SseParser::default();
    loop {
        tokio::select! {
            chunk = response.chunk() => match chunk {
                Ok(Some(bytes)) => {
                    for message in parser.push(&String::from_utf8_lossy(&bytes)) {
                        emit(RealtimeKind::Message, message.data, message.event, false);
                    }
                }
                Ok(None) => return emit(RealtimeKind::Close, "Stream diakhiri oleh server".into(), None, false),
                Err(e) => return emit(RealtimeKind::Error, format!("Stream terputus: {e}"), None, false),
            },
            command = outgoing.recv() => match command {
                // SSE is one-way; text from the UI is ignored and only closing matters.
                Some(Outgoing::Text(_)) => {}
                Some(Outgoing::Close) | None => return emit(RealtimeKind::Close, "Koneksi ditutup".into(), None, false),
            },
        }
    }
}
