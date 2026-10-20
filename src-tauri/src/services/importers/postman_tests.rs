use super::*;
use crate::models::request::Body;

const SAMPLE: &str = r#"{
  "info": { "name": "Shop API", "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json" },
  "auth": { "type": "bearer", "bearer": [{ "key": "token", "value": "{{token}}", "type": "string" }] },
  "variable": [{ "key": "base_url", "value": "https://shop.test" }, { "key": "off", "value": "x", "disabled": true }],
  "item": [
    {
      "name": "Products",
      "item": [
        {
          "name": "List products",
          "event": [{ "listen": "test", "script": { "exec": ["pm.test('ok', () => {", "  pm.response.to.have.status(200);", "});"] } }],
          "request": {
            "method": "GET",
            "header": [{ "key": "Accept", "value": "application/json" }],
            "url": { "raw": "{{base_url}}/products?page=1", "query": [{ "key": "page", "value": "1" }, { "key": "debug", "value": "1", "disabled": true }] }
          }
        },
        {
          "name": "Create product",
          "request": {
            "method": "post",
            "body": { "mode": "raw", "raw": "{\"name\":\"Mug\"}", "options": { "raw": { "language": "json" } } },
            "url": "{{base_url}}/products"
          }
        }
      ]
    },
    {
      "name": "Upload",
      "request": {
        "method": "PUT",
        "auth": { "type": "ntlm" },
        "body": { "mode": "formdata", "formdata": [{ "key": "file", "type": "file", "src": "/tmp/a.png" }, { "key": "note", "value": "hi", "type": "text" }] },
        "url": "{{base_url}}/upload"
      }
    },
    { "name": "Ping", "request": "{{base_url}}/ping" }
  ]
}"#;

#[test]
fn parses_folders_requests_auth_and_variables() {
    let parsed = parse(SAMPLE).unwrap();
    assert_eq!(parsed.name, "Shop API");
    assert!(matches!(parsed.auth, Some(Auth::Bearer { ref token }) if token == "{{token}}"));
    assert_eq!(
        parsed.variables,
        vec![("base_url".to_string(), "https://shop.test".to_string())]
    );
    assert_eq!(parsed.items.len(), 3);

    let ImportedNode::Folder { name, children, .. } = &parsed.items[0] else {
        panic!("expected folder")
    };
    assert_eq!(name, "Products");
    let ImportedNode::Request(list) = &children[0] else {
        panic!("expected request")
    };
    assert_eq!(list.url, "{{base_url}}/products?page=1");
    assert_eq!(list.params.len(), 2);
    assert!(!list.params[1].enabled);
    assert!(matches!(list.auth, Auth::Inherit));
    assert!(
        list.scripts
            .test
            .contains("pm.response.to.have.status(200);")
    );
    assert!(list.scripts.pre_request.is_empty());

    let ImportedNode::Request(create) = &children[1] else {
        panic!("expected request")
    };
    assert_eq!(create.method, HttpMethod::Post);
    assert!(matches!(create.body, Body::Json { ref text } if text.contains("Mug")));
}

#[test]
fn unsupported_auth_is_reported_and_string_requests_work() {
    let parsed = parse(SAMPLE).unwrap();
    let ImportedNode::Request(upload) = &parsed.items[1] else {
        panic!("expected request")
    };
    assert!(matches!(upload.auth, Auth::None));
    assert!(matches!(&upload.body, Body::Multipart { parts } if parts.len() == 2));
    assert_eq!(parsed.warnings.len(), 1);

    let ImportedNode::Request(ping) = &parsed.items[2] else {
        panic!("expected request")
    };
    assert_eq!(ping.url, "{{base_url}}/ping");
    assert_eq!(ping.method, HttpMethod::Get);
}

#[test]
fn maps_oauth2_client_credentials() {
    let json = r#"{"info":{"name":"O","schema":"https://schema.getpostman.com/json/collection/v2.1.0/collection.json"},
        "auth":{"type":"oauth2","oauth2":[{"key":"grant_type","value":"client_credentials"},
        {"key":"accessTokenUrl","value":"https://id.test/token"},{"key":"clientId","value":"abc"},
        {"key":"client_authentication","value":"body"}]},"item":[]}"#;
    let parsed = parse(json).unwrap();
    let Some(Auth::OAuth2(config)) = parsed.auth else {
        panic!("expected oauth2")
    };
    assert_eq!(
        config.grant,
        crate::models::oauth::OAuthGrant::ClientCredentials
    );
    assert_eq!(config.token_url, "https://id.test/token");
    assert_eq!(config.client_auth, crate::models::oauth::ClientAuth::Body);
}

#[test]
fn rejects_non_postman_json() {
    assert!(parse("{\"hello\": 1}").is_err());
    assert!(parse("not json").is_err());
}

#[test]
fn signed_auth_types_round_trip_through_postman() {
    use crate::models::auth::{AwsSigV4Config, OAuth1Config, OAuth1Method};
    use crate::services::exporters::postman::fields_auth;
    let originals = [
        Auth::Digest {
            username: "rani".into(),
            password: "{{pass}}".into(),
        },
        Auth::AwsSigV4(AwsSigV4Config {
            access_key: "{{aws_key}}".into(),
            secret_key: "{{aws_secret}}".into(),
            session_token: String::new(),
            region: "ap-southeast-1".into(),
            service: "execute-api".into(),
        }),
        Auth::OAuth1(OAuth1Config {
            consumer_key: "ck".into(),
            consumer_secret: "{{cs}}".into(),
            token: String::new(),
            token_secret: String::new(),
            signature_method: OAuth1Method::HmacSha256,
            realm: String::new(),
        }),
    ];
    for original in originals {
        let exported = fields_auth(&original).unwrap();
        let mut warnings = Vec::new();
        let back = super::parse_auth_value(&exported, &mut warnings).unwrap();
        assert_eq!(
            serde_json::to_value(&back).unwrap(),
            serde_json::to_value(&original).unwrap()
        );
        assert!(warnings.is_empty());
    }
}
