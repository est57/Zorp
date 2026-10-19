pub mod callback;
pub mod pkce;
pub mod token_endpoint;
pub mod token_store;

use std::collections::BTreeSet;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::oneshot;

use crate::db::now_ms;
use crate::error::{AppError, AppResult};
use crate::models::oauth::{OAuth2Config, OAuthGrant};
use crate::services::request_builder::parse_url;
use crate::services::resolver::Variables;
use crate::services::secrets::SecretStore;
use token_store::StoredToken;

const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

pub fn resolve_config(config: &OAuth2Config, vars: &Variables) -> AppResult<OAuth2Config> {
    let mut missing = BTreeSet::new();
    let mut r = |text: &str| vars.resolve(text.trim(), &mut missing);
    let resolved = OAuth2Config {
        auth_url: r(&config.auth_url),
        token_url: r(&config.token_url),
        client_id: r(&config.client_id),
        client_secret: r(&config.client_secret),
        scope: r(&config.scope),
        ..config.clone()
    };
    if !missing.is_empty() {
        return Err(AppError::UnresolvedVariables(missing.into_iter().collect()));
    }
    if resolved.client_id.is_empty() {
        return Err(AppError::Validation("Client ID wajib diisi".into()));
    }
    Ok(resolved)
}

pub fn authorization_url(
    config: &OAuth2Config,
    redirect_uri: &str,
    state: &str,
    challenge: Option<&str>,
) -> AppResult<String> {
    let mut url = parse_url(&config.auth_url)?;
    {
        let mut query = url.query_pairs_mut();
        query
            .append_pair("response_type", "code")
            .append_pair("client_id", &config.client_id)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("state", state);
        if !config.scope.is_empty() {
            query.append_pair("scope", &config.scope);
        }
        if let Some(challenge) = challenge {
            query
                .append_pair("code_challenge", challenge)
                .append_pair("code_challenge_method", "S256");
        }
    }
    Ok(url.into())
}

/// Runs the configured grant. `open_browser` receives the login URL for the authorization code flow.
pub async fn fetch_token(
    client: &Client,
    config: &OAuth2Config,
    open_browser: impl FnOnce(&str) -> AppResult<()>,
    cancelled: oneshot::Receiver<()>,
) -> AppResult<StoredToken> {
    if config.grant == OAuthGrant::ClientCredentials {
        return token_endpoint::client_credentials(client, config).await;
    }
    let callback = callback::bind(config.callback_port).await?;
    let state = pkce::random_token(16)?;
    let verifier = if config.use_pkce {
        Some(pkce::random_token(32)?)
    } else {
        None
    };
    let challenge = verifier.as_deref().map(pkce::challenge);
    open_browser(&authorization_url(
        config,
        &callback.redirect_uri,
        &state,
        challenge.as_deref(),
    )?)?;

    let code = tokio::select! {
        result = tokio::time::timeout(LOGIN_TIMEOUT, callback.wait_for_code(&state)) => {
            result.map_err(|_| AppError::OAuth("Login tidak selesai dalam 5 menit".into()))??
        }
        _ = cancelled => return Err(AppError::Cancelled),
    };
    token_endpoint::exchange_code(
        client,
        config,
        &code,
        &callback.redirect_uri,
        verifier.as_deref(),
    )
    .await
}

/// Returns a valid access token, refreshing or re-requesting it when it has expired.
pub async fn access_token(
    store: &dyn SecretStore,
    client: &Client,
    config: &OAuth2Config,
) -> AppResult<String> {
    let Some(token) = token_store::load(store, &config.token_key)? else {
        if config.grant == OAuthGrant::ClientCredentials {
            let fresh = token_endpoint::client_credentials(client, config).await?;
            token_store::save(store, &config.token_key, &fresh)?;
            return Ok(fresh.access_token);
        }
        return Err(AppError::OAuth(
            "Token OAuth belum diambil. Buka tab Auth lalu klik Ambil token.".into(),
        ));
    };
    if !token.is_expired(now_ms()) {
        return Ok(token.access_token);
    }
    let fresh = match (&token.refresh_token, config.grant) {
        (Some(refresh), _) => token_endpoint::refresh(client, config, refresh).await?,
        (None, OAuthGrant::ClientCredentials) => {
            token_endpoint::client_credentials(client, config).await?
        }
        (None, OAuthGrant::AuthorizationCode) => {
            return Err(AppError::OAuth(
                "Token OAuth kedaluwarsa. Ambil token baru di tab Auth.".into(),
            ));
        }
    };
    token_store::save(store, &config.token_key, &fresh)?;
    Ok(fresh.access_token)
}

#[cfg(test)]
mod tests;
