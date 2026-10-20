use super::*;
use crate::models::response::Timing;

fn response() -> ResponseData {
    ResponseData {
        status: 201,
        status_text: "Created".into(),
        headers: vec![("X-Request-Id".into(), "abc-1".into())],
        body: ResponseBody::Text {
            text: r#"{"token":"t1","user":{"id":42,"roles":["admin","staff"],"name":"Rani"},"list":[{"a b":1}]}"#.into(),
        },
        size_bytes: 10,
        timing: Timing { total_ms: 120, ..Default::default() },
        http_version: "HTTP/1.1".into(),
        url: "http://x".into(),
        script: Default::default(),
    }
}

fn a(expr: &str, op: AssertOp, value: &str) -> Assertion {
    Assertion {
        enabled: true,
        expr: expr.into(),
        op,
        value: value.into(),
    }
}

#[test]
fn reads_paths_from_the_response() {
    let r = response();
    let view = ResponseView::new(&r);
    assert_eq!(view.get("res.status"), Some(Value::from(201)));
    assert_eq!(
        view.get("res.headers['x-request-id']"),
        Some(Value::from("abc-1"))
    );
    assert_eq!(
        view.get("res.body.user.roles[1]"),
        Some(Value::from("staff"))
    );
    assert_eq!(view.get(r#"res.body.list[0]["a b"]"#), Some(Value::from(1)));
    assert_eq!(view.get("res.body.user.missing"), None);
    assert_eq!(view.get("body.token"), None);
}

#[test]
fn operators_compare_values() {
    let r = response();
    let view = ResponseView::new(&r);
    let results = assert_all(
        &view,
        &[
            a("res.status", AssertOp::Eq, "201"),
            a("res.body.user.name", AssertOp::Eq, "\"Rani\""),
            a("res.responseTime", AssertOp::Lt, "100"),
            a("res.body.user.roles", AssertOp::Contains, "admin"),
            a("res.body.token", AssertOp::Matches, "^t\\d$"),
            a("res.body.user", AssertOp::IsType, "object"),
            a("res.body.nothing", AssertOp::NotExists, ""),
            Assertion {
                enabled: false,
                ..a("res.status", AssertOp::Eq, "500")
            },
        ],
    );
    let passed: Vec<_> = results.iter().map(|t| t.passed).collect();
    assert_eq!(passed, [true, true, false, true, true, true, true]);
    assert_eq!(results[2].name, "res.responseTime kurang dari 100");
    assert_eq!(results[2].error.as_deref(), Some("diterima 120"));
}
