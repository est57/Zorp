use wiremock::matchers::{header, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::db;
use crate::models::environment::{Environment, Variable};
use crate::models::request::{Body, HttpMethod, RequestSettings};
use crate::models::script::Scripts;
use crate::services::cookies::CookieJar;
use crate::services::secrets::MemoryStore;

fn state_with_env() -> AppState {
    let conn = db::open_in_memory().unwrap();
    let ws = db::workspace::current(&conn).unwrap().id;
    let store = MemoryStore::default();
    let env = Environment {
        id: Uuid::now_v7(),
        name: "Dev".into(),
        variables: vec![Variable {
            key: "user".into(),
            value: "rani".into(),
            enabled: true,
            secret: false,
            has_value: false,
        }],
        is_active: true,
    };
    db::environments::save(&conn, ws, env.clone(), &store).unwrap();
    db::environments::set_active(&conn, ws, Some(env.id)).unwrap();
    AppState::new(
        conn,
        ws,
        Box::new(store),
        std::env::temp_dir().join("zorp-tests"),
        CookieJar::default(),
        Default::default(),
    )
}

fn spec(url: String, scripts: Scripts) -> RequestSpec {
    RequestSpec {
        docs: String::new(),
        id: Uuid::now_v7(),
        name: "Login".into(),
        method: HttpMethod::Get,
        url,
        params: vec![],
        headers: vec![],
        body: Body::None,
        auth: Auth::None,
        settings: RequestSettings::default(),
        mode: Default::default(),
        scripts,
    }
}

#[tokio::test]
async fn scripts_shape_the_request_and_update_the_environment() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(header("x-user", "rani"))
        .and(header("x-trace", "t-1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(r#"{"token":"abc"}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let state = state_with_env();
    let scripts = Scripts {
        pre_request: r#"
            pm.variables.set("trace", "t-1");
            pm.request.headers.upsert({ key: "X-User", value: "{{user}}" });
            pm.request.headers.upsert({ key: "X-Trace", value: "{{trace}}" });
        "#
        .into(),
        test: r#"
            pm.test("status", () => pm.response.to.have.status(200));
            pm.environment.set("token", pm.response.json().token);
        "#
        .into(),
    };
    let (_, result) = execute(
        &state,
        spec(server.uri(), scripts),
        Uuid::now_v7(),
        |_, _| {},
    )
    .await;
    let response = result.unwrap();
    assert_eq!(response.status, 200);
    assert!(
        response.script.tests.iter().all(|t| t.passed),
        "{:?}",
        response.script.tests
    );
    assert!(response.script.variables_changed);

    let values = db::environments::active_values(
        &state.db().unwrap(),
        state.workspace_id(),
        state.secrets.as_ref(),
    )
    .unwrap();
    assert!(values.contains(&("token".into(), "abc".into())));
}

#[tokio::test]
async fn failing_pre_request_script_stops_the_request() {
    let server = MockServer::start().await;
    let state = state_with_env();
    let scripts = Scripts {
        pre_request: "throw new Error('stop')".into(),
        test: String::new(),
    };
    let (_, result) = execute(
        &state,
        spec(server.uri(), scripts),
        Uuid::now_v7(),
        |_, _| {},
    )
    .await;
    assert!(matches!(result, Err(AppError::Script(ref m)) if m.contains("stop")));
    assert!(server.received_requests().await.unwrap().is_empty());
}
