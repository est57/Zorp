use std::path::Path;

use reqwest::RequestBuilder;
use reqwest::header::AUTHORIZATION;

use crate::error::{AppError, AppResult};
use crate::models::request::Auth;
use crate::models::response::{ProgressPhase, ResponseData};
use crate::services::http_engine;
use crate::services::signing::{digest, nonce};

/// Sends a request. With Digest auth, a 401 challenge is answered once with credentials;
/// the first attempt was rejected, so the server has not acted on it.
pub async fn execute(
    builder: RequestBuilder,
    auth: &Auth,
    spill_dir: &Path,
    progress: impl Fn(ProgressPhase, u64),
) -> AppResult<ResponseData> {
    let Auth::Digest { username, password } = auth else {
        return http_engine::execute(builder, spill_dir, progress).await;
    };
    let retry = builder.try_clone().ok_or_else(|| {
        AppError::Validation("Digest auth belum mendukung body multipart atau stream".into())
    })?;
    let first = http_engine::execute(builder, spill_dir, &progress).await?;
    let challenge = digest::challenge(
        first
            .headers
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case("www-authenticate"))
            .map(|(_, v)| v.as_str()),
    );
    let Some(challenge) = challenge.filter(|_| first.status == 401) else {
        return Ok(first);
    };
    let target = retry
        .try_clone()
        .and_then(|b| b.build().ok())
        .ok_or_else(|| {
            AppError::Validation("Request tidak bisa diulang untuk Digest auth".into())
        })?;
    let uri = match target.url().query() {
        Some(q) => format!("{}?{q}", target.url().path()),
        None => target.url().path().to_string(),
    };
    let value = digest::authorization(
        challenge,
        target.method().as_str(),
        &uri,
        username,
        password,
        &nonce()?,
    )
    .ok_or_else(|| {
        AppError::Validation(
            "Server meminta varian Digest yang belum didukung (qop auth-int)".into(),
        )
    })?;
    http_engine::execute(retry.header(AUTHORIZATION, value), spill_dir, progress).await
}
