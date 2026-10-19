use uuid::Uuid;

use super::*;
use crate::models::settings::CaCertificate;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

fn client_cert(host: &str) -> ClientCertificate {
    ClientCertificate {
        id: Uuid::now_v7(),
        host: host.into(),
        cert_path: format!("{FIXTURES}/client-cert.pem"),
        key_path: format!("{FIXTURES}/client-key.pem"),
        enabled: true,
    }
}

#[test]
fn validates_proxy_urls() {
    assert!(validate_proxy_url("http://127.0.0.1:8080").is_ok());
    assert!(validate_proxy_url("socks5h://proxy.local:1080").is_ok());
    assert!(validate_proxy_url("ftp://proxy.local").is_err());
    assert!(validate_proxy_url("127.0.0.1:8080").is_err());
}

#[test]
fn disabled_proxy_is_not_built() {
    let settings = ProxySettings {
        enabled: false,
        url: "http://p:1".into(),
        ..Default::default()
    };
    assert!(build_proxy(&settings, None).unwrap().is_none());
}

#[test]
fn picks_exact_host_before_wildcard() {
    let settings = WorkspaceSettings {
        client_certificates: vec![client_cert("*.corp.test"), client_cert("api.corp.test")],
        ..Default::default()
    };
    assert_eq!(
        certificate_for(&settings, "https://api.corp.test/x")
            .unwrap()
            .host,
        "api.corp.test"
    );
    assert_eq!(
        certificate_for(&settings, "https://db.corp.test")
            .unwrap()
            .host,
        "*.corp.test"
    );
    assert!(certificate_for(&settings, "https://corp.test").is_none());
}

#[test]
fn loads_pem_roots_and_identity_into_a_client() {
    let settings = WorkspaceSettings {
        ca_certificates: vec![CaCertificate {
            id: Uuid::now_v7(),
            path: format!("{FIXTURES}/ca-cert.pem"),
            enabled: true,
        }],
        ..Default::default()
    };
    let roots = load_roots(&settings).unwrap();
    assert_eq!(roots.len(), 1);
    let identity = load_identity(&client_cert("api.corp.test")).unwrap();
    reqwest::Client::builder()
        .tls_certs_merge(roots)
        .identity(identity)
        .build()
        .unwrap();
}

#[test]
fn rejects_invalid_certificate_files() {
    let settings = WorkspaceSettings {
        ca_certificates: vec![CaCertificate {
            id: Uuid::now_v7(),
            path: format!("{FIXTURES}/README.md"),
            enabled: true,
        }],
        ..Default::default()
    };
    assert!(load_roots(&settings).is_err());
    let mut cert = client_cert("x");
    cert.key_path = format!("{FIXTURES}/README.md");
    assert!(load_identity(&cert).is_err());
}
