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
