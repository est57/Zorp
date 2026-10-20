use std::collections::HashMap;

use md5::Md5;
use sha2::{Digest, Sha256};

/// Parses `Digest realm="x", nonce="y", qop="auth"` into lowercase keys and values.
fn parse(challenge: &str) -> Option<HashMap<String, String>> {
    let rest = challenge
        .trim()
        .strip_prefix("Digest")
        .or_else(|| challenge.trim().strip_prefix("digest"))?;
    let mut map = HashMap::new();
    let mut chars = rest.trim().chars().peekable();
    loop {
        while chars.peek().is_some_and(|c| *c == ',' || c.is_whitespace()) {
            chars.next();
        }
        let key: String = chars.by_ref().take_while(|c| *c != '=').collect();
        if key.is_empty() {
            break;
        }
        let value: String = if chars.peek() == Some(&'"') {
            chars.next();
            let mut v = String::new();
            while let Some(c) = chars.next() {
                match c {
                    '\\' => v.extend(chars.next()),
                    '"' => break,
                    other => v.push(other),
                }
            }
            v
        } else {
            chars
                .by_ref()
                .take_while(|c| *c != ',')
                .collect::<String>()
                .trim()
                .to_string()
        };
        map.insert(key.trim().to_ascii_lowercase(), value);
    }
    Some(map)
}

/// The Digest challenge among `WWW-Authenticate` header values, if any.
pub fn challenge<'a>(values: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    values
        .into_iter()
        .find(|v| v.trim_start().to_ascii_lowercase().starts_with("digest"))
}

fn hash(algorithm: &str, text: &str) -> String {
    if algorithm.to_ascii_uppercase().starts_with("SHA-256") {
        hex::encode(Sha256::digest(text.as_bytes()))
    } else {
        hex::encode(Md5::digest(text.as_bytes()))
    }
}

/// `Authorization` header value answering a Digest challenge (RFC 7616, qop `auth`).
pub fn authorization(
    challenge: &str,
    method: &str,
    uri: &str,
    user: &str,
    pass: &str,
    cnonce: &str,
) -> Option<String> {
    let c = parse(challenge)?;
    let realm = c.get("realm").cloned().unwrap_or_default();
    let nonce = c.get("nonce")?.clone();
    let algorithm = c.get("algorithm").cloned().unwrap_or_else(|| "MD5".into());
    let nc = "00000001";
    let mut ha1 = hash(&algorithm, &format!("{user}:{realm}:{pass}"));
    if algorithm.to_ascii_lowercase().ends_with("-sess") {
        ha1 = hash(&algorithm, &format!("{ha1}:{nonce}:{cnonce}"));
    }
    let ha2 = hash(&algorithm, &format!("{method}:{uri}"));
    let qop = c
        .get("qop")
        .map(|q| q.split(',').map(str::trim).any(|q| q == "auth"));
    let response = match qop {
        Some(true) => hash(
            &algorithm,
            &format!("{ha1}:{nonce}:{nc}:{cnonce}:auth:{ha2}"),
        ),
        Some(false) => return None,
        None => hash(&algorithm, &format!("{ha1}:{nonce}:{ha2}")),
    };
    let mut header = format!(
        "Digest username=\"{}\", realm=\"{realm}\", nonce=\"{nonce}\", uri=\"{uri}\", algorithm={algorithm}, response=\"{response}\"",
        user.replace('\\', "\\\\").replace('"', "\\\""),
    );
    if qop == Some(true) {
        header.push_str(&format!(", qop=auth, nc={nc}, cnonce=\"{cnonce}\""));
    }
    if let Some(opaque) = c.get("opaque") {
        header.push_str(&format!(", opaque=\"{opaque}\""));
    }
    Some(header)
}
