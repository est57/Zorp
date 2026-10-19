use reqwest::Client;
use serde_json::Value;

use super::token_store::StoredToken;
use crate::db::now_ms;
use crate::error::{AppError, AppResult};
use crate::models::oauth::{ClientAuth, OAuth2Config};
use crate::services::http_engine::classify;
use crate::services::request_builder::parse_url;

fn number(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn text(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn parse_token(body: &str) -> AppResult<StoredToken> {
    let json: Value = serde_json::from_str(body)
        .map_err(|_| AppError::OAuth("Respons token bukan JSON yang valid".into()))?;
    let access_token = text(json.get("access_token"))
        .ok_or_else(|| AppError::OAuth("Respons token tidak berisi access_token".into()))?;
    Ok(StoredToken {
        access_token,
        token_type: text(json.get("token_type")).unwrap_or_else(|| "Bearer".into()),
        refresh_token: text(json.get("refresh_token")),
        expires_at: number(json.get("expires_in")).map(|secs| now_ms() + secs * 1000),
        scope: text(json.get("scope")),
    })
}

fn rejection(status: u16, body: &str) -> AppError {
    let json: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let code = text(json.get("error")).unwrap_or_else(|| format!("HTTP {status}"));
    let detail = text(json.get("error_description"))
        .map(|d| format!(": {d}"))
        .unwrap_or_default();
    AppError::OAuth(format!("Server OAuth menolak permintaan ({code}){detail}"))
}

/// Posts a grant to the token endpoint, adding client authentication.
pub async fn request(
    client: &Client,
    config: &OAuth2Config,
    mut form: Vec<(&str, String)>,
) -> AppResult<StoredToken> {
    let url = parse_url(&config.token_url)?;
    let mut builder = client
        .post(url)
        .header(reqwest::header::ACCEPT, "application/json");
    // Public clients without a secret always identify themselves in the body.
    if config.client_auth == ClientAuth::Basic && !config.client_secret.is_empty() {
        builder = builder.basic_auth(&config.client_id, Some(&config.client_secret));
    } else {
        form.push(("client_id", config.client_id.clone()));
        if !config.client_secret.is_empty() {
            form.push(("client_secret", config.client_secret.clone()));
        }
    }
    let response = builder.form(&form).send().await.map_err(classify)?;
    let status = response.status();
    let body = response.text().await.map_err(classify)?;
    if !status.is_success() {
        return Err(rejection(status.as_u16(), &body));
    }
    parse_token(&body)
}

pub async fn client_credentials(client: &Client, config: &OAuth2Config) -> AppResult<StoredToken> {
    let mut form = vec![("grant_type", "client_credentials".to_string())];
    if !config.scope.is_empty() {
        form.push(("scope", config.scope.clone()));
    }
    request(client, config, form).await
}

pub async fn refresh(
    client: &Client,
    config: &OAuth2Config,
    refresh_token: &str,
) -> AppResult<StoredToken> {
    let form = vec![
        ("grant_type", "refresh_token".to_string()),
        ("refresh_token", refresh_token.to_string()),
    ];
    let mut token = request(client, config, form).await?;
    // Providers may omit the refresh token when it does not rotate.
    token
        .refresh_token
        .get_or_insert_with(|| refresh_token.to_string());
    Ok(token)
}

pub async fn exchange_code(
    client: &Client,
    config: &OAuth2Config,
    code: &str,
    redirect_uri: &str,
    verifier: Option<&str>,
) -> AppResult<StoredToken> {
    let mut form = vec![
        ("grant_type", "authorization_code".to_string()),
        ("code", code.to_string()),
        ("redirect_uri", redirect_uri.to_string()),
    ];
    if let Some(verifier) = verifier {
        form.push(("code_verifier", verifier.to_string()));
    }
    request(client, config, form).await
}
