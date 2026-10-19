use tauri::State;
use uuid::Uuid;

use crate::db::{collections, folders, requests};
use crate::error::{AppError, AppResult};
use crate::models::collection::{
    Collection, CollectionTree, Folder, MoveTarget, NodeKind, SavedRequest,
};
use crate::models::request::{Auth, RequestSpec};
use crate::state::AppState;

#[tauri::command]
pub fn list_tree(state: State<'_, AppState>) -> AppResult<CollectionTree> {
    let conn = state.db()?;
    Ok(CollectionTree {
        collections: collections::list_collections(&conn, state.workspace_id)?,
        folders: folders::list_folders(&conn, state.workspace_id)?,
        requests: requests::list(&conn, state.workspace_id)?,
    })
}

#[tauri::command]
pub fn create_collection(state: State<'_, AppState>, name: String) -> AppResult<Collection> {
    collections::create_collection(&*state.db()?, state.workspace_id, &name)
}

#[tauri::command]
pub fn create_folder(
    state: State<'_, AppState>,
    collection_id: Uuid,
    parent_id: Option<Uuid>,
    name: String,
) -> AppResult<Folder> {
    folders::create_folder(&*state.db()?, collection_id, parent_id, &name)
}

#[tauri::command]
pub fn save_request(
    state: State<'_, AppState>,
    spec: RequestSpec,
    collection_id: Uuid,
    folder_id: Option<Uuid>,
) -> AppResult<SavedRequest> {
    requests::save(&*state.db()?, spec, collection_id, folder_id)
}

#[tauri::command]
pub fn rename_node(
    state: State<'_, AppState>,
    kind: NodeKind,
    id: Uuid,
    name: String,
) -> AppResult<()> {
    let conn = state.db()?;
    match kind {
        NodeKind::Collection => collections::rename_collection(&conn, id, &name),
        NodeKind::Folder => folders::rename_folder(&conn, id, &name),
        NodeKind::Request => requests::rename(&conn, id, &name),
    }
}

#[tauri::command]
pub fn delete_node(state: State<'_, AppState>, kind: NodeKind, id: Uuid) -> AppResult<()> {
    let conn = state.db()?;
    match kind {
        NodeKind::Collection => collections::delete_collection(&conn, id),
        NodeKind::Folder => folders::delete_folder(&conn, id),
        NodeKind::Request => requests::delete(&conn, id),
    }
}

#[tauri::command]
pub fn move_node(
    state: State<'_, AppState>,
    kind: NodeKind,
    id: Uuid,
    target: MoveTarget,
) -> AppResult<()> {
    let conn = state.db()?;
    match kind {
        NodeKind::Request => requests::move_to(&conn, id, &target),
        NodeKind::Folder => folders::move_folder(&conn, id, &target),
        NodeKind::Collection => Err(AppError::Validation(
            "Koleksi tidak bisa dipindahkan".into(),
        )),
    }
}

#[tauri::command]
pub fn set_node_auth(
    state: State<'_, AppState>,
    kind: NodeKind,
    id: Uuid,
    auth: Auth,
) -> AppResult<()> {
    collections::set_auth(&*state.db()?, kind, id, &auth)
}
