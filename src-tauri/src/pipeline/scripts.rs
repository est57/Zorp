use std::collections::HashMap;

use super::context::RequestContext;
use super::script_io::{script_request, script_response};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::request::{KeyValue, RequestSpec};
use crate::models::response::ResponseData;
use crate::models::script::{ScriptReport, Scripts};
use std::sync::Arc;

use super::script_host::PipelineHost;
use crate::services::scripting::{
    self, ScriptHost, ScriptInput, ScriptKind, ScriptOutput, ScriptRequest,
};
use crate::state::AppState;

async fn run(state: &AppState, input: ScriptInput) -> AppResult<ScriptOutput> {
    let host = PipelineHost::new(state).map(|h| Arc::new(h) as Arc<dyn ScriptHost>);
    tauri::async_runtime::spawn_blocking(move || scripting::run_with_host(&input, host))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
}

/// Persists variable changes made by a script and reloads the context when needed.
fn apply_changes(
    state: &AppState,
    ctx: &mut RequestContext,
    out: &ScriptOutput,
    report: &mut ScriptReport,
) -> AppResult<()> {
    if !out.changes_variables() {
        return Ok(());
    }
    let conn = state.db()?;
    db::variables::apply_env_changes(
        &conn,
        state.workspace_id(),
        state.secrets.as_ref(),
        &out.env,
        &out.env_unset,
    )?;
    if let Some(collection_id) = ctx.collection_id {
        db::variables::apply_collection_changes(&conn, collection_id, &out.coll, &out.coll_unset)?;
    }
    drop(conn);
    ctx.reload(state)?;
    report.variables_changed = true;
    Ok(())
}

fn input(
    kind: ScriptKind,
    script: &str,
    spec: &RequestSpec,
    ctx: &RequestContext,
    locals: &HashMap<String, String>,
    response: Option<&ResponseData>,
) -> ScriptInput {
    ScriptInput {
        kind,
        script: script.to_string(),
        request_name: spec.name.clone(),
        request: script_request(spec),
        response: response.map(script_response),
        variables: ctx.variable_map(locals),
        environment: ctx.environment.iter().cloned().collect(),
        collection_variables: ctx.collection.iter().cloned().collect(),
        has_environment: ctx.has_environment,
    }
}

/// Collection and folder scripts, outermost first, followed by the request's own.
fn steps(
    state: &AppState,
    ctx: &RequestContext,
    spec: &RequestSpec,
) -> AppResult<Vec<(String, Scripts)>> {
    let mut steps = match ctx.collection_id {
        Some(collection) => db::node_settings::chain(&*state.db()?, collection, ctx.folder_id)?,
        None => Vec::new(),
    };
    steps.push(("request".into(), spec.scripts.clone()));
    Ok(steps)
}

/// Applies the method, URL and headers a pre-request script left on `pm.request`.
fn take_request(spec: &mut RequestSpec, request: ScriptRequest) {
    if let Ok(method) = serde_json::from_value(serde_json::Value::String(request.method.clone())) {
        spec.method = method;
    }
    spec.url = request.url;
    let disabled = spec.headers.iter().filter(|h| !h.enabled).cloned();
    spec.headers = request
        .headers
        .into_iter()
        .map(|(key, value)| KeyValue {
            key,
            value,
            enabled: true,
            description: String::new(),
        })
        .chain(disabled)
        .collect();
}

/// Runs the pre-request scripts of the collection, its folders and the request, in that
/// order, and returns the script-local variables. A thrown error stops the request.
pub async fn pre_request(
    state: &AppState,
    ctx: &mut RequestContext,
    spec: &mut RequestSpec,
    report: &mut ScriptReport,
) -> AppResult<HashMap<String, String>> {
    let mut locals = HashMap::new();
    for (label, scripts) in steps(state, ctx, spec)? {
        if scripts.pre_request.trim().is_empty() {
            continue;
        }
        let out = run(
            state,
            input(
                ScriptKind::PreRequest,
                &scripts.pre_request,
                spec,
                ctx,
                &locals,
                None,
            ),
        )
        .await?;
        apply_changes(state, ctx, &out, report)?;
        report.logs.extend(out.logs.iter().cloned());
        report.tests.extend(out.tests.iter().cloned());
        if let Some(error) = out.error {
            let place = if label == "request" {
                String::new()
            } else {
                format!(" {label}")
            };
            return Err(AppError::Script(format!("pre-request{place}: {error}")));
        }
        take_request(spec, out.request);
        locals.extend(out.locals);
    }
    Ok(locals)
}

/// Runs the test scripts in the same order; failures are reported instead of failing the request.
pub async fn test(
    state: &AppState,
    ctx: &mut RequestContext,
    spec: &RequestSpec,
    response: &ResponseData,
    report: &mut ScriptReport,
) {
    let steps = match steps(state, ctx, spec) {
        Ok(steps) => steps,
        Err(e) => return report.errors.push(e.to_string()),
    };
    for (label, scripts) in steps {
        if scripts.test.trim().is_empty() {
            continue;
        }
        match run(
            state,
            input(
                ScriptKind::Test,
                &scripts.test,
                spec,
                ctx,
                &HashMap::new(),
                Some(response),
            ),
        )
        .await
        {
            Ok(out) => {
                if let Err(e) = apply_changes(state, ctx, &out, report) {
                    report.errors.push(e.to_string());
                }
                report.tests.extend(out.tests);
                report.logs.extend(out.logs);
                if let Some(error) = out.error {
                    let place = if label == "request" {
                        String::new()
                    } else {
                        format!(" {label}")
                    };
                    report.errors.push(format!("test{place}: {error}"));
                }
            }
            Err(e) => report.errors.push(e.to_string()),
        }
    }
}
