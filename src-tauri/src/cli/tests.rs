use wiremock::matchers::{header, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::args::{Reporter, RunArgs};
use super::run;
use crate::db;
use crate::models::collection::NodeKind;
use crate::models::environment::{Environment, Variable};
use crate::models::request::{Auth, Body, HttpMethod, RequestSpec};
use crate::models::rules::{AssertOp, Assertion};
use crate::services::secrets::MemoryStore;
use crate::services::workspace_sync::{files, render};

/// Writes a workspace folder with one request that needs a secret token.
fn workspace(dir: &std::path::Path, base: &str) {
    let conn = db::open_in_memory().unwrap();
    let ws = db::workspace::current(&conn).unwrap().id;
    let store = MemoryStore::default();
    let col = db::collections::create_collection(&conn, ws, "API").unwrap();
    db::collections::set_auth(
        &conn,
        NodeKind::Collection,
        col.id,
        &Auth::Bearer {
            token: "{{token}}".into(),
        },
    )
    .unwrap();
    let var = |key: &str, value: &str, secret| Variable {
        key: key.into(),
        value: value.into(),
        enabled: true,
        secret,
        has_value: false,
    };
    let env = Environment {
        id: uuid::Uuid::now_v7(),
        name: "Staging".into(),
        variables: vec![var("base", base, false), var("token", "s", true)],
        is_active: false,
    };
    db::environments::save(&conn, ws, env, &store).unwrap();
    let spec = RequestSpec {
        docs: String::new(),
        asserts: vec![Assertion {
            enabled: true,
            expr: "res.status".into(),
            op: AssertOp::Eq,
            value: "200".into(),
        }],
        vars: Vec::new(),
        id: uuid::Uuid::now_v7(),
        name: "Profil".into(),
        method: HttpMethod::Get,
        url: "{{base}}/me".into(),
        params: vec![],
        headers: vec![],
        body: Body::None,
        auth: Auth::Inherit,
        settings: Default::default(),
        mode: Default::default(),
        scripts: Default::default(),
    };
    db::requests::save(&conn, spec, col.id, None).unwrap();
    let snapshot = db::snapshot::load(&conn, ws).unwrap();
    files::write(dir, &render::render(&snapshot).unwrap()).unwrap();
}

fn args(dir: &std::path::Path) -> RunArgs {
    RunArgs {
        folder: dir.to_string_lossy().into(),
        environment: Some("staging".into()),
        ..Default::default()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn runs_a_workspace_folder_and_reports_failures() {
    let server = MockServer::start().await;
    Mock::given(path("/me"))
        .and(header("authorization", "Bearer rahasia"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let dir = tempfile::tempdir().unwrap();
    workspace(dir.path(), &server.uri());

    let mut ok = args(dir.path());
    ok.vars = vec![("token".into(), "rahasia".into())];
    ok.reporter = Reporter::Junit;
    let report = dir.path().join("hasil.xml");
    ok.out = Some(report.to_string_lossy().into());
    assert_eq!(run(ok).await.unwrap(), 0);
    let xml = std::fs::read_to_string(&report).unwrap();
    assert!(xml.contains("tests=\"1\" failures=\"0\""), "{xml}");

    let mut wrong = args(dir.path());
    wrong.vars = vec![("token".into(), "salah".into())];
    assert_eq!(
        run(wrong).await.unwrap(),
        1,
        "token salah membuat assert gagal"
    );

    let mut missing = args(dir.path());
    missing.environment = Some("Produksi".into());
    let error = run(missing).await.unwrap_err().to_string();
    assert!(error.contains("Pilihan: Staging"), "{error}");
}
