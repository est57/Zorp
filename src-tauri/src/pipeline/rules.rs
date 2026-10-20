use std::collections::{BTreeSet, HashMap};

use super::context::RequestContext;
use crate::db;
use crate::error::AppResult;
use crate::models::request::RequestSpec;
use crate::models::response::ResponseData;
use crate::models::rules::{Assertion, VarPhase, VarTarget};
use crate::models::script::{CapturedVar, ScriptReport};
use crate::services::rules::{ResponseView, as_text, assert_all};
use crate::state::AppState;

/// Adds the request's "before request" variables to the script locals.
pub fn pre_request(
    ctx: &RequestContext,
    spec: &RequestSpec,
    mut locals: HashMap<String, String>,
) -> HashMap<String, String> {
    for rule in spec
        .vars
        .iter()
        .filter(|r| r.enabled && r.phase == VarPhase::Pre && !r.name.trim().is_empty())
    {
        let value = ctx
            .variables(&locals)
            .resolve(&rule.expr, &mut BTreeSet::new());
        locals.insert(rule.name.trim().to_string(), value);
    }
    locals
}

/// Runs assertions and stores "after response" variables. Problems are reported, never fatal.
pub fn after_response(
    state: &AppState,
    ctx: &mut RequestContext,
    spec: &RequestSpec,
    response: &ResponseData,
    locals: &HashMap<String, String>,
    report: &mut ScriptReport,
) {
    let view = ResponseView::new(response);
    let vars = ctx.variables(locals);
    let assertions: Vec<Assertion> = spec
        .asserts
        .iter()
        .map(|a| Assertion {
            value: vars.resolve(&a.value, &mut BTreeSet::new()),
            ..a.clone()
        })
        .collect();
    report.tests.extend(assert_all(&view, &assertions));

    let (mut env, mut coll) = (HashMap::new(), HashMap::new());
    for rule in spec
        .vars
        .iter()
        .filter(|r| r.enabled && r.phase == VarPhase::Post && !r.name.trim().is_empty())
    {
        let value = view.get(&rule.expr).map(|v| as_text(&v));
        report.captured.push(CapturedVar {
            name: rule.name.trim().to_string(),
            value: value.clone(),
        });
        match value {
            Some(value) => {
                let target = if rule.target == VarTarget::Environment {
                    &mut env
                } else {
                    &mut coll
                };
                target.insert(rule.name.trim().to_string(), value);
            }
            None => report.errors.push(format!(
                "Vars {}: {} tidak ditemukan di respons",
                rule.name.trim(),
                rule.expr.trim()
            )),
        }
    }
    if env.is_empty() && coll.is_empty() {
        return;
    }
    if let Err(e) = store(state, ctx, &env, &coll, report) {
        report.errors.push(format!("Vars: {e}"));
    }
}

fn store(
    state: &AppState,
    ctx: &mut RequestContext,
    env: &HashMap<String, String>,
    coll: &HashMap<String, String>,
    report: &mut ScriptReport,
) -> AppResult<()> {
    {
        let conn = state.db()?;
        if !env.is_empty() {
            if ctx.has_environment {
                db::variables::apply_env_changes(
                    &conn,
                    state.workspace_id(),
                    state.secrets.as_ref(),
                    env,
                    &[],
                )?;
            } else {
                report
                    .errors
                    .push("Vars: pilih environment untuk menyimpan variabel ke environment".into());
            }
        }
        if !coll.is_empty() {
            match ctx.collection_id {
                Some(id) => db::variables::apply_collection_changes(&conn, id, coll, &[])?,
                None => report
                    .errors
                    .push("Vars: simpan request ke koleksi untuk memakai variabel koleksi".into()),
            }
        }
    }
    ctx.reload(state)?;
    report.variables_changed = true;
    Ok(())
}
