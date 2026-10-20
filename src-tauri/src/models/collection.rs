use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::example::ExampleSummary;
use super::request::{Auth, RequestSpec};
use super::script::Scripts;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub auth: Option<Auth>,
    pub sort_order: f64,
    /// Run before and after every request inside, ahead of folder and request scripts.
    #[serde(default)]
    pub scripts: Scripts,
    #[serde(default)]
    pub docs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Folder {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub auth: Option<Auth>,
    pub sort_order: f64,
    #[serde(default)]
    pub scripts: Scripts,
    #[serde(default)]
    pub docs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SavedRequest {
    pub collection_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub sort_order: f64,
    pub spec: RequestSpec,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct CollectionTree {
    pub collections: Vec<Collection>,
    pub folders: Vec<Folder>,
    pub requests: Vec<SavedRequest>,
    pub examples: Vec<ExampleSummary>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum NodeKind {
    Collection,
    Folder,
    Request,
}

/// Target location for a moved folder or request.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MoveTarget {
    pub collection_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub sort_order: f64,
}
