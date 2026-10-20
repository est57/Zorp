use std::sync::{Arc, Mutex};

use uuid::Uuid;

use super::routes;
use super::server;
use crate::models::collection::SavedRequest;
use crate::models::example::ResponseExample;
use crate::models::request::{Auth, Body, HttpMethod, RequestSpec};

fn saved(method: HttpMethod, url: &str) -> SavedRequest {
    SavedRequest {
        collection_id: Uuid::nil(),
        folder_id: None,
        sort_order: 1.0,
        spec: RequestSpec {
            docs: String::new(),
            asserts: Vec::new(),
            vars: Vec::new(),
            id: Uuid::now_v7(),
            name: format!("{method:?} {url}"),
            method,
            url: url.into(),
            params: vec![],
            headers: vec![],
            body: Body::None,
            auth: Auth::None,
            settings: Default::default(),
            scripts: Default::default(),
            mode: Default::default(),
        },
    }
}

fn example(
    request: &SavedRequest,
    name: &str,
    status: u16,
    body: &str,
    order: f64,
) -> ResponseExample {
    ResponseExample {
        id: Uuid::now_v7(),
        request_id: request.spec.id,
        name: name.into(),
        status,
        status_text: String::new(),
        headers: vec![
            ("content-type".into(), "application/json".into()),
            ("content-length".into(), "999".into()),
        ],
        body: body.into(),
        sort_order: order,
    }
}

#[test]
fn routes_match_paths_and_parameters() {
    let by_id = saved(HttpMethod::Get, "{{baseUrl}}/users/{{id}}?full=1");
    let me = saved(HttpMethod::Get, "https://api.contoh.id/users/me");
    let login = saved(HttpMethod::Post, "{{baseUrl}}/api/login");
    let all = routes::build(&[by_id.clone(), me.clone(), login.clone()], &[]);
    assert_eq!(
        routes::find(&all, "GET", "/users/42").unwrap().request_name,
        by_id.spec.name
    );
    assert_eq!(
        routes::find(&all, "GET", "/users/me").unwrap().request_name,
        me.spec.name
    );
    assert!(
        routes::find(&all, "GET", "/api/login").is_none(),
        "method harus cocok"
    );
    let info = routes::info(&all);
    assert!(info.iter().any(|r| r.path == "/users/:id"));
}

#[tokio::test]
async fn serves_examples_over_http() {
    let login = saved(HttpMethod::Post, "{{baseUrl}}/api/login");
    let empty = saved(HttpMethod::Post, "{{baseUrl}}/checklists");
    let examples = vec![
        example(&login, "Login berhasil", 200, r#"{"token":"t"}"#, 1.0),
        example(&login, "Password salah", 401, r#"{"error":"salah"}"#, 2.0),
    ];
    let all = routes::build(&[login, empty], &examples);
    let log = Arc::new(Mutex::new(Vec::new()));
    let sink = log.clone();
    let running = server::start(
        Uuid::nil(),
        0,
        Arc::new(move || Ok(all.clone())),
        Arc::new(move |entry| sink.lock().unwrap().push(entry)),
    )
    .await
    .unwrap();
    let base = format!("http://127.0.0.1:{}", running.port);
    let client = reqwest::Client::new();

    let ok = client
        .post(format!("{base}/api/login"))
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), 200);
    assert_eq!(ok.headers()["access-control-allow-origin"], "*");
    assert_eq!(ok.text().await.unwrap(), r#"{"token":"t"}"#);
    let chosen = client
        .post(format!("{base}/api/login"))
        .header("x-zorp-example", "password salah")
        .send()
        .await
        .unwrap();
    assert_eq!(chosen.status(), 401);
    let missing = client
        .post(format!("{base}/checklists"))
        .send()
        .await
        .unwrap();
    assert_eq!(missing.status(), 501);
    assert!(
        missing
            .text()
            .await
            .unwrap()
            .contains("Simpan sebagai contoh")
    );
    assert_eq!(
        client
            .get(format!("{base}/nope"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );

    let statuses: Vec<_> = log.lock().unwrap().iter().map(|e| e.status).collect();
    assert_eq!(statuses, [200, 401, 501, 404]);
    running.stop();
}
