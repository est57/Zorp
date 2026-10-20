use rusqlite::Connection;
use uuid::Uuid;

use super::{folders, requests};
use crate::error::AppResult;
use crate::models::collection::{Folder, SavedRequest};
use crate::models::realtime::RequestMode;

/// HTTP requests under a collection or folder in sidebar order: sub-folders first (depth first),
/// then requests. WebSocket and SSE requests are long-lived connections, so runs skip them.
pub fn ordered_requests(
    conn: &Connection,
    workspace_id: Uuid,
    collection_id: Uuid,
    folder_id: Option<Uuid>,
) -> AppResult<Vec<SavedRequest>> {
    let all_folders = folders::list_folders(conn, workspace_id)?;
    let all_requests = requests::list(conn, workspace_id)?;
    let mut out = Vec::new();
    collect(
        &all_folders,
        &all_requests,
        collection_id,
        folder_id,
        &mut out,
    );
    Ok(out)
}

fn collect(
    folders: &[Folder],
    requests: &[SavedRequest],
    collection_id: Uuid,
    parent: Option<Uuid>,
    out: &mut Vec<SavedRequest>,
) {
    let mut children: Vec<&Folder> = folders
        .iter()
        .filter(|f| f.collection_id == collection_id && f.parent_id == parent)
        .collect();
    children.sort_by(|a, b| a.sort_order.total_cmp(&b.sort_order));
    for folder in children {
        collect(folders, requests, collection_id, Some(folder.id), out);
    }
    let mut items: Vec<&SavedRequest> = requests
        .iter()
        .filter(|r| r.collection_id == collection_id && r.folder_id == parent)
        .filter(|r| r.spec.mode == RequestMode::Http)
        .collect();
    items.sort_by(|a, b| a.sort_order.total_cmp(&b.sort_order));
    out.extend(items.into_iter().cloned());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{collections, open_in_memory, workspace};
    use crate::models::request::{Auth, Body, HttpMethod, RequestSettings, RequestSpec};

    fn spec(name: &str) -> RequestSpec {
        RequestSpec {
            docs: String::new(),
            id: Uuid::now_v7(),
            name: name.into(),
            method: HttpMethod::Get,
            url: "/".into(),
            params: vec![],
            headers: vec![],
            body: Body::None,
            auth: Auth::None,
            settings: RequestSettings::default(),
            mode: Default::default(),
            scripts: Default::default(),
        }
    }

    #[test]
    fn orders_folders_depth_first_then_requests() {
        let conn = open_in_memory().unwrap();
        let ws = workspace::current(&conn).unwrap().id;
        let col = collections::create_collection(&conn, ws, "API").unwrap();
        let auth = folders::create_folder(&conn, col.id, None, "Auth").unwrap();
        let nested = folders::create_folder(&conn, col.id, Some(auth.id), "Nested").unwrap();
        requests::save(&conn, spec("root"), col.id, None).unwrap();
        requests::save(&conn, spec("login"), col.id, Some(auth.id)).unwrap();
        requests::save(&conn, spec("deep"), col.id, Some(nested.id)).unwrap();
        let socket = RequestSpec {
            mode: RequestMode::Websocket,
            ..spec("socket")
        };
        requests::save(&conn, socket, col.id, None).unwrap();

        let names: Vec<String> = ordered_requests(&conn, ws, col.id, None)
            .unwrap()
            .into_iter()
            .map(|r| r.spec.name)
            .collect();
        assert_eq!(names, ["deep", "login", "root"]);

        let only_auth = ordered_requests(&conn, ws, col.id, Some(auth.id)).unwrap();
        assert_eq!(only_auth.len(), 2);
    }
}
