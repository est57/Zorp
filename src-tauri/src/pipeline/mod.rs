mod context;
mod digest;
mod script_host;
mod script_io;
mod scripts;

use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::request::{Auth, RequestSpec};
use crate::models::response::{ProgressPhase, ResponseData};
use crate::models::script::ScriptReport;
use crate::services::client_pool::ClientKey;
use crate::services::resolver::resolve_spec;
use crate::services::{oauth, request_builder, signing};
use crate::state::AppState;
pub use context::RequestContext;

/// Runs one request end to end: pre-request script, send, then test script.
/// Returns the request as actually sent (after script changes) alongside the result.
pub async fn execute(
    state: &AppState,
    spec: RequestSpec,
    request_id: Uuid,
    progress: impl Fn(ProgressPhase, u64),
) -> (RequestSpec, AppResult<ResponseData>) {
    execute_with_data(state, spec, Vec::new(), request_id, progress).await
}

/// Like [`execute`], with runner iteration values layered over the environment.
pub async fn execute_with_data(
    state: &AppState,
    mut spec: RequestSpec,
    data: Vec<(String, String)>,
    request_id: Uuid,
    progress: impl Fn(ProgressPhase, u64),
) -> (RequestSpec, AppResult<ResponseData>) {
    let mut ctx = match RequestContext::load(state, &spec) {
        Ok(ctx) => ctx,
        Err(e) => return (spec, Err(e)),
    };
    ctx.data = data;
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
        Auth::Jwt(config) => signing::jwt::to_auth(&config, signing::unix_now())?,
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
        result = digest::execute(builder, &prepared.auth, &state.spill_dir, progress) => result,
        _ = cancelled => Err(AppError::Cancelled),
    };
    state.finish(request_id);
    state.persist_cookies()?;
    result
}

#[cfg(test)]
mod auth_tests;
#[cfg(test)]
mod chain_tests;
#[cfg(test)]
mod tests;
