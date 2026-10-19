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
        "auth": { "type": "oauth2" },
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
fn rejects_non_postman_json() {
    assert!(parse("{\"hello\": 1}").is_err());
    assert!(parse("not json").is_err());
}
