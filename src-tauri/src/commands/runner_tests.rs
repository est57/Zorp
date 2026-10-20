use std::sync::Mutex;

use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::models::request::{Auth, Body, HttpMethod, RequestSettings, RequestSpec};
use crate::models::script::Scripts;
use crate::services::cookies::CookieJar;
use crate::services::secrets::MemoryStore;

fn state() -> AppState {
    let conn = db::open_in_memory().unwrap();
    let ws = db::workspace::current(&conn).unwrap().id;
    AppState::new(
        conn,
        ws,
        Box::new(MemoryStore::default()),
        std::env::temp_dir(),
        CookieJar::default(),
        Default::default(),
    )
}

fn item(name: &str, url: String, test: &str) -> SavedRequest {
    SavedRequest {
        collection_id: Uuid::now_v7(),
        folder_id: None,
        sort_order: 1.0,
        spec: RequestSpec {
            docs: String::new(),
            id: Uuid::now_v7(),
            name: name.into(),
            method: HttpMethod::Get,
            url,
            params: vec![],
            headers: vec![],
            body: Body::None,
            auth: Auth::None,
            settings: RequestSettings::default(),
            mode: Default::default(),
            scripts: Scripts {
                pre_request: String::new(),
                test: test.into(),
            },
        },
    }
}

async fn server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(path("/ok"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(path("/fail"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    server
}

const STATUS_200: &str = "pm.test('200', () => pm.response.to.have.status(200));";

#[tokio::test]
async fn runs_items_and_stops_on_failure() {
    let server = server().await;
    let items = vec![
        item("a", format!("{}/ok", server.uri()), STATUS_200),
        item("b", format!("{}/fail", server.uri()), STATUS_200),
        item("c", format!("{}/ok", server.uri()), STATUS_200),
    ];
    let seen = Mutex::new(Vec::new());
    let (_tx, rx) = oneshot::channel();
    let options = RunnerOptions {
        data_path: None,
        delay_ms: 0,
        stop_on_failure: true,
    };
    let summary = run_items(&state(), &items, &[], &options, rx, |r| {
        seen.lock().unwrap().push((r.name, r.passed))
    })
    .await;

    assert_eq!(
        (summary.passed, summary.failed, summary.cancelled),
        (1, 1, false)
    );
    assert_eq!(
        *seen.lock().unwrap(),
        vec![("a".to_string(), true), ("b".to_string(), false)]
    );
}

#[tokio::test]
async fn continues_after_failure_and_can_be_cancelled() {
    let server = server().await;
    let items = vec![
        item("a", format!("{}/fail", server.uri()), STATUS_200),
        item("b", format!("{}/ok", server.uri()), ""),
    ];
    let (_tx, rx) = oneshot::channel();
    let options = RunnerOptions {
        data_path: None,
        delay_ms: 0,
        stop_on_failure: false,
    };
    let summary = run_items(&state(), &items, &[], &options, rx, |_| {}).await;
    assert_eq!((summary.passed, summary.failed), (1, 1));

    let (tx, rx) = oneshot::channel();
    tx.send(()).unwrap();
    let options = RunnerOptions {
        data_path: None,
        delay_ms: 50,
        stop_on_failure: false,
    };
    let cancelled = run_items(&state(), &items, &[], &options, rx, |_| {}).await;
    assert!(cancelled.cancelled);
}

#[tokio::test]
async fn runs_once_per_data_row_with_row_values() {
    let server = MockServer::start().await;
    Mock::given(path("/users/7"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(path("/users/8"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    let items = vec![item(
        "User",
        format!("{}/users/{{{{id}}}}", server.uri()),
        "",
    )];
    let data = vec![
        vec![("id".to_string(), "7".to_string())],
        vec![("id".to_string(), "8".to_string())],
    ];
    let options = RunnerOptions {
        delay_ms: 0,
        stop_on_failure: false,
        data_path: None,
    };
    let (_tx, rx) = oneshot::channel();
    let seen = Mutex::new(Vec::new());
    let summary = run_items(&state(), &items, &data, &options, rx, |r| {
        seen.lock()
            .unwrap()
            .push((r.iteration, r.index, r.total, r.status))
    })
    .await;
    assert_eq!((summary.iterations, summary.total), (2, 2));
    assert_eq!(
        seen.into_inner().unwrap(),
        [(1, 1, 2, Some(200)), (2, 2, 2, Some(404))]
    );
}
