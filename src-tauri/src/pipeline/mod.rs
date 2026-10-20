mod context;
mod scripts;

use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::request::{Auth, RequestSpec};
use crate::models::response::{ProgressPhase, ResponseData};
use crate::models::script::ScriptReport;
use crate::services::client_pool::ClientKey;
use crate::services::resolver::resolve_spec;
use crate::services::{http_engine, oauth, request_builder};
use crate::state::AppState;
pub use context::RequestContext;

/// Runs one request end to end: pre-request script, send, then test script.
/// Returns the request as actually sent (after script changes) alongside the result.
pub async fn execute(
    state: &AppState,
    mut spec: RequestSpec,
    request_id: Uuid,
    progress: impl Fn(ProgressPhase, u64),
) -> (RequestSpec, AppResult<ResponseData>) {
    let mut ctx = match RequestContext::load(state, &spec) {
        Ok(ctx) => ctx,
        Err(e) => return (spec, Err(e)),
    };
    let mut report = ScriptReport::default();
    let locals = match scripts::pre_request(state, &mut ctx, &mut spec, &mut report).await {
        Ok(locals) => locals,
        Err(e) => return (spec, Err(e)),
    };
    let result = send(state, &spec, &ctx, &locals, request_id, progress).await;
    let result = match result {
        Ok(mut response) => {
            scripts::test(state, &mut ctx, &spec, &response, &mut report).await;
            response.script = report;
            Ok(response)
        }
        Err(e) => Err(e),
    };
    (spec, result)
}

/// A request with variables substituted, auth resolved (OAuth tokens fetched) and a matching client.
pub struct Prepared {
    pub spec: RequestSpec,
    pub auth: Auth,
    pub client: reqwest::Client,
}

pub async fn prepare(
    state: &AppState,
    spec: &RequestSpec,
    ctx: &RequestContext,
    locals: &std::collections::HashMap<String, String>,
) -> AppResult<Prepared> {
    let vars = ctx.variables(locals);
    let auth = ctx.effective_auth(state, spec)?;
    let (resolved, auth) = resolve_spec(spec, &auth, &vars)?;
    let client = state.client(ClientKey {
        verify_tls: spec.settings.verify_tls,
        follow_redirects: spec.settings.follow_redirects,
        use_cookies: spec.settings.use_cookies,
        identity: state.pool.identity_for(&resolved.url)?,
    })?;
    let auth = match auth {
        Auth::OAuth2(config) => {
            let config = oauth::resolve_config(&config, &vars)?;
            let token_client = state.client(ClientKey::plain())?;
            Auth::Bearer {
                token: oauth::access_token(state.secrets.as_ref(), &token_client, &config).await?,
            }
        }
        other => other,
    };
    Ok(Prepared {
        spec: resolved,
        auth,
        client,
    })
}

async fn send(
    state: &AppState,
    spec: &RequestSpec,
    ctx: &RequestContext,
    locals: &std::collections::HashMap<String, String>,
    request_id: Uuid,
    progress: impl Fn(ProgressPhase, u64),
) -> AppResult<ResponseData> {
    let prepared = prepare(state, spec, ctx, locals).await?;
    let builder = request_builder::build(&prepared.client, &prepared.spec, &prepared.auth)?;
    let cancelled = state.register(request_id)?;
    let result = tokio::select! {
        result = http_engine::execute(builder, &state.spill_dir, progress) => result,
        _ = cancelled => Err(AppError::Cancelled),
    };
    state.finish(request_id);
    state.persist_cookies()?;
    result
}

#[cfg(test)]
mod tests;
