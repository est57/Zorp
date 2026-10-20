//! Evaluates `res.*` expressions, assertions and post-response variables against a response.
use serde_json::Value;

use crate::models::response::{ResponseBody, ResponseData};
use crate::models::rules::{AssertOp, Assertion};
use crate::models::script::TestResult;

/// Response values that expressions can read.
pub struct ResponseView<'a> {
    response: &'a ResponseData,
    body: Option<Value>,
}

impl<'a> ResponseView<'a> {
    pub fn new(response: &'a ResponseData) -> Self {
        let text = match &response.body {
            ResponseBody::Text { text } => Some(text.as_str()),
            ResponseBody::TooLarge { preview, .. } => Some(preview.as_str()),
            ResponseBody::Base64 { .. } => None,
        };
        let body =
            text.map(|t| serde_json::from_str(t).unwrap_or_else(|_| Value::String(t.to_string())));
        Self { response, body }
    }

    /// Reads `res.status`, `res.statusText`, `res.responseTime`, `res.headers['name']` or
    /// `res.body.path[0]["key"]`. Returns `None` when the path does not exist.
    pub fn get(&self, expr: &str) -> Option<Value> {
        let rest = expr.trim().strip_prefix("res")?;
        let mut steps = path(rest)?.into_iter();
        let first = steps.next()?;
        let mut value = match first.as_str() {
            "status" => Value::from(self.response.status),
            "statusText" => Value::from(self.response.status_text.clone()),
            "responseTime" => Value::from(self.response.timing.total_ms),
            "headers" => {
                let name = steps.next()?;
                let found = self
                    .response
                    .headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(&name))?;
                return steps.next().is_none().then(|| Value::from(found.1.clone()));
            }
            "body" => self.body.clone()?,
            _ => return None,
        };
        for step in steps {
            value = match value {
                Value::Object(mut map) => map.remove(&step)?,
                Value::Array(mut list) => {
                    let index: usize = step.parse().ok()?;
                    (index < list.len()).then(|| list.swap_remove(index))?
                }
                _ => return None,
            };
        }
        Some(value)
    }
}

/// Splits `.a.b[0]["c d"]` into `a`, `b`, `0`, `c d`.
fn path(text: &str) -> Option<Vec<String>> {
    let mut steps = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '.' => {
                let mut name = String::new();
                while let Some(&n) = chars.peek() {
                    if n == '.' || n == '[' {
                        break;
                    }
                    name.push(n);
                    chars.next();
                }
                steps.push(name);
            }
            '[' => {
                let quote = chars.peek().copied().filter(|q| *q == '\'' || *q == '"');
                if quote.is_some() {
                    chars.next();
                }
                let mut name = String::new();
                for n in chars.by_ref() {
                    if Some(n) == quote || (quote.is_none() && n == ']') {
                        break;
                    }
                    name.push(n);
                }
                if quote.is_some() && chars.next() != Some(']') {
                    return None;
                }
                steps.push(name);
            }
            c if c.is_whitespace() => {}
            _ => return None,
        }
    }
    (!steps.is_empty() && steps.iter().all(|s| !s.is_empty())).then_some(steps)
}

/// Text form of a value, as stored into a variable.
pub fn as_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// The expected value typed in the table: numbers, booleans and null keep their type,
/// quoted text loses its quotes.
fn expected(text: &str) -> Value {
    let t = text.trim();
    if let Some(inner) = t.strip_prefix('"').and_then(|x| x.strip_suffix('"')) {
        return Value::from(inner);
    }
    serde_json::from_str(t).unwrap_or_else(|_| Value::from(t))
}

fn number(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub fn op_label(op: AssertOp) -> &'static str {
    match op {
        AssertOp::Eq => "sama dengan",
        AssertOp::Neq => "tidak sama dengan",
        AssertOp::Gt => "lebih dari",
        AssertOp::Gte => "minimal",
        AssertOp::Lt => "kurang dari",
        AssertOp::Lte => "maksimal",
        AssertOp::Contains => "berisi",
        AssertOp::NotContains => "tidak berisi",
        AssertOp::Matches => "cocok regex",
        AssertOp::Exists => "ada",
        AssertOp::NotExists => "tidak ada",
        AssertOp::IsType => "bertipe",
    }
}

fn check(actual: Option<&Value>, op: AssertOp, value: &str) -> Result<(), String> {
    let want = expected(value);
    let got = || actual.map(as_text).unwrap_or_else(|| "tidak ada".into());
    let fail = || Err(format!("diterima {}", got()));
    let Some(actual) = actual else {
        return if op == AssertOp::NotExists {
            Ok(())
        } else {
            fail()
        };
    };
    let ok = match op {
        AssertOp::Exists => true,
        AssertOp::NotExists => false,
        AssertOp::Eq | AssertOp::Neq => {
            let same = actual == &want
                || number(actual)
                    .zip(number(&want))
                    .is_some_and(|(a, b)| a == b)
                || as_text(actual) == as_text(&want);
            same == (op == AssertOp::Eq)
        }
        AssertOp::Gt | AssertOp::Gte | AssertOp::Lt | AssertOp::Lte => {
            let (Some(a), Some(b)) = (number(actual), number(&want)) else {
                return Err(format!("{} bukan angka", got()));
            };
            match op {
                AssertOp::Gt => a > b,
                AssertOp::Gte => a >= b,
                AssertOp::Lt => a < b,
                _ => a <= b,
            }
        }
        AssertOp::Contains | AssertOp::NotContains => {
            let found = match actual {
                Value::Array(list) => list
                    .iter()
                    .any(|v| v == &want || as_text(v) == as_text(&want)),
                Value::Object(map) => map.contains_key(&as_text(&want)),
                other => as_text(other).contains(&as_text(&want)),
            };
            found == (op == AssertOp::Contains)
        }
        AssertOp::Matches => {
            let re = regex_lite::Regex::new(value.trim())
                .map_err(|e| format!("regex tidak valid: {e}"))?;
            re.is_match(&as_text(actual))
        }
        AssertOp::IsType => type_name(actual) == value.trim().to_ascii_lowercase(),
    };
    if ok { Ok(()) } else { fail() }
}

/// Runs enabled assertions; `value` should already have its variables filled in.
pub fn assert_all(view: &ResponseView, assertions: &[Assertion]) -> Vec<TestResult> {
    assertions
        .iter()
        .filter(|a| a.enabled && !a.expr.trim().is_empty())
        .map(|a| {
            let shown = if matches!(a.op, AssertOp::Exists | AssertOp::NotExists) {
                String::new()
            } else {
                format!(" {}", a.value.trim())
            };
            let name = format!("{} {}{shown}", a.expr.trim(), op_label(a.op));
            match check(view.get(&a.expr).as_ref(), a.op, &a.value) {
                Ok(()) => TestResult {
                    name,
                    passed: true,
                    error: None,
                },
                Err(error) => TestResult {
                    name,
                    passed: false,
                    error: Some(error),
                },
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "rules_tests.rs"]
mod tests;
