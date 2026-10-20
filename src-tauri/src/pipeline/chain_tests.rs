use wiremock::matchers::{header, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::db;
use crate::models::collection::NodeKind;
use crate::models::request::{Body, HttpMethod, RequestSettings};
use crate::models::script::Scripts;
use crate::services::cookies::CookieJar;
use crate::services::secrets::MemoryStore;

fn scripts(pre: &str, test: &str) -> Scripts {
    Scripts {
        pre_request: pre.into(),
        test: test.into(),
    }
}

#[tokio::test]
async fn collection_and_folder_scripts_run_before_the_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(header("x-app", "zorp"))
        .and(header("x-outlet", "242"))
        .and(header("x-step", "folder"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let conn = db::open_in_memory().unwrap();
    let ws = db::workspace::current(&conn).unwrap().id;
    let col = db::collections::create_collection(&conn, ws, "API").unwrap();
    let folder = db::folders::create_folder(&conn, col.id, None, "Outlet").unwrap();
    db::node_settings::set(
        &conn,
        NodeKind::Collection,
        col.id,
        &scripts(
            r#"pm.variables.set("outlet", "242"); pm.request.headers.upsert({ key: "X-App", value: "zorp" });"#,
            r#"pm.test("koleksi", () => pm.expect(pm.response.code).to.equal(200));"#,
        ),
        "",
    )
    .unwrap();
    db::node_settings::set(
        &conn,
        NodeKind::Folder,
        folder.id,
        &scripts(
            r#"pm.request.headers.upsert({ key: "X-Step", value: "folder" });"#,
            "",
        ),
        "",
    )
    .unwrap();
    let spec = RequestSpec {
        docs: String::new(),
        id: Uuid::now_v7(),
        name: "Checklist".into(),
        method: HttpMethod::Get,
        url: server.uri(),
        params: vec![],
        headers: vec![],
        body: Body::None,
        auth: Auth::None,
        settings: RequestSettings::default(),
        mode: Default::default(),
        scripts: scripts(
            r#"pm.request.headers.upsert({ key: "X-Outlet", value: pm.variables.get("outlet") });"#,
            r#"pm.test("request", () => pm.expect(true).to.equal(true));"#,
        ),
    };
    db::requests::save(&conn, spec.clone(), col.id, Some(folder.id)).unwrap();
    let state = AppState::new(
        conn,
        ws,
        Box::new(MemoryStore::default()),
        std::env::temp_dir().join("zorp-tests"),
        CookieJar::default(),
        Default::default(),
    );

    let (_, result) = execute(&state, spec, Uuid::now_v7(), |_, _| {}).await;
    let response = result.unwrap();
    assert_eq!(response.status, 200, "header dari ketiga script terkirim");
    let names: Vec<_> = response
        .script
        .tests
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(names, ["koleksi", "request"]);
}

#[tokio::test]
async fn collection_script_error_names_its_source() {
    let conn = db::open_in_memory().unwrap();
    let ws = db::workspace::current(&conn).unwrap().id;
    let col = db::collections::create_collection(&conn, ws, "API").unwrap();
    db::node_settings::set(
        &conn,
        NodeKind::Collection,
        col.id,
        &scripts("throw new Error('rusak')", ""),
        "",
    )
    .unwrap();
    let spec = RequestSpec {
        docs: String::new(),
        id: Uuid::now_v7(),
        name: "x".into(),
        method: HttpMethod::Get,
        url: "http://127.0.0.1:9".into(),
        params: vec![],
        headers: vec![],
        body: Body::None,
        auth: Auth::None,
        settings: RequestSettings::default(),
        mode: Default::default(),
        scripts: Scripts::default(),
    };
    db::requests::save(&conn, spec.clone(), col.id, None).unwrap();
    let state = AppState::new(
        conn,
        ws,
        Box::new(MemoryStore::default()),
        std::env::temp_dir().join("zorp-tests"),
        CookieJar::default(),
        Default::default(),
    );
    let (_, result) = execute(&state, spec, Uuid::now_v7(), |_, _| {}).await;
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains("pre-request koleksi API") && message.contains("rusak"),
        "{message}"
    );
}

#[tokio::test]
async fn collection_script_can_log_in_with_send_request() {
    use wiremock::matchers::path;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"token":"t-123"}"#))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/me"))
        .and(header("authorization", "Bearer t-123"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let conn = db::open_in_memory().unwrap();
    let ws = db::workspace::current(&conn).unwrap().id;
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
    let login = format!(
        r#"if (!pm.collectionVariables.get("token")) {{
            const res = pm.sendRequest({{ url: "{}/login", method: "POST" }});
            pm.collectionVariables.set("token", res.json().token);
        }}"#,
        server.uri()
    );
    db::node_settings::set(
        &conn,
        NodeKind::Collection,
        col.id,
        &scripts(&login, ""),
        "",
    )
    .unwrap();
    let spec = RequestSpec {
        docs: String::new(),
        id: Uuid::now_v7(),
        name: "Me".into(),
        method: HttpMethod::Get,
        url: format!("{}/me", server.uri()),
        params: vec![],
        headers: vec![],
        body: Body::None,
        auth: Auth::Inherit,
        settings: RequestSettings::default(),
        mode: Default::default(),
        scripts: Scripts::default(),
    };
    db::requests::save(&conn, spec.clone(), col.id, None).unwrap();
    let state = AppState::new(
        conn,
        ws,
        Box::new(MemoryStore::default()),
        std::env::temp_dir().join("zorp-tests"),
        CookieJar::default(),
        Default::default(),
    );

    for _ in 0..2 {
        let (_, result) = execute(&state, spec.clone(), Uuid::now_v7(), |_, _| {}).await;
        assert_eq!(
            result.unwrap().status,
            200,
            "token dipakai dan login hanya sekali"
        );
    }
}
