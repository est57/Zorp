use wiremock::matchers::{header_exists, header_regex, method};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

use super::*;
use crate::db;
use crate::models::auth::{AwsSigV4Config, JwtAlgorithm, JwtConfig};
use crate::models::request::{ApiKeyLocation, Body, HttpMethod, RequestSettings};
use crate::services::cookies::CookieJar;
use crate::services::secrets::MemoryStore;

fn state() -> AppState {
    let conn = db::open_in_memory().unwrap();
    let ws = db::workspace::current(&conn).unwrap().id;
    AppState::new(
        conn,
        ws,
        Box::new(MemoryStore::default()),
        std::env::temp_dir().join("zorp-tests"),
        CookieJar::default(),
        Default::default(),
    )
}

fn spec(url: String, auth: Auth) -> RequestSpec {
    RequestSpec {
        asserts: Vec::new(),
        vars: Vec::new(),
        docs: String::new(),
        id: Uuid::now_v7(),
        name: "Auth".into(),
        method: HttpMethod::Post,
        url,
        params: vec![],
        headers: vec![],
        body: Body::Json {
            text: "{\"a\":1}".into(),
        },
        auth,
        settings: RequestSettings::default(),
        mode: Default::default(),
        scripts: Default::default(),
    }
}

/// Challenges requests without credentials and accepts any Digest answer for user rani.
struct DigestServer;

impl Respond for DigestServer {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        match req
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
        {
            Some(v)
                if v.starts_with("Digest username=\"rani\"")
                    && v.contains("uri=\"/secure?x=1\"") =>
            {
                ResponseTemplate::new(200).set_body_string("ok")
            }
            _ => ResponseTemplate::new(401).insert_header(
                "www-authenticate",
                r#"Digest realm="zorp", qop="auth", nonce="abc123", algorithm=SHA-256"#,
            ),
        }
    }
}

#[tokio::test]
async fn digest_answers_the_challenge_once() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(DigestServer)
        .expect(2)
        .mount(&server)
        .await;
    let auth = Auth::Digest {
        username: "rani".into(),
        password: "rahasia".into(),
    };
    let (_, result) = execute(
        &state(),
        spec(format!("{}/secure?x=1", server.uri()), auth),
        Uuid::now_v7(),
        |_, _| {},
    )
    .await;
    assert_eq!(result.unwrap().status, 200);
}

#[tokio::test]
async fn jwt_and_aws_add_their_headers() {
    let server = MockServer::start().await;
    Mock::given(header_regex(
        "authorization",
        r"^Bearer eyJ[\w-]+\.eyJ[\w-]+\.[\w-]+$",
    ))
    .respond_with(ResponseTemplate::new(200))
    .mount(&server)
    .await;
    Mock::given(header_regex("authorization", r"^AWS4-HMAC-SHA256 Credential=AKID/\d{8}/eu-west-1/execute-api/aws4_request, SignedHeaders=host;x-amz-date, Signature=[0-9a-f]{64}$"))
        .and(header_exists("x-amz-date"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;
    let jwt = Auth::Jwt(JwtConfig {
        algorithm: JwtAlgorithm::HS256,
        secret: "kunci".into(),
        secret_base64: false,
        payload: "{\"sub\":\"rani\"}".into(),
        headers: String::new(),
        location: ApiKeyLocation::Header,
        param: "Authorization".into(),
        prefix: "Bearer".into(),
    });
    let (_, result) = execute(&state(), spec(server.uri(), jwt), Uuid::now_v7(), |_, _| {}).await;
    assert_eq!(result.unwrap().status, 200);
    let aws = Auth::AwsSigV4(AwsSigV4Config {
        access_key: "AKID".into(),
        secret_key: "rahasia".into(),
        session_token: String::new(),
        region: "eu-west-1".into(),
        service: "execute-api".into(),
    });
    let (_, result) = execute(&state(), spec(server.uri(), aws), Uuid::now_v7(), |_, _| {}).await;
    assert_eq!(result.unwrap().status, 201);
}
