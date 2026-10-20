use std::sync::Arc;

use super::*;

fn input(kind: ScriptKind, script: &str) -> ScriptInput {
    ScriptInput {
        kind,
        script: script.into(),
        request_name: "List users".into(),
        request: ScriptRequest {
            method: "GET".into(),
            url: "https://api.test/users".into(),
            headers: vec![("Accept".into(), "application/json".into())],
            body: None,
        },
        response: (kind == ScriptKind::Test).then(|| ScriptResponse {
            code: 200,
            status: "OK".into(),
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: r#"{"data":[{"id":1,"name":"Rani"}],"total":1}"#.into(),
            response_time: 42,
        }),
        variables: HashMap::from([("base".into(), "https://api.test".into())]),
        environment: HashMap::from([("token".into(), "old".into())]),
        collection_variables: HashMap::new(),
        globals: HashMap::from([("app".into(), "zorp".into())]),
        has_environment: true,
    }
}

#[test]
fn pre_request_can_change_variables_and_request() {
    let out = run(&input(
        ScriptKind::PreRequest,
        r#"
        pm.environment.set("token", "new");
        pm.collectionVariables.set("count", 3);
        pm.variables.set("trace", "abc");
        pm.request.headers.upsert({ key: "X-Trace", value: pm.variables.get("trace") });
        pm.request.url = pm.request.url + "?page=2";
        console.log("token was", pm.environment.get("token"));
        "#,
    ))
    .unwrap();
    assert_eq!(out.error, None);
    assert_eq!(out.env.get("token").map(String::as_str), Some("new"));
    assert_eq!(out.coll.get("count").map(String::as_str), Some("3"));
    assert_eq!(out.locals.get("trace").map(String::as_str), Some("abc"));
    assert!(
        out.request
            .headers
            .contains(&("X-Trace".into(), "abc".into()))
    );
    assert_eq!(out.request.url, "https://api.test/users?page=2");
    assert_eq!(out.logs[0].message, "token was new");
}

#[test]
fn tests_report_passes_and_failures() {
    let out = run(&input(
        ScriptKind::Test,
        r#"
        pm.test("status is 200", () => pm.response.to.have.status(200));
        pm.test("has one user", () => {
            const body = pm.response.json();
            pm.expect(body.data).to.have.lengthOf(1);
            pm.expect(body.data[0]).to.have.property("name", "Rani");
            pm.expect(body).to.deep.include({ total: 1 });
        });
        pm.test("fast", () => pm.expect(pm.response.responseTime).to.be.below(10));
        pm.test("header", () => pm.response.to.have.header("content-type"));
        tests["legacy"] = pm.response.code === 200;
        "#,
    ))
    .unwrap();
    let summary: Vec<(&str, bool)> = out
        .tests
        .iter()
        .map(|t| (t.name.as_str(), t.passed))
        .collect();
    assert_eq!(
        summary,
        vec![
            ("status is 200", true),
            ("has one user", true),
            ("fast", false),
            ("header", true),
            ("legacy", true)
        ]
    );
    assert!(out.tests[2].error.as_deref().unwrap().contains("below 10"));
}

#[test]
fn script_errors_are_reported() {
    let thrown = run(&input(ScriptKind::Test, "throw new Error('boom')")).unwrap();
    assert_eq!(thrown.error.as_deref(), Some("boom"));
    let syntax = run(&input(ScriptKind::Test, "pm.test(")).unwrap();
    assert!(syntax.error.unwrap().starts_with("Syntax error"));
}

#[test]
fn environment_set_requires_active_environment() {
    let mut no_env = input(ScriptKind::PreRequest, "pm.environment.set('a', 1)");
    no_env.has_environment = false;
    let out = run(&no_env).unwrap();
    assert!(out.error.unwrap().contains("Tidak ada environment aktif"));
    assert!(out.env.is_empty());
}

#[test]
fn sandbox_has_no_host_access() {
    let out = run(&input(
        ScriptKind::Test,
        "pm.test('isolated', () => { pm.expect(typeof require).to.equal('undefined'); pm.expect(typeof fetch).to.equal('undefined'); pm.expect(typeof std).to.equal('undefined'); });",
    ))
    .unwrap();
    assert!(out.tests[0].passed, "{:?}", out.tests[0].error);
}

#[test]
fn infinite_loops_are_stopped() {
    let err = run(&input(ScriptKind::Test, "while (true) {}")).unwrap_err();
    assert!(matches!(err, AppError::Script(ref m) if m.contains("batas waktu")));
}

/// Answers every request with its own method, URL, headers and body, for inspection.
struct EchoHost;

impl ScriptHost for EchoHost {
    fn send(&self, request: HostRequest) -> Result<ScriptResponse, String> {
        if request.url.contains("down") {
            return Err("Tidak bisa terhubung ke server".into());
        }
        let body = serde_json::json!({
            "method": request.method, "url": request.url, "headers": request.headers, "body": request.body
        });
        Ok(ScriptResponse {
            code: 200,
            status: "OK".into(),
            headers: vec![],
            body: body.to_string(),
            response_time: 5,
        })
    }
}

#[test]
fn send_request_uses_the_host_and_fills_variables() {
    let script = r#"
        const res = pm.sendRequest({
            url: "{{base}}/login",
            method: "post",
            header: [{ key: "X-Trace", value: "{{base}}" }],
            body: { mode: "raw", raw: JSON.stringify({ user: "rani" }), options: { raw: { language: "json" } } },
        });
        const echo = res.json();
        pm.environment.set("method", echo.method);
        pm.environment.set("url", echo.url);
        pm.environment.set("type", echo.headers.find(([k]) => k === "Content-Type")[1]);
        pm.sendRequest("https://down.test", (err, r) => pm.environment.set("failed", String(err.message) + "|" + r));
    "#;
    let out = run_with_host(
        &input(ScriptKind::PreRequest, script),
        Some(Arc::new(EchoHost)),
    )
    .unwrap();
    assert_eq!(out.error, None, "{:?}", out.logs);
    assert_eq!(out.env["method"], "POST");
    assert_eq!(out.env["url"], "https://api.test/login");
    assert_eq!(out.env["type"], "application/json");
    assert_eq!(out.env["failed"], "Tidak bisa terhubung ke server|null");
    assert_eq!(
        out.logs[0].message,
        "pm.sendRequest POST https://api.test/login"
    );
}

#[test]
fn send_request_is_limited_and_unavailable_without_host() {
    let looped = "for (let i = 0; i < 25; i++) pm.sendRequest('https://api.test/x');";
    let out = run_with_host(
        &input(ScriptKind::PreRequest, looped),
        Some(Arc::new(EchoHost)),
    )
    .unwrap();
    assert!(out.error.unwrap().contains("dibatasi 20 kali"));
    let out = run(&input(
        ScriptKind::PreRequest,
        "pm.sendRequest('https://api.test')",
    ))
    .unwrap();
    assert!(out.error.unwrap().contains("tidak tersedia"));
}

#[test]
fn globals_can_be_read_and_changed() {
    let out = run(&input(
        ScriptKind::PreRequest,
        r#"pm.globals.set("seen", pm.globals.get("app") + "!"); pm.globals.unset("old");"#,
    ))
    .unwrap();
    assert_eq!(out.error, None);
    assert_eq!(out.glob.get("seen").map(String::as_str), Some("zorp!"));
    assert_eq!(out.glob_unset, ["old"]);
}
