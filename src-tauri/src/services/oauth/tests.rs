use tokio::sync::oneshot;
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::models::oauth::ClientAuth;
use crate::services::http_engine::new_client;
use crate::services::secrets::MemoryStore;

fn config(server: &MockServer, grant: OAuthGrant) -> OAuth2Config {
    OAuth2Config {
        grant,
        auth_url: format!("{}/authorize", server.uri()),
        token_url: format!("{}/token", server.uri()),
        client_id: "zorp".into(),
        client_secret: "s3cret".into(),
        scope: "read".into(),
        use_pkce: true,
        client_auth: ClientAuth::Basic,
        callback_port: 0,
        token_key: "test".into(),
    }
}

fn token_response(body: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(body.to_string(), "application/json")
}

#[tokio::test]
async fn client_credentials_uses_basic_auth_and_parses_expiry() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(header("authorization", "Basic em9ycDpzM2NyZXQ="))
        .and(body_string_contains("grant_type=client_credentials"))
        .and(body_string_contains("scope=read"))
        .respond_with(token_response(
            r#"{"access_token":"cc-1","expires_in":3600}"#,
        ))
        .mount(&server)
        .await;

    let client = new_client(true, true).unwrap();
    let (_tx, rx) = oneshot::channel();
    let token = fetch_token(
        &client,
        &config(&server, OAuthGrant::ClientCredentials),
        |_| Ok(()),
        rx,
    )
    .await
    .unwrap();
    assert_eq!(token.access_token, "cc-1");
    assert!(token.expires_at.unwrap() > now_ms());
}

#[tokio::test]
async fn expired_token_is_refreshed_and_keeps_refresh_token() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(body_string_contains("grant_type=refresh_token"))
        .and(body_string_contains("refresh_token=r-1"))
        .respond_with(token_response(r#"{"access_token":"new","expires_in":60}"#))
        .mount(&server)
        .await;

    let store = MemoryStore::default();
    let cfg = config(&server, OAuthGrant::AuthorizationCode);
    let old = StoredToken {
        access_token: "old".into(),
        token_type: "Bearer".into(),
        refresh_token: Some("r-1".into()),
        expires_at: Some(now_ms() - 1000),
        scope: None,
    };
    token_store::save(&store, &cfg.token_key, &old).unwrap();

    let client = new_client(true, true).unwrap();
    assert_eq!(access_token(&store, &client, &cfg).await.unwrap(), "new");
    let saved = token_store::load(&store, &cfg.token_key).unwrap().unwrap();
    assert_eq!(saved.refresh_token.as_deref(), Some("r-1"));
}

#[tokio::test]
async fn missing_token_for_authorization_code_is_an_error() {
    let server = MockServer::start().await;
    let client = new_client(true, true).unwrap();
    let err = access_token(
        &MemoryStore::default(),
        &client,
        &config(&server, OAuthGrant::AuthorizationCode),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::OAuth(_)));
}

/// Simulates the provider redirecting the browser back to the loopback callback.
fn redirect_with(auth_url: &str, code: &str, override_state: Option<&str>) {
    let url = url::Url::parse(auth_url).unwrap();
    let param = |name: &str| {
        url.query_pairs()
            .find(|(k, _)| k == name)
            .unwrap()
            .1
            .into_owned()
    };
    assert_eq!(param("code_challenge_method"), "S256");
    let state = override_state
        .map(str::to_string)
        .unwrap_or_else(|| param("state"));
    let target = format!("{}?code={code}&state={state}", param("redirect_uri"));
    tokio::spawn(async move {
        let _ = reqwest::get(target).await;
    });
}

#[tokio::test]
async fn authorization_code_flow_exchanges_code_with_pkce_verifier() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .and(body_string_contains("code=abc"))
        .and(body_string_contains("code_verifier="))
        .respond_with(token_response(
            r#"{"access_token":"ac-1","refresh_token":"r","token_type":"bearer"}"#,
        ))
        .mount(&server)
        .await;

    let client = new_client(true, true).unwrap();
    let (_tx, rx) = oneshot::channel();
    let cfg = config(&server, OAuthGrant::AuthorizationCode);
    let open = |url: &str| {
        redirect_with(url, "abc", None);
        Ok(())
    };
    let token = fetch_token(&client, &cfg, open, rx).await.unwrap();
    assert_eq!(token.access_token, "ac-1");
    assert_eq!(token.refresh_token.as_deref(), Some("r"));
}

#[tokio::test]
async fn authorization_code_rejects_mismatched_state() {
    let server = MockServer::start().await;
    let client = new_client(true, true).unwrap();
    let (_tx, rx) = oneshot::channel();
    let open = |url: &str| {
        redirect_with(url, "abc", Some("forged"));
        Ok(())
    };
    let err = fetch_token(
        &client,
        &config(&server, OAuthGrant::AuthorizationCode),
        open,
        rx,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, AppError::OAuth(ref m) if m.contains("state")));
}

#[tokio::test]
async fn token_endpoint_errors_are_reported() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401).set_body_raw(
            r#"{"error":"invalid_client","error_description":"Unknown client"}"#,
            "application/json",
        ))
        .mount(&server)
        .await;
    let client = new_client(true, true).unwrap();
    let err = token_endpoint::client_credentials(
        &client,
        &config(&server, OAuthGrant::ClientCredentials),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, AppError::OAuth(ref m) if m.contains("invalid_client") && m.contains("Unknown client"))
    );
}
