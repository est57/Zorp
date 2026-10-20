use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use wiremock::matchers::{header, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::models::request::{Body, HttpMethod, KeyValue, RequestSettings, RequestSpec};

type Log = Arc<Mutex<Vec<(RealtimeKind, String, Option<String>)>>>;

fn spec(url: String, mode: crate::models::realtime::RequestMode) -> RequestSpec {
    RequestSpec {
        docs: String::new(),
        id: Uuid::now_v7(),
        name: "Live".into(),
        method: HttpMethod::Get,
        url,
        params: vec![],
        headers: vec![KeyValue {
            key: "X-Client".into(),
            value: "zorp".into(),
            enabled: true,
            description: String::new(),
        }],
        body: Body::None,
        auth: crate::models::request::Auth::None,
        settings: RequestSettings::default(),
        scripts: Default::default(),
        mode,
    }
}

fn plain_network() -> websocket::Network {
    websocket::Network {
        proxy: None,
        tls: tls::client_config(&Default::default(), true, None).unwrap(),
        timeout: Duration::from_secs(5),
    }
}

fn recorder() -> (Log, Emit) {
    let log: Log = Arc::default();
    let sink = log.clone();
    (
        log,
        Box::new(move |kind, data, event, _| sink.lock().unwrap().push((kind, data, event))),
    )
}

fn kinds(log: &Log) -> Vec<String> {
    log.lock()
        .unwrap()
        .iter()
        .map(|(k, d, _)| format!("{k:?}:{d}"))
        .collect()
}

/// Echo server that also replies with the X-Client handshake header it received.
// The handshake callback signature, including its large error type, is defined by tungstenite.
#[allow(clippy::result_large_err)]
async fn echo_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let client = Arc::new(Mutex::new(String::new()));
        let seen = client.clone();
        let callback = move |req: &tokio_tungstenite::tungstenite::handshake::server::Request,
                             res| {
            *seen.lock().unwrap() = req
                .headers()
                .get("x-client")
                .map(|v| v.to_str().unwrap().to_string())
                .unwrap_or_default();
            Ok(res)
        };
        let mut ws = tokio_tungstenite::accept_hdr_async(stream, callback)
            .await
            .unwrap();
        let hello = format!("hello {}", client.lock().unwrap());
        ws.send(tokio_tungstenite::tungstenite::Message::Text(hello.into()))
            .await
            .unwrap();
        while let Some(Ok(message)) = ws.next().await {
            if message.is_text() {
                ws.send(message).await.unwrap();
            }
        }
    });
    format!("ws://{addr}")
}

#[tokio::test]
async fn websocket_sends_headers_echoes_and_closes() {
    let url = echo_server().await;
    let (log, emit) = recorder();
    let (tx, rx) = mpsc::unbounded_channel();
    let task = tokio::spawn(websocket::run(
        spec(url, crate::models::realtime::RequestMode::Websocket),
        crate::models::request::Auth::None,
        plain_network(),
        rx,
        emit,
    ));
    tokio::time::sleep(Duration::from_millis(200)).await;
    tx.send(Outgoing::Text("ping".into())).unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    tx.send(Outgoing::Close).unwrap();
    task.await.unwrap();

    let events = kinds(&log);
    assert!(
        events[0].starts_with("Open:Terhubung (HTTP 101)"),
        "{events:?}"
    );
    assert_eq!(
        &events[1..],
        [
            "Message:hello zorp",
            "Sent:ping",
            "Message:ping",
            "Close:Koneksi ditutup"
        ]
    );
}

#[tokio::test]
async fn websocket_rejects_http_urls() {
    let (log, emit) = recorder();
    let (_tx, rx) = mpsc::unbounded_channel();
    websocket::run(
        spec(
            "https://x.test".into(),
            crate::models::realtime::RequestMode::Websocket,
        ),
        crate::models::request::Auth::None,
        plain_network(),
        rx,
        emit,
    )
    .await;
    assert!(kinds(&log)[0].contains("pilih SSE"), "{:?}", kinds(&log));
}

#[tokio::test]
async fn sse_streams_named_events_until_server_ends() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(header("accept", "text/event-stream"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw("event: tick\ndata: 1\n\ndata: 2\n\n", "text/event-stream"),
        )
        .mount(&server)
        .await;
    let (log, emit) = recorder();
    let (_tx, rx) = mpsc::unbounded_channel();
    let client = crate::services::http_client::new_client(true, true, None).unwrap();
    sse_client::run(
        client,
        spec(server.uri(), crate::models::realtime::RequestMode::Sse),
        crate::models::request::Auth::None,
        rx,
        emit,
    )
    .await;

    let events = log.lock().unwrap().clone();
    assert!(matches!(events[0].0, RealtimeKind::Open));
    assert_eq!(
        (events[1].1.as_str(), events[1].2.as_deref()),
        ("1", Some("tick"))
    );
    assert_eq!((events[2].1.as_str(), events[2].2.as_deref()), ("2", None));
    assert!(matches!(events[3].0, RealtimeKind::Close));
}

#[tokio::test]
async fn sse_points_websocket_urls_to_ws_mode() {
    let (log, emit) = recorder();
    let (_tx, rx) = mpsc::unbounded_channel();
    let client = crate::services::http_client::new_client(true, true, None).unwrap();
    let request = spec(
        "wss://echo.test".into(),
        crate::models::realtime::RequestMode::Sse,
    );
    sse_client::run(
        client,
        request,
        crate::models::request::Auth::None,
        rx,
        emit,
    )
    .await;
    assert!(kinds(&log)[0].contains("pilih WS"), "{:?}", kinds(&log));
}
