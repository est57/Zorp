use base64::Engine;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use url::Url;

use crate::error::{AppError, AppResult};
use crate::models::settings::ProxySettings;

const MAX_PROXY_REPLY: usize = 8 * 1024;

#[derive(Debug, Clone)]
pub struct Proxy {
    pub url: Url,
    pub username: String,
    pub password: String,
}

fn proxy_error(message: impl Into<String>) -> AppError {
    AppError::Network {
        kind: crate::error::ErrorKind::Connect,
        message: message.into(),
    }
}

/// True when `host` matches the comma-separated bypass list (`*`, exact hosts, `.domain` or `*.domain`).
pub fn bypassed(bypass: &str, host: &str) -> bool {
    let host = host.trim_matches(['[', ']']).to_ascii_lowercase();
    bypass
        .split(',')
        .map(|e| e.trim().to_ascii_lowercase())
        .filter(|e| !e.is_empty())
        .any(|entry| {
            let suffix = entry.trim_start_matches('*');
            entry == "*"
                || entry == host
                || (suffix.starts_with('.') && (host.ends_with(suffix) || host == suffix[1..]))
        })
}

/// The proxy to use for `host`, if proxying is on and the host is not bypassed.
pub fn select(
    settings: &ProxySettings,
    password: Option<String>,
    host: &str,
) -> AppResult<Option<Proxy>> {
    if !settings.enabled || settings.url.trim().is_empty() || bypassed(&settings.bypass, host) {
        return Ok(None);
    }
    let url = Url::parse(settings.url.trim())
        .map_err(|_| AppError::Validation("URL proxy tidak valid".into()))?;
    Ok(Some(Proxy {
        url,
        username: settings.username.clone(),
        password: password.unwrap_or_default(),
    }))
}

async fn http_connect(
    stream: &mut TcpStream,
    proxy: &Proxy,
    host: &str,
    port: u16,
) -> AppResult<()> {
    let mut request = format!("CONNECT {host}:{port} HTTP/1.1\r\nHost: {host}:{port}\r\n");
    if !proxy.username.is_empty() {
        let token = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", proxy.username, proxy.password));
        request.push_str(&format!("Proxy-Authorization: Basic {token}\r\n"));
    }
    request.push_str("\r\n");
    stream.write_all(request.as_bytes()).await?;
    let mut reply = Vec::new();
    let mut byte = [0u8; 1];
    while !reply.ends_with(b"\r\n\r\n") {
        if reply.len() > MAX_PROXY_REPLY || stream.read(&mut byte).await? == 0 {
            return Err(proxy_error(
                "Proxy menutup koneksi sebelum tunnel terbentuk",
            ));
        }
        reply.push(byte[0]);
    }
    let status = String::from_utf8_lossy(&reply)
        .lines()
        .next()
        .unwrap_or_default()
        .to_string();
    if status.split_whitespace().nth(1) != Some("200") {
        return Err(proxy_error(format!(
            "Proxy menolak tunnel WebSocket: {status}"
        )));
    }
    Ok(())
}

/// Address part of a SOCKS5 request. `socks5h` lets the proxy resolve names; `socks5` resolves locally.
async fn socks_target(scheme: &str, host: &str, port: u16) -> AppResult<Vec<u8>> {
    let ip = match host.parse::<std::net::IpAddr>() {
        Ok(ip) => Some(ip),
        Err(_) if scheme == "socks5" => {
            let addr = tokio::net::lookup_host((host, port)).await?.next();
            Some(
                addr.ok_or_else(|| proxy_error(format!("Host {host} tidak ditemukan")))?
                    .ip(),
            )
        }
        Err(_) => None,
    };
    let mut target = match ip {
        Some(std::net::IpAddr::V4(v4)) => [&[1u8][..], &v4.octets()].concat(),
        Some(std::net::IpAddr::V6(v6)) => [&[4u8][..], &v6.octets()].concat(),
        None => [&[3u8, host.len() as u8][..], host.as_bytes()].concat(),
    };
    target.extend_from_slice(&port.to_be_bytes());
    Ok(target)
}

async fn socks5(stream: &mut TcpStream, proxy: &Proxy, host: &str, port: u16) -> AppResult<()> {
    let target = socks_target(proxy.url.scheme(), host, port).await?;
    let with_auth = !proxy.username.is_empty();
    stream
        .write_all(if with_auth { &[5, 2, 0, 2] } else { &[5, 1, 0] })
        .await?;
    let mut choice = [0u8; 2];
    stream.read_exact(&mut choice).await?;
    match choice {
        [5, 0] => {}
        [5, 2] if with_auth => {
            let (user, pass) = (proxy.username.as_bytes(), proxy.password.as_bytes());
            let auth = [
                &[1u8, user.len() as u8][..],
                user,
                &[pass.len() as u8],
                pass,
            ]
            .concat();
            stream.write_all(&auth).await?;
            let mut status = [0u8; 2];
            stream.read_exact(&mut status).await?;
            if status[1] != 0 {
                return Err(proxy_error("Username atau password proxy SOCKS5 ditolak"));
            }
        }
        _ => {
            return Err(proxy_error(
                "Proxy SOCKS5 tidak menerima metode autentikasi yang tersedia",
            ));
        }
    }
    stream
        .write_all(&[&[5u8, 1, 0][..], &target].concat())
        .await?;
    let mut head = [0u8; 4];
    stream.read_exact(&mut head).await?;
    if head[1] != 0 {
        return Err(proxy_error(format!(
            "Proxy SOCKS5 gagal membuka koneksi (kode {})",
            head[1]
        )));
    }
    let address_len = match head[3] {
        1 => 4,
        4 => 16,
        3 => stream.read_u8().await? as usize,
        _ => 0,
    };
    stream.read_exact(&mut vec![0u8; address_len + 2]).await?;
    Ok(())
}

/// TCP stream to `host:port`, tunnelled through the proxy when one is given.
pub async fn connect(host: &str, port: u16, proxy: Option<&Proxy>) -> AppResult<TcpStream> {
    let Some(proxy) = proxy else {
        return TcpStream::connect((host, port))
            .await
            .map_err(|e| proxy_error(format!("Tidak bisa terhubung ke {host}:{port}: {e}")));
    };
    let proxy_host = proxy.url.host_str().unwrap_or_default();
    let proxy_port = proxy.url.port_or_known_default().unwrap_or(1080);
    let mut stream = TcpStream::connect((proxy_host, proxy_port))
        .await
        .map_err(|e| {
            proxy_error(format!(
                "Tidak bisa terhubung ke proxy {proxy_host}:{proxy_port}: {e}"
            ))
        })?;
    match proxy.url.scheme() {
        "http" => http_connect(&mut stream, proxy, host, port).await?,
        "socks5" | "socks5h" => socks5(&mut stream, proxy, host, port).await?,
        other => {
            return Err(AppError::Validation(format!(
                "Proxy {other}:// belum didukung untuk WebSocket. Gunakan proxy http:// atau socks5://."
            )));
        }
    }
    Ok(stream)
}
