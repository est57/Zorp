use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;
use uuid::Uuid;

use super::format::{
    CollectionFile, EnvironmentFile, FORMAT_VERSION, FolderFile, RequestFile, WorkspaceFile,
};
use super::*;
use crate::models::collection::{Collection, Folder, SavedRequest};
use crate::models::environment::{Environment, Variable};

const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_FILES: usize = 20_000;

/// Problems found while reading a folder. While any exist, nothing is written or deleted.
#[derive(Debug, Default)]
pub struct Problems(pub Vec<String>);

struct Reader<'a> {
    root: &'a Path,
    files: usize,
    ids: HashSet<Uuid>,
    problems: Problems,
    snapshot: Snapshot,
}

/// Reads a workspace folder. Symbolic links and hidden entries such as `.git` are skipped.
pub fn read(root: &Path) -> Result<Snapshot, Problems> {
    let mut reader = Reader {
        root,
        files: 0,
        ids: HashSet::new(),
        problems: Problems::default(),
        snapshot: Snapshot::default(),
    };
    let Some(head) = reader.load::<WorkspaceFile>(&root.join(ROOT_FILE)) else {
        return Err(reader.problems);
    };
    if head.zorp > FORMAT_VERSION {
        reader.problems.0.push(format!(
            "{ROOT_FILE} dibuat oleh versi Zorp yang lebih baru"
        ));
    }
    reader.snapshot.name = head.name;
    for dir in reader.entries(&root.join(COLLECTIONS_DIR), true) {
        reader.collection(&dir);
    }
    for path in reader.entries(&root.join(ENVIRONMENTS_DIR), false) {
        if path.extension().is_some_and(|e| e == "yaml") {
            reader.environment(&path);
        }
    }
    if reader.problems.0.is_empty() {
        Ok(reader.snapshot)
    } else {
        Err(reader.problems)
    }
}

impl Reader<'_> {
    fn relative(&self, path: &Path) -> String {
        path.strip_prefix(self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }

    /// Visible, non-link entries of a directory, sorted; `dirs` picks directories or files.
    fn entries(&mut self, dir: &Path, dirs: bool) -> Vec<std::path::PathBuf> {
        let Ok(read) = fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut out: Vec<_> = read
            .flatten()
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .filter(|e| {
                e.file_type()
                    .is_ok_and(|t| if dirs { t.is_dir() } else { t.is_file() })
            })
            .map(|e| e.path())
            .collect();
        out.sort();
        out
    }

    fn load<T: DeserializeOwned>(&mut self, path: &Path) -> Option<T> {
        let name = self.relative(path);
        self.files += 1;
        if self.files > MAX_FILES {
            if self.files == MAX_FILES + 1 {
                self.problems
                    .0
                    .push(format!("Folder berisi lebih dari {MAX_FILES} file"));
            }
            return None;
        }
        let text = match fs::symlink_metadata(path) {
            Ok(m) if !m.is_file() => Err("bukan file biasa".to_string()),
            Ok(m) if m.len() > MAX_FILE_BYTES => Err("ukuran file melebihi 2 MB".to_string()),
            Ok(_) => fs::read_to_string(path).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };
        match text.and_then(|t| serde_saphyr::from_str::<T>(&t).map_err(|e| e.to_string())) {
            Ok(value) => Some(value),
            Err(e) => {
                self.problems.0.push(format!("{name}: {e}"));
                None
            }
        }
    }

    fn claim(&mut self, id: Uuid, name: &str, path: &Path) -> bool {
        let problem = if name.trim().is_empty() {
            "nama tidak boleh kosong".to_string()
        } else if !self.ids.insert(id) {
            format!("id {id} dipakai lebih dari sekali")
        } else {
            return true;
        };
        self.problems
            .0
            .push(format!("{}: {problem}", self.relative(path)));
        false
    }

    fn collection(&mut self, dir: &Path) {
        let path = dir.join(COLLECTION_FILE);
        if !path.is_file() {
            return;
        }
        let Some(file) = self.load::<CollectionFile>(&path) else {
            return;
        };
        if !self.claim(file.id, &file.name, &path) {
            return;
        }
        self.snapshot.collections.push(CollectionEntry {
            collection: Collection {
                id: file.id,
                name: file.name,
                auth: file.auth,
                sort_order: file.order,
            },
            variables: file
                .variables
                .into_iter()
                .map(|p| (p.key, p.value))
                .collect(),
        });
        self.level(dir, file.id, None);
    }

    fn level(&mut self, dir: &Path, collection: Uuid, parent: Option<Uuid>) {
        for path in self.entries(dir, false) {
            if !path.to_string_lossy().ends_with(REQUEST_SUFFIX) {
                continue;
            }
            let Some(file) = self.load::<RequestFile>(&path) else {
                continue;
            };
            if self.claim(file.id, &file.name, &path) {
                let (spec, sort_order) = file.into_spec();
                self.snapshot.requests.push(SavedRequest {
                    collection_id: collection,
                    folder_id: parent,
                    sort_order,
                    spec,
                });
            }
        }
        for sub in self.entries(dir, true) {
            let path = sub.join(FOLDER_FILE);
            if !path.is_file() {
                continue;
            }
            let Some(file) = self.load::<FolderFile>(&path) else {
                continue;
            };
            if self.claim(file.id, &file.name, &path) {
                self.snapshot.folders.push(Folder {
                    id: file.id,
                    collection_id: collection,
                    parent_id: parent,
                    name: file.name,
                    auth: file.auth,
                    sort_order: file.order,
                });
                self.level(&sub, collection, Some(file.id));
            }
        }
    }

    fn environment(&mut self, path: &Path) {
        let Some(file) = self.load::<EnvironmentFile>(path) else {
            return;
        };
        if !self.claim(file.id, &file.name, path) {
            return;
        }
        let variables = file
            .variables
            .into_iter()
            .map(|v| Variable {
                key: v.key,
                value: if v.secret { String::new() } else { v.value },
                enabled: v.enabled,
                secret: v.secret,
                has_value: false,
            })
            .collect();
        self.snapshot.environments.push(Environment {
            id: file.id,
            name: file.name,
            variables,
            is_active: false,
        });
    }
}
