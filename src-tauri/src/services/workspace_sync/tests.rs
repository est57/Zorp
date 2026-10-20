use std::fs;

use uuid::Uuid;

use super::*;
use crate::models::collection::{Collection, Folder, SavedRequest};
use crate::models::environment::{Environment, Variable};
use crate::models::example::ResponseExample;
use crate::models::request::{Auth, Body, HttpMethod, KeyValue, RequestSpec};

fn request(name: &str, collection: Uuid, folder: Option<Uuid>, order: f64) -> SavedRequest {
    SavedRequest {
        collection_id: collection,
        folder_id: folder,
        sort_order: order,
        spec: RequestSpec {
            docs: String::new(),
            id: Uuid::now_v7(),
            name: name.into(),
            method: HttpMethod::Post,
            url: "{{base}}/users".into(),
            params: vec![],
            headers: vec![KeyValue {
                key: "Accept".into(),
                value: "application/json".into(),
                enabled: true,
                description: String::new(),
            }],
            body: Body::Json {
                text: "{\n  \"name\": \"Ada\"\n}".into(),
            },
            auth: Auth::Inherit,
            settings: Default::default(),
            scripts: crate::models::script::Scripts {
                pre_request: String::new(),
                test: "pm.test('ok', () => {\n  pm.response.to.have.status(201);\n});".into(),
            },
            mode: Default::default(),
        },
    }
}

fn sample() -> Snapshot {
    let collection = Collection {
        scripts: Default::default(),
        docs: String::new(),
        id: Uuid::now_v7(),
        name: "Users API".into(),
        auth: Some(Auth::Bearer {
            token: "{{token}}".into(),
        }),
        sort_order: 1.0,
    };
    let folder = Folder {
        scripts: Default::default(),
        docs: String::new(),
        id: Uuid::now_v7(),
        collection_id: collection.id,
        parent_id: None,
        name: "Admin".into(),
        auth: None,
        sort_order: 1.0,
    };
    let nested = Folder {
        scripts: Default::default(),
        docs: String::new(),
        id: Uuid::now_v7(),
        collection_id: collection.id,
        parent_id: Some(folder.id),
        name: "Admin".into(),
        auth: None,
        sort_order: 1.0,
    };
    let mut documented = request("Create user", collection.id, None, 1.0);
    documented.spec.docs = "# Create user\nMembuat user baru.".into();
    let example = ResponseExample {
        id: Uuid::now_v7(),
        request_id: documented.spec.id,
        name: "201 Dibuat".into(),
        status: 201,
        status_text: "Created".into(),
        headers: vec![("content-type".into(), "application/json".into())],
        body: "{\n  \"id\": 41\n}".into(),
        sort_order: 1.0,
    };
    Snapshot {
        name: "Tim".into(),
        examples: vec![example],
        requests: vec![
            documented,
            request("Create user", collection.id, None, 2.0),
            request("Delete user", collection.id, Some(nested.id), 1.0),
        ],
        collections: vec![CollectionEntry {
            collection,
            variables: vec![("base".into(), "https://api.example.com".into())],
        }],
        folders: vec![folder, nested],
        environments: vec![Environment {
            id: Uuid::now_v7(),
            name: "Staging".into(),
            variables: vec![
                Variable {
                    key: "base".into(),
                    value: "https://staging".into(),
                    enabled: true,
                    secret: false,
                    has_value: false,
                },
                Variable {
                    key: "token".into(),
                    value: "do-not-write".into(),
                    enabled: true,
                    secret: true,
                    has_value: true,
                },
            ],
            is_active: true,
        }],
    }
}

#[test]
fn folder_round_trip_keeps_structure_and_omits_secrets() {
    let dir = tempfile::tempdir().unwrap();
    let snapshot = sample();
    let files = render::render(&snapshot).unwrap();
    assert!(files.contains_key("collections/users-api/admin/admin/delete-user.request.yaml"));
    assert_eq!(
        files.keys().filter(|k| k.contains("create-user")).count(),
        2
    );
    assert!(files.values().all(|text| !text.contains("do-not-write")));
    assert!(files.values().any(|text| text.contains("test: |-")));
    assert!(
        files
            .values()
            .any(|text| text.contains("examples:") && text.contains("docs: |-"))
    );

    assert_eq!(files::write(dir.path(), &files).unwrap(), files.len());
    assert_eq!(files::write(dir.path(), &files).unwrap(), 0);
    let back = parse::read(dir.path()).unwrap();
    assert_eq!(render::render(&back).unwrap(), files);
    assert_eq!(
        back.folders
            .iter()
            .filter(|f| f.parent_id.is_some())
            .count(),
        1
    );
    assert_eq!(back.environments[0].variables[1].value, "");
    assert_eq!(back.examples, snapshot.examples);
}

#[test]
fn removes_stale_files_but_keeps_foreign_ones() {
    let dir = tempfile::tempdir().unwrap();
    let mut snapshot = sample();
    files::write(dir.path(), &render::render(&snapshot).unwrap()).unwrap();
    fs::write(dir.path().join("README.md"), "notes").unwrap();
    fs::write(
        dir.path().join("collections/users-api/admin/notes.txt"),
        "keep",
    )
    .unwrap();

    snapshot.requests.retain(|r| r.spec.name != "Delete user");
    snapshot.folders.retain(|f| f.parent_id.is_none());
    files::write(dir.path(), &render::render(&snapshot).unwrap()).unwrap();
    assert!(
        !dir.path()
            .join("collections/users-api/admin/admin")
            .exists()
    );
    assert!(
        dir.path()
            .join("collections/users-api/admin/notes.txt")
            .exists()
    );
    assert!(dir.path().join("README.md").exists());
}

#[test]
fn invalid_files_are_reported_instead_of_read() {
    let dir = tempfile::tempdir().unwrap();
    assert!(parse::read(dir.path()).is_err());
    let snapshot = sample();
    let files = render::render(&snapshot).unwrap();
    files::write(dir.path(), &files).unwrap();
    let first = files
        .keys()
        .find(|k| k.contains("create-user"))
        .unwrap()
        .clone();
    let copy = dir.path().join("collections/users-api/copy.request.yaml");
    fs::copy(dir.path().join(&first), &copy).unwrap();
    fs::write(dir.path().join("environments/broken.yaml"), "id: [unclosed").unwrap();
    let conflict = "<<<<<<< HEAD\nname: A\n=======\nname: B\n>>>>>>> main\n";
    fs::write(dir.path().join("environments/merge.yaml"), conflict).unwrap();
    let problems = parse::read(dir.path()).unwrap_err().0;
    assert_eq!(problems.len(), 3, "{problems:?}");
    assert!(
        problems
            .iter()
            .any(|p| p == "environments/merge.yaml: berisi konflik Git yang belum diselesaikan")
    );
    assert!(
        problems
            .iter()
            .any(|p| p.contains("dipakai lebih dari sekali"))
    );
    assert!(
        problems
            .iter()
            .any(|p| p.starts_with("environments/broken.yaml"))
    );
}
