//! Auth types whose credentials are computed per request: JWT, AWS Signature v4,
//! OAuth 1.0 and HTTP Digest.
pub mod aws;
pub mod digest;
pub mod jwt;
pub mod oauth1;

#[cfg(test)]
mod tests;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{AppError, AppResult};

/// Percent-encoding from RFC 3986, as required by AWS and OAuth 1.0 signatures.
pub fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Random hex string, used for nonces.
pub fn nonce() -> AppResult<String> {
    let mut buf = [0u8; 16];
    getrandom::fill(&mut buf).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(hex::encode(buf))
}

pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

/// UTC calendar date and time for a Unix timestamp: (year, month, day, hour, minute, second).
pub fn utc_parts(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400) as u32;
    // Civil-from-days conversion by Howard Hinnant.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day, rem / 3600, rem % 3600 / 60, rem % 60)
}
