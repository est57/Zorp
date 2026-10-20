use wiremock::matchers::{body_string, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::http_client::new_client;
use super::http_engine::execute;
use super::request_builder::build;
use super::resolver::{Variables, resolve_spec};
use crate::error::AppError;
use crate::models::request::*;
use crate::models::response::ResponseBody;

fn spec(method: HttpMethod, url: &str) -> RequestSpec {
    RequestSpec {
        asserts: Vec::new(),
        vars: Vec::new(),
        docs: String::new(),
        id: uuid::Uuid::now_v7(),
        name: "Test".into(),
        method,
        url: url.into(),
        params: vec![],
        headers: vec![],
        body: Body::None,
        auth: Auth::None,
        settings: RequestSettings::default(),
        mode: Default::default(),
        scripts: Default::default(),
    }
}

fn kv(key: &str, value: &str, enabled: bool) -> KeyValue {
    KeyValue {
        key: key.into(),
        value: value.into(),
        enabled,
        description: String::new(),
    }
}

async fn send(
    spec: &RequestSpec,
    vars: Variables,
) -> Result<crate::models::response::ResponseData, AppError> {
    let (resolved, auth) = resolve_spec(spec, &spec.auth, &vars)?;
    let client = new_client(true, true, None)?;
    let dir = std::env::temp_dir().join("zorp-tests");
    execute(build(&client, &resolved, &auth)?, &dir, |_, _| {}).await
}

#[tokio::test]
async fn sends_params_headers_auth_and_json_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/users"))
        .and(query_param("page", "2"))
        .and(header("authorization", "Bearer secret-token"))
        .and(header("content-type", "application/json"))
        .and(body_string(r#"{"name":"Rani"}"#))
        .respond_with(ResponseTemplate::new(201).set_body_raw(r#"{"id":1}"#, "application/json"))
        .mount(&server)
        .await;

    // Mirrors the UI: the URL holds enabled params and the params list repeats them.
    let mut req = spec(HttpMethod::Post, "{{base}}/users?page=2");
    req.params = vec![kv("page", "2", true), kv("skip", "x", false)];
    req.auth = Auth::Bearer {
        token: "{{token}}".into(),
    };
    req.body = Body::Json {
        text: r#"{"name":"{{name}}"}"#.into(),
    };
    let vars = Variables::from_layers(vec![vec![
        ("base".to_string(), server.uri()),
        ("token".to_string(), "secret-token".to_string()),
        ("name".to_string(), "Rani".to_string()),
    ]]);

    let res = send(&req, vars).await.unwrap();
    assert_eq!(res.status, 201);
    let received = server.received_requests().await.unwrap();
    assert_eq!(
        received[0].url.query(),
        Some("page=2"),
        "query params must not be duplicated"
    );
    assert!(matches!(res.body, ResponseBody::Text { ref text } if text == r#"{"id":1}"#));
}

#[tokio::test]
async fn sends_form_urlencoded_body() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .and(body_string("a=1&b=two+words"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let mut req = spec(HttpMethod::Put, &server.uri());
    req.body = Body::FormUrlEncoded {
        fields: vec![kv("a", "1", true), kv("b", "two words", true)],
    };
    assert_eq!(
        send(&req, Variables::from_layers(Vec::<Vec<_>>::new()))
            .await
            .unwrap()
            .status,
        200
    );
}

#[tokio::test]
async fn binary_response_is_base64() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(vec![0u8, 159, 146, 150], "image/png"),
        )
        .mount(&server)
        .await;

    let res = send(
        &spec(HttpMethod::Get, &server.uri()),
        Variables::from_layers(Vec::<Vec<_>>::new()),
    )
    .await
    .unwrap();
    assert!(matches!(res.body, ResponseBody::Base64 { ref mime, .. } if mime == "image/png"));
}

#[tokio::test]
async fn unresolved_variables_block_sending() {
    let req = spec(HttpMethod::Get, "{{base}}/users/{{id}}");
    let err = send(&req, Variables::from_layers(Vec::<Vec<_>>::new()))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::UnresolvedVariables(ref n) if n == &["base", "id"]));
}

#[tokio::test]
async fn timeout_is_classified() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_millis(500)))
        .mount(&server)
        .await;

    let mut req = spec(HttpMethod::Get, &server.uri());
    req.settings.timeout_ms = 50;
    let err = send(&req, Variables::from_layers(Vec::<Vec<_>>::new()))
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        AppError::Network {
            kind: crate::error::ErrorKind::Timeout,
            ..
        }
    ));
}

#[tokio::test]
async fn connection_refused_is_classified() {
    let req = spec(HttpMethod::Get, "http://127.0.0.1:9");
    let err = send(&req, Variables::from_layers(Vec::<Vec<_>>::new()))
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        AppError::Network {
            kind: crate::error::ErrorKind::Connect,
            ..
        }
    ));
}

#[tokio::test]
async fn sends_graphql_as_json_with_resolved_variables() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(header("content-type", "application/json"))
        .and(body_string(
            r#"{"query":"query($id: ID!) { user(id: $id) { name } }","variables":{"id":"42"}}"#,
        ))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let mut req = spec(HttpMethod::Post, &server.uri());
    req.body = Body::Graphql {
        query: "query($id: ID!) { user(id: $id) { name } }".into(),
        variables: r#"{"id": "{{user_id}}"}"#.into(),
    };
    let vars = Variables::from_layers(vec![vec![("user_id".to_string(), "42".to_string())]]);
    assert_eq!(send(&req, vars).await.unwrap().status, 200);

    req.body = Body::Graphql {
        query: "{ a }".into(),
        variables: "{oops".into(),
    };
    let err = send(&req, Variables::from_layers(Vec::<Vec<_>>::new()))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Validation(ref m) if m.contains("Variables GraphQL")));
}
