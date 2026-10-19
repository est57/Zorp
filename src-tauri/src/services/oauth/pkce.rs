use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

/// Random URL-safe string with `bytes` bytes of entropy.
pub fn random_token(bytes: usize) -> AppResult<String> {
    let mut buf = vec![0u8; bytes];
    getrandom::fill(&mut buf).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(buf))
}

/// S256 code challenge for a PKCE verifier (RFC 7636).
pub fn challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_is_base64url_sha256() {
        // SHA-256("abc") is the standard test vector ba7816bf...f20015ad.
        assert_eq!(
            challenge("abc"),
            "ungWv48Bz-pBQUDeXa4iI7ADYaOWF3qctBD_YfIAFa0"
        );
    }

    #[test]
    fn random_tokens_are_unique_and_long_enough() {
        let a = random_token(32).unwrap();
        assert!(a.len() >= 43);
        assert_ne!(a, random_token(32).unwrap());
    }
}
