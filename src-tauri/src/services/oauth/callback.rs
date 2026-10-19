use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use url::Url;

use crate::error::{AppError, AppResult};

const MAX_REQUEST_BYTES: usize = 16 * 1024;
const SUCCESS_PAGE: &str = "<!doctype html><meta charset=utf-8><title>Zorp</title><body style=\"font-family:system-ui;padding:40px\"><h2>Login berhasil</h2><p>Anda bisa menutup jendela ini dan kembali ke Zorp.</p></body>";
const FAILURE_PAGE: &str = "<!doctype html><meta charset=utf-8><title>Zorp</title><body style=\"font-family:system-ui;padding:40px\"><h2>Login gagal</h2><p>Kembali ke Zorp untuk melihat detail dan mencoba lagi.</p></body>";

pub struct Callback {
    listener: TcpListener,
    pub redirect_uri: String,
}

/// Binds the loopback listener used as the OAuth redirect URI (RFC 8252).
pub async fn bind(port: u16) -> AppResult<Callback> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| AppError::OAuth(format!("Port callback {port} tidak bisa dipakai: {e}")))?;
    let port = listener.local_addr()?.port();
    Ok(Callback {
        listener,
        redirect_uri: format!("http://127.0.0.1:{port}/callback"),
    })
}

async fn read_request_target(stream: &mut TcpStream) -> AppResult<String> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 1024];
    while !buf.windows(4).any(|w| w == b"\r\n\r\n") && buf.len() < MAX_REQUEST_BYTES {
        let n = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut chunk))
            .await
            .map_err(|_| AppError::OAuth("Callback tidak lengkap".into()))??;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    let head = String::from_utf8_lossy(&buf);
    let line = head.lines().next().unwrap_or_default();
    Ok(line
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_string())
}

async fn respond(stream: &mut TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

impl Callback {
    /// Waits for the provider redirect and returns the authorization code.
    pub async fn wait_for_code(&self, expected_state: &str) -> AppResult<String> {
        loop {
            let (mut stream, _) = self.listener.accept().await?;
            let target = read_request_target(&mut stream).await.unwrap_or_default();
            let Ok(url) = Url::parse(&format!("http://127.0.0.1{target}")) else {
                continue;
            };
            if url.path() != "/callback" {
                respond(&mut stream, "404 Not Found", "").await;
                continue;
            }
            let param = |name: &str| {
                url.query_pairs()
                    .find(|(k, _)| k == name)
                    .map(|(_, v)| v.into_owned())
            };
            let result = if let Some(error) = param("error") {
                let detail = param("error_description")
                    .map(|d| format!(": {d}"))
                    .unwrap_or_default();
                Err(AppError::OAuth(format!("Login ditolak ({error}){detail}")))
            } else if param("state").as_deref() != Some(expected_state) {
                Err(AppError::OAuth(
                    "Parameter state tidak cocok. Login dibatalkan demi keamanan.".into(),
                ))
            } else {
                param("code")
                    .ok_or_else(|| AppError::OAuth("Callback tidak berisi kode otorisasi".into()))
            };
            let (status, page) = if result.is_ok() {
                ("200 OK", SUCCESS_PAGE)
            } else {
                ("400 Bad Request", FAILURE_PAGE)
            };
            respond(&mut stream, status, page).await;
            return result;
        }
    }
}
