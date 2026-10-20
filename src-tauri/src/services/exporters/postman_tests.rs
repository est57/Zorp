use uuid::Uuid;

use super::*;
use crate::models::oauth::{ClientAuth, OAuth2Config, OAuthGrant};
use crate::models::request::{Auth, Body, HttpMethod, KeyValue, MultipartPart, RequestSettings};
use crate::models::script::Scripts;
use crate::services::importers::{ImportedNode, postman as importer};

fn kv(key: &str, value: &str, enabled: bool) -> KeyValue {
    KeyValue {
        key: key.into(),
        value: value.into(),
        enabled,
        description: String::new(),
    }
}

fn saved(collection: Uuid, folder: Option<Uuid>, order: f64, spec: RequestSpec) -> SavedRequest {
    SavedRequest {
        collection_id: collection,
        folder_id: folder,
        sort_order: order,
        spec,
    }
}

fn spec(name: &str, method: HttpMethod, body: Body, auth: Auth) -> RequestSpec {
    RequestSpec {
        docs: String::new(),
        id: Uuid::now_v7(),
        name: name.into(),
        method,
        url: "{{base}}/users?page=2".into(),
        params: vec![kv("page", "2", true), kv("debug", "1", false)],
        headers: vec![kv("Accept", "application/json", true)],
        body,
        auth,
        settings: RequestSettings::default(),
        scripts: Scripts {
            pre_request: String::new(),
            test: "pm.test('ok', () => {\n  pm.response.to.have.status(200);\n});".into(),
        },
        mode: Default::default(),
    }
}

#[test]
fn exported_collection_imports_back_with_the_same_content() {
    let collection = Collection {
        id: Uuid::now_v7(),
        name: "Shop".into(),
        auth: Some(Auth::Bearer {
            token: "{{token}}".into(),
        }),
        sort_order: 1.0,
    };
    let folder = Folder {
        id: Uuid::now_v7(),
        collection_id: collection.id,
        parent_id: None,
        name: "Users".into(),
        auth: None,
        sort_order: 1.0,
    };
    let oauth = Auth::OAuth2(OAuth2Config {
        grant: OAuthGrant::AuthorizationCode,
        auth_url: "https://id.test/auth".into(),
        token_url: "https://id.test/token".into(),
        client_id: "zorp".into(),
        client_secret: String::new(),
        scope: "read".into(),
        use_pkce: false,
        client_auth: ClientAuth::Body,
        callback_port: 0,
        token_key: "k".into(),
    });
    let mut socket = spec("Socket", HttpMethod::Get, Body::None, Auth::Inherit);
    socket.mode = crate::models::realtime::RequestMode::Websocket;
    let requests = vec![
        saved(
            collection.id,
            Some(folder.id),
            1.0,
            spec("List", HttpMethod::Get, Body::None, Auth::Inherit),
        ),
        saved(
            collection.id,
            Some(folder.id),
            2.0,
            spec(
                "Create",
                HttpMethod::Post,
                Body::Json {
                    text: "{\"a\":1}".into(),
                },
                oauth,
            ),
        ),
        saved(
            collection.id,
            None,
            1.0,
            spec(
                "Upload",
                HttpMethod::Put,
                Body::Multipart {
                    parts: vec![
                        MultipartPart::File {
                            key: "f".into(),
                            path: "/tmp/a.png".into(),
                            enabled: true,
                        },
                        MultipartPart::Text {
                            key: "note".into(),
                            value: "hi".into(),
                            enabled: false,
                        },
                    ],
                },
                Auth::None,
            ),
        ),
        saved(
            collection.id,
            None,
            2.0,
            spec(
                "Query",
                HttpMethod::Post,
                Body::Graphql {
                    query: "{ a }".into(),
                    variables: "{\"x\":1}".into(),
                },
                Auth::Inherit,
            ),
        ),
        saved(collection.id, None, 3.0, socket),
    ];
    let variables = vec![("base".to_string(), "https://shop.test".to_string())];
    let data = CollectionData {
        collection: &collection,
        variables: &variables,
        folders: &[folder],
        requests: &requests,
    };

    let exported = export(&data);
    assert_eq!(
        (exported.requests, exported.skipped.clone()),
        (4, vec!["Socket".to_string()])
    );

    let back = importer::parse(&exported.json.to_string()).unwrap();
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    assert_eq!(back.name, "Shop");
    assert_eq!(back.variables, variables);
    assert!(matches!(back.auth, Some(Auth::Bearer { ref token }) if token == "{{token}}"));

    let ImportedNode::Folder { name, children, .. } = &back.items[0] else {
        panic!("folder first")
    };
    assert_eq!(name, "Users");
    let ImportedNode::Request(list) = &children[0] else {
        panic!()
    };
    assert_eq!(list.url, "{{base}}/users?page=2");
    assert!(!list.params[1].enabled);
    assert!(matches!(list.auth, Auth::Inherit));
    assert!(list.scripts.test.contains("to.have.status(200)"));
    let ImportedNode::Request(create) = &children[1] else {
        panic!()
    };
    assert!(
        matches!(&create.auth, Auth::OAuth2(c) if !c.use_pkce && c.client_auth == ClientAuth::Body)
    );

    let ImportedNode::Request(upload) = &back.items[1] else {
        panic!()
    };
    assert!(matches!(&upload.auth, Auth::None));
    assert!(
        matches!(&upload.body, Body::Multipart { parts } if matches!(&parts[1], MultipartPart::Text { enabled: false, .. }))
    );
    let ImportedNode::Request(query) = &back.items[2] else {
        panic!()
    };
    assert!(
        matches!(&query.body, Body::Graphql { query, variables } if query == "{ a }" && variables == "{\"x\":1}")
    );
}
