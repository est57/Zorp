use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::render::FileMap;
use super::*;
use crate::error::AppResult;

/// Whether a relative path is one of the files this app owns inside the folder.
fn is_managed(relative: &str) -> bool {
    if relative == ROOT_FILE {
        return true;
    }
    if let Some(rest) = relative.strip_prefix(&format!("{ENVIRONMENTS_DIR}/")) {
        return !rest.contains('/') && rest.ends_with(".yaml");
    }
    relative.starts_with(&format!("{COLLECTIONS_DIR}/"))
        && (relative.ends_with(REQUEST_SUFFIX)
            || relative.ends_with(&format!("/{COLLECTION_FILE}"))
            || relative.ends_with(&format!("/{FOLDER_FILE}")))
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>, dirs: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if kind.is_dir() {
            dirs.push(path.clone());
            walk(root, &path, out, dirs);
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if is_managed(&relative) {
                out.push(relative);
            }
        }
    }
}

/// Managed files in the folder, plus the directories under `collections`.
fn scan(root: &Path) -> (Vec<String>, Vec<PathBuf>) {
    let (mut paths, mut dirs) = (Vec::new(), Vec::new());
    if fs::symlink_metadata(root.join(ROOT_FILE)).is_ok_and(|m| m.is_file()) {
        paths.push(ROOT_FILE.to_string());
    }
    walk(root, &root.join(COLLECTIONS_DIR), &mut paths, &mut dirs);
    let mut env_dirs = Vec::new();
    walk(
        root,
        &root.join(ENVIRONMENTS_DIR),
        &mut paths,
        &mut env_dirs,
    );
    (paths, dirs)
}

/// Contents of the managed files currently in the folder.
pub fn read_managed(root: &Path) -> FileMap {
    scan(root)
        .0
        .into_iter()
        .filter_map(|p| fs::read_to_string(root.join(&p)).ok().map(|text| (p, text)))
        .collect()
}

fn write_atomic(path: &Path, content: &str) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temp = path.with_file_name(format!(".{name}.tmp"));
    let mut file = fs::File::create(&temp)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    fs::rename(&temp, path)?;
    Ok(())
}

/// Writes changed files and removes managed files that are no longer part of the workspace.
/// Other files in the folder are left alone. Returns the number of files touched.
pub fn write(root: &Path, files: &FileMap) -> AppResult<usize> {
    let (existing, mut dirs) = scan(root);
    let mut touched = 0;
    for (relative, content) in files {
        let path = root.join(relative);
        if fs::read_to_string(&path).ok().as_deref() != Some(content.as_str()) {
            write_atomic(&path, content)?;
            touched += 1;
        }
    }
    for relative in existing.iter().filter(|p| !files.contains_key(*p)) {
        fs::remove_file(root.join(relative))?;
        touched += 1;
    }
    // Deepest first, so emptied parents can go too; non-empty directories fail and stay.
    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    for dir in &dirs {
        let _ = fs::remove_dir(dir);
    }
    Ok(touched)
}
