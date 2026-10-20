//! Mirrors a workspace to a folder of YAML files that can be kept in Git.
pub mod files;
pub mod format;
pub mod parse;
pub mod render;
mod request_file;
pub mod slug;

#[cfg(test)]
mod tests;

use crate::models::collection::{Collection, Folder, SavedRequest};
use crate::models::environment::{Environment, Variable};
use crate::models::example::ResponseExample;

/// The shared part of a workspace: everything stored in the folder.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub name: String,
    pub collections: Vec<CollectionEntry>,
    pub folders: Vec<Folder>,
    pub requests: Vec<SavedRequest>,
    pub examples: Vec<ResponseExample>,
    /// Secret values are never part of a snapshot.
    pub environments: Vec<Environment>,
    /// Workspace-wide variables; secret values are left out here as well.
    pub globals: Vec<Variable>,
}

#[derive(Debug, Clone)]
pub struct CollectionEntry {
    pub collection: Collection,
    pub variables: Vec<(String, String)>,
}

pub const ROOT_FILE: &str = "zorp.yaml";
pub const COLLECTIONS_DIR: &str = "collections";
pub const ENVIRONMENTS_DIR: &str = "environments";
pub const COLLECTION_FILE: &str = "collection.yaml";
pub const FOLDER_FILE: &str = "folder.yaml";
pub const REQUEST_SUFFIX: &str = ".request.yaml";
pub const GLOBAL_FILE: &str = "_global.yaml";
