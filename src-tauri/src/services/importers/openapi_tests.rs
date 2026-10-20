use serde_json::Value;

use super::openapi::{convert, load, summarize};
use super::{ImportedCollection, ImportedNode};
use crate::models::openapi::{GroupBy, OpenApiOptions};
use crate::models::request::{Auth, Body, HttpMethod, MultipartPart, RequestSpec};

use super::openapi_fixtures::{OAS3, SWAGGER2};

fn options(group_by: GroupBy) -> OpenApiOptions {
    OpenApiOptions {
        group_by,
        base_url: "https://eu.petstore.dev/api/v3".into(),
        example_bodies: true,
        collection_auth: true,
    }
}

fn requests(collection: &ImportedCollection) -> Vec<(Option<String>, &RequestSpec)> {
    let mut out = Vec::new();
    for node in &collection.items {
        match node {
            ImportedNode::Folder { name, children, .. } => {
                for child in children {
                    if let ImportedNode::Request(spec) = child {
                        out.push((Some(name.clone()), spec.as_ref()));
                    }
                }
            }
            ImportedNode::Request(spec) => out.push((None, spec.as_ref())),
        }
    }
    out
}

fn find<'a>(all: &'a [(Option<String>, &'a RequestSpec)], name: &str) -> &'a RequestSpec {
    all.iter()
        .find(|(_, r)| r.name == name)
        .unwrap_or_else(|| panic!("{name} tidak ada"))
        .1
}

#[test]
fn summarizes_before_import() {
    let summary = summarize(&load(OAS3).unwrap()).unwrap();
    assert_eq!(
        (
            summary.title.as_str(),
            summary.version.as_str(),
            summary.format.as_str()
        ),
        ("Petstore", "1.0.27", "OpenAPI 3.0")
    );
    assert_eq!(summary.endpoints, 6);
    assert_eq!(summary.servers, ["https://eu.petstore.dev/api/v3"]);
    assert_eq!(summary.auth, ["bearerAuth", "apiKey"]);
    let groups: Vec<_> = summary
        .groups
        .iter()
        .map(|g| (g.name.as_str(), g.count))
        .collect();
    assert_eq!(groups, [("pet", 4), ("store", 1), ("Tanpa tag", 1)]);
}

#[test]
fn converts_openapi3_operations() {
    let collection = convert(&load(OAS3).unwrap(), &options(GroupBy::Tag)).unwrap();
    assert_eq!(collection.name, "Petstore");
    assert!(matches!(&collection.auth, Some(Auth::Bearer { token }) if token == "{{token}}"));
    assert_eq!(
        collection.variables[0],
        ("baseUrl".into(), "https://eu.petstore.dev/api/v3".into())
    );
    assert!(collection.variables.contains(&("petId".into(), "7".into())));

    let all = requests(&collection);
    let folders: Vec<_> = collection
        .items
        .iter()
        .filter_map(|n| match n {
            ImportedNode::Folder { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(folders, ["store", "pet"]);
    assert!(
        all.iter()
            .any(|(g, r)| g.is_none() && r.name == "GET /health")
    );

    let get = find(&all, "Find pet by ID");
    assert_eq!(get.url, "{{baseUrl}}/pet/{{petId}}?lang=id");
    assert_eq!(
        get.params
            .iter()
            .map(|p| (p.key.as_str(), p.enabled))
            .collect::<Vec<_>>(),
        [("fields", false), ("lang", true)]
    );
    assert_eq!(get.headers[0].value, "00000000-0000-0000-0000-000000000000");
    assert!(matches!(get.auth, Auth::Inherit));
    assert!(matches!(find(&all, "deletePet").auth, Auth::None));
    assert!(
        matches!(&find(&all, "Inventory").auth, Auth::ApiKey { key, .. } if key == "X-Api-Key")
    );

    let Body::Json { text } = &find(&all, "Add a pet").body else {
        panic!("body JSON")
    };
    let body: Value = serde_json::from_str(text).unwrap();
    assert_eq!(body["name"], "Rex");
    assert_eq!(body["status"], "available");
    assert!(body.get("id").is_none(), "readOnly dilewati");
    assert_eq!(body["tags"][0]["label"], "string");
    assert!(body["parent"].is_null(), "skema rekursif berhenti");

    let Body::Multipart { parts } = &find(&all, "Upload image").body else {
        panic!("multipart")
    };
    assert!(matches!(&parts[0], MultipartPart::File { key, .. } if key == "file"));
    assert!(matches!(&parts[1], MultipartPart::Text { value, .. } if value == "front"));
}

#[test]
fn groups_by_path_and_can_skip_examples() {
    let mut opts = options(GroupBy::Path);
    opts.example_bodies = false;
    opts.collection_auth = false;
    let collection = convert(&load(OAS3).unwrap(), &opts).unwrap();
    assert!(collection.auth.is_none());
    let all = requests(&collection);
    assert_eq!(
        all.iter()
            .find(|(_, r)| r.name == "Inventory")
            .unwrap()
            .0
            .as_deref(),
        Some("store")
    );
    assert!(matches!(&find(&all, "Add a pet").body, Body::Json { text } if text.is_empty()));
}

#[test]
fn converts_swagger2() {
    let root = load(SWAGGER2).unwrap();
    assert_eq!(
        summarize(&root).unwrap().servers,
        ["http://legacy.example.com/v1"]
    );
    let collection = convert(&root, &options(GroupBy::Tag)).unwrap();
    let all = requests(&collection);
    let create = find(&all, "Create user");
    assert_eq!(create.method, HttpMethod::Post);
    let Body::Json { text } = &create.body else {
        panic!("body JSON")
    };
    assert_eq!(
        serde_json::from_str::<Value>(text).unwrap()["email"],
        "user@example.com"
    );
    let Auth::OAuth2(config) = &create.auth else {
        panic!("OAuth 2.0")
    };
    assert_eq!(
        (
            config.token_url.as_str(),
            config.scope.as_str(),
            config.client_secret.as_str()
        ),
        (
            "https://auth.example.com/token",
            "users:write",
            "{{clientSecret}}"
        )
    );
    let Body::FormUrlEncoded { fields } = &find(&all, "Login").body else {
        panic!("form")
    };
    assert_eq!(
        fields
            .iter()
            .map(|f| (f.key.as_str(), f.value.as_str(), f.enabled))
            .collect::<Vec<_>>(),
        [
            ("email", "user@example.com", true),
            ("remember", "false", false)
        ]
    );
}

#[test]
fn rejects_documents_that_are_not_openapi() {
    assert!(load("{\"info\": {}}").is_err());
    assert!(load("openapi: [").is_err());
    assert!(load("swagger: \"1.2\"").is_err());
}
