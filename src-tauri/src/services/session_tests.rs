use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::http_engine::{execute, new_client};
use super::request_builder::build;
use super::resolver::{Variables, resolve_spec};
use crate::models::request::*;

fn spec(method: HttpMethod, url: &str) -> RequestSpec {
    RequestSpec {
        id: uuid::Uuid::now_v7(),
        name: "Test".into(),
        method,
        url: url.into(),
        params: vec![],
        headers: vec![],
        body: Body::None,
        auth: Auth::None,
        settings: RequestSettings::default(),
    }
}

#[tokio::test]
async fn timing_records_new_connections_and_reuse() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    // A hostname (not an IP literal) goes through the DNS resolver.
    let url = server.uri().replace("127.0.0.1", "localhost");
    let client = new_client(true, true, None).unwrap();
    let dir = std::env::temp_dir().join("zorp-tests");
    let req = spec(HttpMethod::Get, &url);
    let (resolved, auth) = resolve_spec(
        &req,
        &req.auth,
        &Variables::from_layers(Vec::<Vec<_>>::new()),
    )
    .unwrap();

    let first = execute(build(&client, &resolved, &auth).unwrap(), &dir, |_, _| {})
        .await
        .unwrap();
    assert!(first.timing.dns_ms.is_some());
    assert!(first.timing.connect_ms.is_some());

    let second = execute(build(&client, &resolved, &auth).unwrap(), &dir, |_, _| {})
        .await
        .unwrap();
    assert!(
        second.timing.connect_ms.is_none(),
        "keep-alive connection should be reused"
    );
}

#[tokio::test]
async fn cookie_jar_sends_cookies_set_by_previous_response() {
    let server = MockServer::start().await;
    Mock::given(path("/login"))
        .respond_with(ResponseTemplate::new(200).insert_header("set-cookie", "session=abc; Path=/"))
        .mount(&server)
        .await;
    Mock::given(path("/me"))
        .and(header("cookie", "session=abc"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(path("/me"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let jar = std::sync::Arc::new(crate::services::cookies::CookieJar::default());
    let client = new_client(true, true, Some(jar.clone())).unwrap();
    let dir = std::env::temp_dir().join("zorp-tests");
    for (route, expected) in [("/login", 200), ("/me", 200)] {
        let req = spec(HttpMethod::Get, &format!("{}{route}", server.uri()));
        let res = execute(build(&client, &req, &Auth::None).unwrap(), &dir, |_, _| {})
            .await
            .unwrap();
        assert_eq!(res.status, expected, "{route}");
    }
    assert_eq!(jar.list()[0].name, "session");
}
