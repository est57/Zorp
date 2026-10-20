use std::collections::BTreeMap;

use serde::Serialize;
use uuid::Uuid;

use super::format::{
    CollectionFile, EnvVariable, EnvironmentFile, FORMAT_VERSION, FolderFile, Pair, RequestFile,
    WorkspaceFile,
};
use super::slug::Names;
use super::*;
use crate::error::{AppError, AppResult};

/// File contents keyed by path relative to the workspace folder, using `/` separators.
pub type FileMap = BTreeMap<String, String>;

fn yaml<T: Serialize>(value: &T) -> AppResult<String> {
    serde_saphyr::to_string(value).map_err(|e| AppError::Internal(format!("YAML: {e}")))
}

fn by_order<T>(items: &mut [T], key: impl Fn(&T) -> (f64, Uuid)) {
    items.sort_by(|a, b| {
        let (a, b) = (key(a), key(b));
        a.0.total_cmp(&b.0).then(a.1.cmp(&b.1))
    });
}

/// Lays out a snapshot as files. Slug clashes are settled by sort order, then id.
pub fn render(snapshot: &Snapshot) -> AppResult<FileMap> {
    let mut files = FileMap::new();
    let root = WorkspaceFile {
        zorp: FORMAT_VERSION,
        name: snapshot.name.clone(),
    };
    files.insert(ROOT_FILE.into(), yaml(&root)?);

    let mut collections: Vec<&CollectionEntry> = snapshot.collections.iter().collect();
    by_order(&mut collections, |c| {
        (c.collection.sort_order, c.collection.id)
    });
    let mut names = Names::default();
    for entry in collections {
        let c = &entry.collection;
        let dir = format!("{COLLECTIONS_DIR}/{}", names.claim(&c.name, c.id));
        let file = CollectionFile {
            id: c.id,
            name: c.name.clone(),
            order: c.sort_order,
            auth: c.auth.clone(),
            variables: entry
                .variables
                .iter()
                .map(|(key, value)| Pair {
                    key: key.clone(),
                    value: value.clone(),
                })
                .collect(),
        };
        files.insert(format!("{dir}/{COLLECTION_FILE}"), yaml(&file)?);
        render_level(snapshot, c.id, None, &dir, &mut files)?;
    }

    let mut envs: Vec<_> = snapshot.environments.iter().collect();
    envs.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    let mut names = Names::default();
    for env in envs {
        let file = EnvironmentFile {
            id: env.id,
            name: env.name.clone(),
            variables: env
                .variables
                .iter()
                .map(|v| EnvVariable {
                    key: v.key.clone(),
                    value: if v.secret {
                        String::new()
                    } else {
                        v.value.clone()
                    },
                    enabled: v.enabled,
                    secret: v.secret,
                })
                .collect(),
        };
        let path = format!("{ENVIRONMENTS_DIR}/{}.yaml", names.claim(&env.name, env.id));
        files.insert(path, yaml(&file)?);
    }
    Ok(files)
}

fn render_level(
    snapshot: &Snapshot,
    collection: Uuid,
    parent: Option<Uuid>,
    dir: &str,
    files: &mut FileMap,
) -> AppResult<()> {
    let mut folders: Vec<_> = snapshot
        .folders
        .iter()
        .filter(|f| f.collection_id == collection && f.parent_id == parent)
        .collect();
    by_order(&mut folders, |f| (f.sort_order, f.id));
    let mut dirs = Names::default();
    for folder in folders {
        let sub = format!("{dir}/{}", dirs.claim(&folder.name, folder.id));
        let file = FolderFile {
            id: folder.id,
            name: folder.name.clone(),
            order: folder.sort_order,
            auth: folder.auth.clone(),
        };
        files.insert(format!("{sub}/{FOLDER_FILE}"), yaml(&file)?);
        render_level(snapshot, collection, Some(folder.id), &sub, files)?;
    }

    let mut requests: Vec<_> = snapshot
        .requests
        .iter()
        .filter(|r| r.collection_id == collection && r.folder_id == parent)
        .collect();
    by_order(&mut requests, |r| (r.sort_order, r.spec.id));
    let mut names = Names::default();
    for request in requests {
        let name = names.claim(&request.spec.name, request.spec.id);
        let file = RequestFile::from_spec(request.spec.clone(), request.sort_order);
        files.insert(format!("{dir}/{name}{REQUEST_SUFFIX}"), yaml(&file)?);
    }
    Ok(())
}
