use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::db;
use crate::models::request::{Body, HttpMethod, RequestSettings};
use crate::models::rules::{AssertOp, Assertion, VarPhase, VarRule, VarTarget};
use crate::services::cookies::CookieJar;
use crate::services::secrets::MemoryStore;

fn spec(url: String) -> RequestSpec {
    RequestSpec {
        docs: String::new(),
        asserts: Vec::new(),
        vars: Vec::new(),
        id: Uuid::now_v7(),
        name: "Login".into(),
        method: HttpMethod::Post,
        url,
        params: vec![],
        headers: vec![],
        body: Body::None,
        auth: Auth::None,
        settings: RequestSettings::default(),
        mode: Default::default(),
        scripts: Default::default(),
    }
}

#[tokio::test]
async fn asserts_and_vars_work_without_scripts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/login"))
        .and(header("x-device", "kasir-1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(r#"{"token":"t-9","user":{"outlet":242}}"#),
        )
        .mount(&server)
        .await;
    let conn = db::open_in_memory().unwrap();
    let ws = db::workspace::current(&conn).unwrap().id;
    let col = db::collections::create_collection(&conn, ws, "API").unwrap();
    let mut login = spec(format!("{}/login", server.uri()));
    login.headers.push(crate::models::request::KeyValue {
        key: "X-Device".into(),
        value: "{{device}}".into(),
        enabled: true,
        description: String::new(),
    });
    login.vars = vec![
        VarRule {
            enabled: true,
            phase: VarPhase::Pre,
            name: "device".into(),
            expr: "kasir-1".into(),
            target: VarTarget::Collection,
        },
        VarRule {
            enabled: true,
            phase: VarPhase::Post,
            name: "token".into(),
            expr: "res.body.token".into(),
            target: VarTarget::Collection,
        },
        VarRule {
            enabled: true,
            phase: VarPhase::Post,
            name: "outlet".into(),
            expr: "res.body.user.outlet".into(),
            target: VarTarget::Environment,
        },
    ];
    login.asserts = vec![
        Assertion {
            enabled: true,
            expr: "res.status".into(),
            op: AssertOp::Eq,
            value: "200".into(),
        },
        Assertion {
            enabled: true,
            expr: "res.body.token".into(),
            op: AssertOp::Eq,
            value: "{{expected}}".into(),
        },
    ];
    db::requests::save(&conn, login.clone(), col.id, None).unwrap();
    db::globals::save(
        &conn,
        ws,
        crate::models::environment::Environment {
            id: ws,
            name: String::new(),
            variables: vec![crate::models::environment::Variable {
                key: "expected".into(),
                value: "t-9".into(),
                enabled: true,
                secret: false,
                has_value: false,
            }],
            is_active: false,
        },
        &MemoryStore::default(),
    )
    .unwrap();
    let state = AppState::new(
        conn,
        ws,
        Box::new(MemoryStore::default()),
        std::env::temp_dir().join("zorp-tests"),
        CookieJar::default(),
        Default::default(),
    );

    let (_, result) = execute(&state, login, Uuid::now_v7(), |_, _| {}).await;
    let report = result.unwrap().script;
    assert!(report.tests.iter().all(|t| t.passed), "{:?}", report.tests);
    assert_eq!(report.tests.len(), 2);
    let captured: Vec<_> = report
        .captured
        .iter()
        .map(|c| (c.name.as_str(), c.value.as_deref()))
        .collect();
    assert_eq!(captured, [("token", Some("t-9")), ("outlet", Some("242"))]);
    assert_eq!(
        db::collections::variables(&state.db().unwrap(), col.id).unwrap(),
        [("token".to_string(), "t-9".to_string())]
    );
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("pilih environment")),
        "{:?}",
        report.errors
    );
}
