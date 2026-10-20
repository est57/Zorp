use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::mpsc;
use uuid::Uuid;

use super::test_servers::{Shared, echo_server, fixture, http_proxy, socks_proxy};
use super::transport::{self, bypassed};
use super::websocket::{self, Network};
use super::{Emit, Outgoing, tls};
use crate::models::realtime::{RealtimeKind, RequestMode};
use crate::models::request::{Auth, Body, HttpMethod, RequestSettings, RequestSpec};
use crate::models::settings::{CaCertificate, ClientCertificate, ProxySettings, WorkspaceSettings};

fn spec(url: String) -> RequestSpec {
    RequestSpec {
        docs: String::new(),
        id: Uuid::now_v7(),
        name: "ws".into(),
        method: HttpMethod::Get,
        url,
        params: vec![],
        headers: vec![],
        body: Body::None,
        auth: Auth::None,
        settings: RequestSettings::default(),
        scripts: Default::default(),
        mode: RequestMode::Websocket,
    }
}

/// Connects, sends "ping" and returns the event log.
async fn exchange(url: String, network: Network) -> Vec<String> {
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();
    let emit: Emit = Box::new(move |kind: RealtimeKind, data, _, _| {
        sink.lock().unwrap().push(format!("{kind:?}:{data}"))
    });
    let (tx, rx) = mpsc::unbounded_channel();
    let task = tokio::spawn(websocket::run(spec(url), Auth::None, network, rx, emit));
    tokio::time::sleep(Duration::from_millis(400)).await;
    let _ = tx.send(Outgoing::Text("ping".into()));
    tokio::time::sleep(Duration::from_millis(300)).await;
    let _ = tx.send(Outgoing::Close);
    task.await.unwrap();
    events.lock().unwrap().clone()
}

fn network(
    settings: &WorkspaceSettings,
    verify: bool,
    cert: Option<&ClientCertificate>,
    proxy: Option<transport::Proxy>,
) -> Network {
    Network {
        proxy,
        tls: tls::client_config(settings, verify, cert).unwrap(),
        timeout: Duration::from_secs(5),
    }
}

fn echoed(events: &[String]) -> bool {
    events.iter().any(|e| e == "Message:ping")
}

#[test]
fn bypass_list_matches_hosts_and_domains() {
    let list = "localhost, .internal.test, *.corp.test";
    assert!(
        bypassed(list, "localhost")
            && bypassed(list, "api.internal.test")
            && bypassed(list, "corp.test")
    );
    assert!(
        !bypassed(list, "example.com") && !bypassed("", "localhost") && bypassed("*", "anything")
    );
}

#[tokio::test]
async fn websocket_through_http_connect_proxy_sends_credentials() {
    let target = echo_server(false, false).await;
    let seen: Shared = Arc::default();
    let proxy_port = http_proxy(seen.clone()).await;
    let settings = ProxySettings {
        enabled: true,
        url: format!("http://127.0.0.1:{proxy_port}"),
        username: "rani".into(),
        ..Default::default()
    };
    let proxy = transport::select(&settings, Some("rahasia".into()), "localhost").unwrap();
    let events = exchange(
        format!("ws://localhost:{target}"),
        network(&WorkspaceSettings::default(), true, None, proxy),
    )
    .await;
    assert!(echoed(&events), "{events:?}");
    let head = seen.lock().unwrap().clone();
    assert!(head.starts_with(&format!("CONNECT localhost:{target} HTTP/1.1")));
    assert!(head.contains("Proxy-Authorization: Basic cmFuaTpyYWhhc2lh"));
}

#[tokio::test]
async fn websocket_through_socks5h_proxy_with_auth() {
    let target = echo_server(false, false).await;
    let seen: Shared = Arc::default();
    let proxy_port = socks_proxy(seen.clone()).await;
    let settings = ProxySettings {
        enabled: true,
        url: format!("socks5h://127.0.0.1:{proxy_port}"),
        username: "rani".into(),
        ..Default::default()
    };
    let proxy = transport::select(&settings, Some("x".into()), "localhost").unwrap();
    let events = exchange(
        format!("ws://localhost:{target}"),
        network(&WorkspaceSettings::default(), true, None, proxy),
    )
    .await;
    assert!(echoed(&events), "{events:?}");
    assert_eq!(*seen.lock().unwrap(), format!("rani localhost:{target}"));
}

#[tokio::test]
async fn wss_trusts_custom_ca_and_honours_verify_off() {
    let port = echo_server(true, false).await;
    let url = format!("wss://localhost:{port}");
    let with_ca = WorkspaceSettings {
        ca_certificates: vec![CaCertificate {
            id: Uuid::now_v7(),
            path: fixture("ws-ca.pem"),
            enabled: true,
        }],
        ..Default::default()
    };
    assert!(echoed(
        &exchange(url.clone(), network(&with_ca, true, None, None)).await
    ));
    let untrusted = exchange(
        url.clone(),
        network(&WorkspaceSettings::default(), true, None, None),
    )
    .await;
    assert!(untrusted[0].starts_with("Error:"), "{untrusted:?}");
    assert!(echoed(
        &exchange(
            url,
            network(&WorkspaceSettings::default(), false, None, None)
        )
        .await
    ));
}

#[tokio::test]
async fn wss_presents_client_certificate_for_mtls() {
    let port = echo_server(true, true).await;
    let url = format!("wss://localhost:{port}");
    let settings = WorkspaceSettings {
        ca_certificates: vec![CaCertificate {
            id: Uuid::now_v7(),
            path: fixture("ws-ca.pem"),
            enabled: true,
        }],
        ..Default::default()
    };
    let cert = ClientCertificate {
        id: Uuid::now_v7(),
        host: "localhost".into(),
        cert_path: fixture("ws-client.pem"),
        key_path: fixture("ws-client-key.pem"),
        enabled: true,
    };
    assert!(echoed(
        &exchange(url.clone(), network(&settings, true, Some(&cert), None)).await
    ));
    let without = exchange(url, network(&settings, true, None, None)).await;
    assert!(!echoed(&without), "{without:?}");
}
