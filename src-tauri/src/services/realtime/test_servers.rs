//! Local WebSocket, HTTP CONNECT and SOCKS5 servers for the real-time network tests.
use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use rustls::crypto::aws_lc_rs;
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
pub type Shared = Arc<Mutex<String>>;

pub fn fixture(name: &str) -> String {
    format!("{FIXTURES}/{name}")
}

async fn echo<S: AsyncRead + AsyncWrite + Unpin>(stream: S) {
    let Ok(mut ws) = tokio_tungstenite::accept_async(stream).await else {
        return;
    };
    while let Some(Ok(message)) = ws.next().await {
        if message.is_text() && ws.send(message).await.is_err() {
            break;
        }
    }
}

/// Plain or TLS echo server; with `client_ca`, clients must present a certificate signed by it.
pub async fn echo_server(tls: bool, client_ca: bool) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let acceptor = tls.then(|| {
        let provider = Arc::new(aws_lc_rs::default_provider());
        let certs = CertificateDer::pem_file_iter(fixture("ws-server.pem"))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let key = PrivateKeyDer::from_pem_file(fixture("ws-server-key.pem")).unwrap();
        let builder = ServerConfig::builder_with_provider(provider.clone())
            .with_safe_default_protocol_versions()
            .unwrap();
        let builder = if client_ca {
            let mut roots = RootCertStore::empty();
            roots
                .add(CertificateDer::from_pem_file(fixture("ws-ca.pem")).unwrap())
                .unwrap();
            builder.with_client_cert_verifier(
                WebPkiClientVerifier::builder_with_provider(Arc::new(roots), provider)
                    .build()
                    .unwrap(),
            )
        } else {
            builder.with_no_client_auth()
        };
        tokio_rustls::TlsAcceptor::from(Arc::new(builder.with_single_cert(certs, key).unwrap()))
    });
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                match acceptor {
                    Some(acceptor) => {
                        if let Ok(tls) = acceptor.accept(stream).await {
                            echo(tls).await;
                        }
                    }
                    None => echo(stream).await,
                }
            });
        }
    });
    port
}

async fn read_until(stream: &mut TcpStream, end: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    while !buf.ends_with(end) {
        buf.push(stream.read_u8().await.unwrap());
    }
    buf
}

/// HTTP CONNECT proxy that records the request it received.
pub async fn http_proxy(seen: Shared) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (mut client, _) = listener.accept().await.unwrap();
        let head = String::from_utf8(read_until(&mut client, b"\r\n\r\n").await).unwrap();
        let target = head.split_whitespace().nth(1).unwrap().to_string();
        *seen.lock().unwrap() = head;
        let mut upstream = TcpStream::connect(target).await.unwrap();
        client
            .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
            .await
            .unwrap();
        let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
    });
    port
}

/// SOCKS5 proxy with username/password auth that records "user target".
pub async fn socks_proxy(seen: Shared) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (mut c, _) = listener.accept().await.unwrap();
        let n = {
            let mut h = [0u8; 2];
            c.read_exact(&mut h).await.unwrap();
            h[1] as usize
        };
        c.read_exact(&mut vec![0u8; n]).await.unwrap();
        c.write_all(&[5, 2]).await.unwrap();
        let mut ver_len = [0u8; 2];
        c.read_exact(&mut ver_len).await.unwrap();
        let mut user = vec![0u8; ver_len[1] as usize];
        c.read_exact(&mut user).await.unwrap();
        let pass_len = c.read_u8().await.unwrap() as usize;
        c.read_exact(&mut vec![0u8; pass_len]).await.unwrap();
        c.write_all(&[1, 0]).await.unwrap();
        let mut head = [0u8; 4];
        c.read_exact(&mut head).await.unwrap();
        assert_eq!(head[3], 3, "socks5h must send the host name");
        let len = c.read_u8().await.unwrap() as usize;
        let mut host = vec![0u8; len];
        c.read_exact(&mut host).await.unwrap();
        let port = c.read_u16().await.unwrap();
        let host = String::from_utf8(host).unwrap();
        *seen.lock().unwrap() = format!("{} {host}:{port}", String::from_utf8(user).unwrap());
        let mut upstream = TcpStream::connect((host.as_str(), port)).await.unwrap();
        c.write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0]).await.unwrap();
        let _ = tokio::io::copy_bidirectional(&mut c, &mut upstream).await;
    });
    port
}
