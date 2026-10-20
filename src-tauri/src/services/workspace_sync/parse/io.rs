use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;

use super::{MAX_FILE_BYTES, MAX_FILES, Reader};

impl Reader<'_> {
    pub(super) fn relative(&self, path: &Path) -> String {
        path.strip_prefix(self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }

    /// Visible, non-link entries of a directory, sorted; `dirs` picks directories or files.
    pub(super) fn entries(&mut self, dir: &Path, dirs: bool) -> Vec<std::path::PathBuf> {
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

    pub(super) fn load<T: DeserializeOwned>(&mut self, path: &Path) -> Option<T> {
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
        let text = text.and_then(|t| {
            if t.lines()
                .any(|l| l.starts_with("<<<<<<< ") || l == "=======")
            {
                Err("berisi konflik Git yang belum diselesaikan".to_string())
            } else {
                Ok(t)
            }
        });
        match text.and_then(|t| serde_saphyr::from_str::<T>(&t).map_err(|e| e.to_string())) {
            Ok(value) => Some(value),
            Err(e) => {
                self.problems.0.push(format!("{name}: {e}"));
                None
            }
        }
    }
}
