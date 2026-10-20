use reqwest::header::AUTHORIZATION;

use super::*;
use crate::models::auth::{AwsSigV4Config, JwtAlgorithm, JwtConfig, OAuth1Config, OAuth1Method};
use crate::models::request::{ApiKeyLocation, Auth};

fn header(req: &reqwest::Request) -> String {
    req.headers()[AUTHORIZATION].to_str().unwrap().to_string()
}

#[test]
fn digest_matches_rfc2617_example() {
    let challenge = r#"Digest realm="testrealm@host.com", qop="auth,auth-int", nonce="dcd98b7102dd2f0e8b11d0f600bfb0c093", opaque="5ccc069c403ebaf9f0171e9517f40e41""#;
    assert_eq!(
        digest::challenge(["Basic realm=x", challenge].into_iter()),
        Some(challenge)
    );
    let value = digest::authorization(
        challenge,
        "GET",
        "/dir/index.html",
        "Mufasa",
        "Circle Of Life",
        "0a4f113b",
    )
    .unwrap();
    assert!(
        value.contains(r#"response="6629fae49393a05397450978507c4ef1""#),
        "{value}"
    );
    assert!(value.contains(r#"opaque="5ccc069c403ebaf9f0171e9517f40e41""#));
}

#[test]
fn aws_matches_sigv4_test_suite() {
    // "get-vanilla" from the AWS Signature Version 4 test suite.
    let mut req = reqwest::Client::new()
        .get("https://example.amazonaws.com/")
        .build()
        .unwrap();
    let config = AwsSigV4Config {
        access_key: "AKIDEXAMPLE".into(),
        secret_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY".into(),
        session_token: String::new(),
        region: "us-east-1".into(),
        service: "service".into(),
    };
    aws::sign(&mut req, &config, 1_440_938_160).unwrap();
    assert_eq!(req.headers()["x-amz-date"], "20150830T123600Z");
    assert_eq!(
        header(&req),
        "AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/20150830/us-east-1/service/aws4_request, SignedHeaders=host;x-amz-date, Signature=5fa00fa31553b73ebf1942676e86291e8372ff2a2260956d9b8aae1d763fbf31"
    );
}

#[test]
fn oauth1_matches_twitter_example() {
    let mut req = reqwest::Client::new()
        .post("https://api.twitter.com/1/statuses/update.json?include_entities=true")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("status=Hello%20Ladies%20%2B%20Gentlemen%2C%20a%20signed%20OAuth%20request%21")
        .build()
        .unwrap();
    let config = OAuth1Config {
        consumer_key: "xvz1evFS4wEEPTGEFPHBog".into(),
        consumer_secret: "kAcSOqF21Fu85e7zjz7ZN2U4ZRhfV3WpwPAoE3Z7kBw".into(),
        token: "370773112-GmHxMAgYyLbNEtIKZeRNFsMKPR9EyMZeS9weJAEb".into(),
        token_secret: "LswwdoUaIvS8ltyTt5jkRh4J50vUPVVHtR2YPi5kE".into(),
        signature_method: OAuth1Method::HmacSha1,
        realm: String::new(),
    };
    oauth1::sign(
        &mut req,
        &config,
        "kYjzVBB8Y0ZFabxSWbWovY3uYSQ2pTgmZeNu2VS4cg",
        1_318_622_958,
    )
    .unwrap();
    assert!(
        header(&req).contains(r#"oauth_signature="tnnArxj06cWHq44gCs1OSKk%2FjLY%3D""#),
        "{}",
        header(&req)
    );
}

#[test]
fn jwt_signs_and_attaches_token() {
    let config = JwtConfig {
        algorithm: JwtAlgorithm::HS256,
        secret: "your-256-bit-secret".into(),
        secret_base64: false,
        payload: r#"{"sub":"1234567890","name":"John Doe","iat":1516239022}"#.into(),
        headers: String::new(),
        location: ApiKeyLocation::Header,
        param: "Authorization".into(),
        prefix: "Bearer".into(),
    };
    let expected = jwt::token(&config, 0).unwrap();
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.required_spec_claims.clear();
    validation.validate_exp = false;
    let decoded = jsonwebtoken::decode::<serde_json::Value>(
        &expected,
        &jsonwebtoken::DecodingKey::from_secret(b"your-256-bit-secret"),
        &validation,
    )
    .unwrap();
    assert_eq!(decoded.claims["name"], "John Doe");
    assert_eq!(
        decoded.claims["iat"], 1516239022,
        "iat yang ada tidak ditimpa"
    );
    let Auth::ApiKey { key, value, .. } = jwt::to_auth(&config, 0).unwrap() else {
        panic!()
    };
    assert_eq!(
        (key.as_str(), value),
        ("Authorization", format!("Bearer {expected}"))
    );

    let query = JwtConfig {
        location: ApiKeyLocation::Query,
        param: String::new(),
        ..config.clone()
    };
    assert!(jwt::to_auth(&query, 0).is_err());
    let bad = JwtConfig {
        payload: "[1]".into(),
        ..config
    };
    assert!(jwt::token(&bad, 0).is_err());
    let no_iat = JwtConfig {
        payload: "{}".into(),
        ..bad
    };
    let token = jwt::token(&no_iat, 42).unwrap();
    let claims = token.split('.').nth(1).unwrap();
    use base64::Engine;
    let claims = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(claims)
        .unwrap();
    assert_eq!(String::from_utf8(claims).unwrap(), r#"{"iat":42}"#);
}

#[test]
fn utc_parts_and_encoding() {
    assert_eq!(utc_parts(0), (1970, 1, 1, 0, 0, 0));
    assert_eq!(utc_parts(1_709_208_000), (2024, 2, 29, 12, 0, 0));
    assert_eq!(encode("a b+c/~"), "a%20b%2Bc%2F~");
}
