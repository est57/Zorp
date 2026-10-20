use std::collections::HashMap;

use super::context::RequestContext;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::request::{Body, KeyValue, RequestSpec};
use crate::models::response::{ResponseBody, ResponseData};
use crate::models::script::ScriptReport;
use crate::services::scripting::{
    self, ScriptInput, ScriptKind, ScriptOutput, ScriptRequest, ScriptResponse,
};
use crate::state::AppState;

fn script_request(spec: &RequestSpec) -> ScriptRequest {
    ScriptRequest {
        method: format!("{:?}", spec.method).to_uppercase(),
        url: spec.url.clone(),
        headers: spec
            .headers
            .iter()
            .filter(|h| h.enabled && !h.key.is_empty())
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect(),
        body: match &spec.body {
            Body::Json { text } | Body::Raw { text, .. } => Some(text.clone()),
            Body::Graphql { query, variables } => {
                crate::services::request_builder::graphql_payload(query, variables)
                    .ok()
                    .map(|v| v.to_string())
            }
            _ => None,
        },
    }
}

fn script_response(response: &ResponseData) -> ScriptResponse {
    ScriptResponse {
        code: response.status,
        status: response.status_text.clone(),
        headers: response.headers.clone(),
        body: match &response.body {
            ResponseBody::Text { text } => text.clone(),
            ResponseBody::TooLarge { preview, .. } => preview.clone(),
            ResponseBody::Base64 { .. } => String::new(),
        },
        response_time: response.timing.total_ms,
    }
}

async fn run(input: ScriptInput) -> AppResult<ScriptOutput> {
    tauri::async_runtime::spawn_blocking(move || scripting::run(&input))
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
    *ctx = RequestContext::for_location(state, ctx.collection_id, ctx.folder_id)?;
    report.variables_changed = true;
    Ok(())
}

fn input(
    kind: ScriptKind,
    script: &str,
    spec: &RequestSpec,
    ctx: &RequestContext,
    response: Option<&ResponseData>,
) -> ScriptInput {
    ScriptInput {
        kind,
        script: script.to_string(),
        request_name: spec.name.clone(),
        request: script_request(spec),
        response: response.map(script_response),
        variables: ctx.variable_map(&HashMap::new()),
        environment: ctx.environment.iter().cloned().collect(),
        collection_variables: ctx.collection.iter().cloned().collect(),
        has_environment: ctx.has_environment,
    }
}

/// Runs the pre-request script and returns script-local variables. A thrown error stops the request.
pub async fn pre_request(
    state: &AppState,
    ctx: &mut RequestContext,
    spec: &mut RequestSpec,
    report: &mut ScriptReport,
) -> AppResult<HashMap<String, String>> {
    if spec.scripts.pre_request.trim().is_empty() {
        return Ok(HashMap::new());
    }
    let out = run(input(
        ScriptKind::PreRequest,
        &spec.scripts.pre_request,
        spec,
        ctx,
        None,
    ))
    .await?;
    apply_changes(state, ctx, &out, report)?;
    report.logs.extend(out.logs.iter().cloned());
    report.tests.extend(out.tests.iter().cloned());
    if let Some(error) = out.error {
        return Err(AppError::Script(format!("pre-request: {error}")));
    }
    if let Ok(method) =
        serde_json::from_value(serde_json::Value::String(out.request.method.clone()))
    {
        spec.method = method;
    }
    spec.url = out.request.url;
    let disabled = spec.headers.iter().filter(|h| !h.enabled).cloned();
    spec.headers = out
        .request
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
    Ok(out.locals)
}

/// Runs the test script; failures are reported instead of failing the request.
pub async fn test(
    state: &AppState,
    ctx: &mut RequestContext,
    spec: &RequestSpec,
    response: &ResponseData,
    report: &mut ScriptReport,
) {
    if spec.scripts.test.trim().is_empty() {
        return;
    }
    match run(input(
        ScriptKind::Test,
        &spec.scripts.test,
        spec,
        ctx,
        Some(response),
    ))
    .await
    {
        Ok(out) => {
            if let Err(e) = apply_changes(state, ctx, &out, report) {
                report.errors.push(e.to_string());
            }
            report.tests.extend(out.tests);
            report.logs.extend(out.logs);
            if let Some(error) = out.error {
                report.errors.push(format!("test: {error}"));
            }
        }
        Err(e) => report.errors.push(e.to_string()),
    }
}
