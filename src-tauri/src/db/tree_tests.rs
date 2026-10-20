use uuid::Uuid;

use super::{collections, folders, open_in_memory, requests, workspace};
use crate::models::collection::{MoveTarget, NodeKind};
use crate::models::request::{Auth, Body, HttpMethod, RequestSettings, RequestSpec};

fn spec(name: &str) -> RequestSpec {
    RequestSpec {
        id: Uuid::now_v7(),
        name: name.into(),
        method: HttpMethod::Get,
        url: "/".into(),
        params: vec![],
        headers: vec![],
        body: Body::None,
        auth: Auth::Inherit,
        settings: RequestSettings::default(),
        mode: Default::default(),
        scripts: Default::default(),
    }
}

fn target(collection_id: Uuid, folder_id: Option<Uuid>) -> MoveTarget {
    MoveTarget {
        collection_id,
        folder_id,
        sort_order: 1.0,
    }
}

#[test]
fn folder_cannot_move_into_itself() {
    let conn = open_in_memory().unwrap();
    let ws = workspace::current(&conn).unwrap().id;
    let col = collections::create_collection(&conn, ws, "API").unwrap();
    let parent = folders::create_folder(&conn, col.id, None, "A").unwrap();
    let child = folders::create_folder(&conn, col.id, Some(parent.id), "B").unwrap();
    assert!(folders::move_folder(&conn, parent.id, &target(col.id, Some(child.id))).is_err());
}

#[test]
fn folder_moves_to_another_collection_with_contents() {
    let conn = open_in_memory().unwrap();
    let ws = workspace::current(&conn).unwrap().id;
    let a = collections::create_collection(&conn, ws, "A").unwrap();
    let b = collections::create_collection(&conn, ws, "B").unwrap();
    let outer = folders::create_folder(&conn, a.id, None, "Outer").unwrap();
    let inner = folders::create_folder(&conn, a.id, Some(outer.id), "Inner").unwrap();
    requests::save(&conn, spec("Deep"), a.id, Some(inner.id)).unwrap();

    folders::move_folder(&conn, outer.id, &target(b.id, None)).unwrap();

    assert!(
        folders::list_folders(&conn, ws)
            .unwrap()
            .iter()
            .all(|f| f.collection_id == b.id)
    );
    assert_eq!(requests::list(&conn, ws).unwrap()[0].collection_id, b.id);
}

#[test]
fn folder_parent_must_be_in_target_collection() {
    let conn = open_in_memory().unwrap();
    let ws = workspace::current(&conn).unwrap().id;
    let a = collections::create_collection(&conn, ws, "A").unwrap();
    let b = collections::create_collection(&conn, ws, "B").unwrap();
    let in_a = folders::create_folder(&conn, a.id, None, "InA").unwrap();
    let moving = folders::create_folder(&conn, a.id, None, "Moving").unwrap();
    assert!(folders::move_folder(&conn, moving.id, &target(b.id, Some(in_a.id))).is_err());
}

#[test]
fn inherited_auth_skips_inherit_and_uses_collection() {
    let conn = open_in_memory().unwrap();
    let ws = workspace::current(&conn).unwrap().id;
    let col = collections::create_collection(&conn, ws, "API").unwrap();
    let folder = folders::create_folder(&conn, col.id, None, "Users").unwrap();
    let bearer = Auth::Bearer {
        token: "{{token}}".into(),
    };
    collections::set_auth(&conn, NodeKind::Collection, col.id, &bearer).unwrap();
    collections::set_auth(&conn, NodeKind::Folder, folder.id, &Auth::Inherit).unwrap();
    let auth = collections::inherited_auth(&conn, col.id, Some(folder.id)).unwrap();
    assert!(matches!(auth, Some(Auth::Bearer { .. })));

    collections::set_auth(&conn, NodeKind::Folder, folder.id, &Auth::None).unwrap();
    let auth = collections::inherited_auth(&conn, col.id, Some(folder.id)).unwrap();
    assert!(matches!(auth, Some(Auth::None)));
}

#[test]
fn newer_schema_is_refused_with_a_clear_message() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("zorp.db");
    drop(super::open(&path).unwrap());
    rusqlite::Connection::open(&path)
        .unwrap()
        .pragma_update(None, "user_version", 99)
        .unwrap();
    let err = super::open(&path).unwrap_err().to_string();
    assert!(err.contains("versi Zorp yang lebih baru"), "{err}");
}
