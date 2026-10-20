use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use uuid::Uuid;

use super::routes::{self, Route};
use crate::db::now_ms;
use crate::error::{AppError, AppResult};
use crate::models::mock::MockLogEntry;

/// Reads the current routes on every request, so edits in the app apply right away.
pub type RouteSource = Arc<dyn Fn() -> AppResult<Vec<Route>> + Send + Sync>;
pub type LogSink = Arc<dyn Fn(MockLogEntry) + Send + Sync>;

/// Headers that described the original transfer, not the stored example body.
const DROPPED: [&str; 4] = [
    "content-length",
    "transfer-encoding",
    "content-encoding",
    "connection",
];

pub struct Running {
    pub port: u16,
    stop: oneshot::Sender<()>,
}

impl Running {
    pub fn stop(self) {
        let _ = self.stop.send(());
    }
}

fn reply(status: u16, headers: &[(String, String)], body: String) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::from(body)));
    *response.status_mut() = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
    let map = response.headers_mut();
    for (k, v) in headers
        .iter()
        .filter(|(k, _)| !DROPPED.contains(&k.to_ascii_lowercase().as_str()))
    {
        if let (Ok(name), Ok(value)) = (
            hyper::header::HeaderName::from_bytes(k.as_bytes()),
            hyper::header::HeaderValue::from_str(v),
        ) {
            map.append(name, value);
        }
    }
    // Lets web apps under development call the mock from a browser.
    map.insert(
        "access-control-allow-origin",
        hyper::header::HeaderValue::from_static("*"),
    );
    map.insert(
        "access-control-allow-headers",
        hyper::header::HeaderValue::from_static("*"),
    );
    map.insert(
        "access-control-allow-methods",
        hyper::header::HeaderValue::from_static("*"),
    );
    response
}

fn json_error(status: u16, message: String) -> Response<Full<Bytes>> {
    let headers = [("content-type".to_string(), "application/json".to_string())];
    reply(
        status,
        &headers,
        serde_json::json!({ "error": message }).to_string(),
    )
}

fn handle(
    collection_id: Uuid,
    req: &Request<Incoming>,
    source: &RouteSource,
    log: &LogSink,
) -> Response<Full<Bytes>> {
    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();
    if method == "OPTIONS" {
        return reply(204, &[], String::new());
    }
    let wanted = req
        .headers()
        .get("x-zorp-example")
        .and_then(|v| v.to_str().ok());
    let (response, example) = match source() {
        Err(e) => (json_error(500, e.to_string()), None),
        Ok(all) => match routes::find(&all, &method, &path) {
            None => (
                json_error(
                    404,
                    format!("Tidak ada request {method} {path} di koleksi ini"),
                ),
                None,
            ),
            Some(route) => match routes::pick(route, wanted) {
                Some(e) => (
                    reply(e.status, &e.headers, e.body.clone()),
                    Some(e.name.clone()),
                ),
                None => (
                    json_error(
                        501,
                        format!(
                            "Belum ada contoh respons untuk {method} {path}. Kirim request \"{}\" di Zorp lalu klik Simpan sebagai contoh.",
                            route.request_name
                        ),
                    ),
                    None,
                ),
            },
        },
    };
    log(MockLogEntry {
        collection_id,
        at: now_ms(),
        method,
        path,
        status: response.status().as_u16(),
        example,
    });
    response
}

/// Starts serving on 127.0.0.1. Port 0 picks a free port.
pub async fn start(
    collection_id: Uuid,
    port: u16,
    source: RouteSource,
    log: LogSink,
) -> AppResult<Running> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| AppError::Validation(format!("Port {port} tidak bisa dipakai: {e}")))?;
    let port = listener.local_addr()?.port();
    let (stop, mut stopped) = oneshot::channel();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut stopped => break,
                accepted = listener.accept() => {
                    let Ok((stream, _)) = accepted else { continue };
                    let (source, log) = (source.clone(), log.clone());
                    tokio::spawn(async move {
                        let service = service_fn(move |req: Request<Incoming>| {
                            let response = handle(collection_id, &req, &source, &log);
                            async move { Ok::<_, Infallible>(response) }
                        });
                        let _ = http1::Builder::new().serve_connection(TokioIo::new(stream), service).await;
                    });
                }
            }
        }
    });
    Ok(Running { port, stop })
}
