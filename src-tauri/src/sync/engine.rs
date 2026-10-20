use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use notify_debouncer_mini::Debouncer;
use notify_debouncer_mini::notify::RecommendedWatcher;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::sync::SyncStatus;
use crate::services::workspace_sync::render::{FileMap, render};
use crate::services::workspace_sync::{ROOT_FILE, files, parse};
use crate::state::AppState;

#[derive(Default)]
struct Inner {
    root: Option<PathBuf>,
    /// Managed files as last written or read, to tell our own writes from outside edits.
    last: Option<FileMap>,
    problems: Vec<String>,
    watcher: Option<Debouncer<RecommendedWatcher>>,
}

#[derive(Default)]
pub struct SyncEngine {
    inner: Mutex<Inner>,
}

impl SyncEngine {
    fn lock(&self) -> AppResult<MutexGuard<'_, Inner>> {
        self.inner
            .lock()
            .map_err(|_| AppError::Internal("Status sinkron tidak konsisten".into()))
    }

    pub fn status(&self) -> SyncStatus {
        let Ok(inner) = self.lock() else {
            return SyncStatus::default();
        };
        SyncStatus {
            folder: inner
                .root
                .as_ref()
                .map(|r| r.to_string_lossy().into_owned()),
            problems: inner.problems.clone(),
        }
    }

    /// Records a problem; sync stays paused until the folder reads cleanly again.
    pub fn report(&self, problem: String) {
        if let Ok(mut inner) = self.lock() {
            inner.problems = vec![problem];
        }
    }

    /// Binds to a folder (or none) and brings both sides in step: a folder that already
    /// holds a workspace is read into the app, an empty one receives the app's data.
    pub fn attach(
        &self,
        state: &AppState,
        folder: Option<PathBuf>,
        watcher: Option<Debouncer<RecommendedWatcher>>,
    ) -> AppResult<bool> {
        let old = {
            let mut inner = self.lock()?;
            let old = inner.watcher.take();
            *inner = Inner {
                root: folder.clone(),
                watcher,
                ..Inner::default()
            };
            old
        };
        drop(old);
        match folder {
            Some(root) if !root.is_dir() => {
                self.lock()?.problems = vec![format!("Folder {} tidak ditemukan", root.display())];
                Ok(false)
            }
            Some(root) if root.join(ROOT_FILE).is_file() => self.pull(state),
            Some(root) if !files::read_managed(&root).is_empty() => {
                self.lock()?.problems = vec![format!(
                    "Folder berisi file workspace tanpa {ROOT_FILE}; pilih folder kosong"
                )];
                Ok(false)
            }
            Some(_) => self.push(state).map(|_| false),
            None => Ok(false),
        }
    }

    /// Reads the folder into the app. Returns true when the app's data changed.
    pub fn pull(&self, state: &AppState) -> AppResult<bool> {
        let mut inner = self.lock()?;
        let Some(root) = inner.root.clone() else {
            return Ok(false);
        };
        let disk = files::read_managed(&root);
        if inner.last.as_ref() == Some(&disk) {
            // Files are back to what this app wrote: resume and write what changed meanwhile.
            if !inner.problems.is_empty() {
                inner.problems.clear();
                drop(inner);
                self.push(state)?;
            }
            return Ok(false);
        }
        let snapshot = match parse::read(&root) {
            Ok(snapshot) => snapshot,
            Err(problems) => {
                inner.problems = problems.0;
                return Ok(false);
            }
        };
        let conn = state.db()?;
        let ws = state.workspace_id();
        let changed = render(&snapshot)? != render(&db::snapshot::load(&conn, ws)?)?;
        if changed
            && let Err(e) = db::snapshot::replace(&conn, ws, &snapshot, state.secrets.as_ref())
        {
            inner.problems = vec![e.to_string()];
            return Ok(false);
        }
        inner.problems.clear();
        inner.last = Some(disk);
        Ok(changed)
    }

    /// Writes the app's data to the folder. Skipped while problems are open or while
    /// outside edits are waiting to be read, so those edits are never overwritten.
    pub fn push(&self, state: &AppState) -> AppResult<()> {
        let mut inner = self.lock()?;
        let Some(root) = inner.root.clone() else {
            return Ok(());
        };
        if !inner.problems.is_empty() || !root.is_dir() {
            return Ok(());
        }
        if inner
            .last
            .as_ref()
            .is_some_and(|last| *last != files::read_managed(&root))
        {
            return Ok(());
        }
        let wanted = render(&db::snapshot::load(&*state.db()?, state.workspace_id())?)?;
        files::write(&root, &wanted)?;
        inner.last = Some(wanted);
        Ok(())
    }
}

/// Canonical form of a folder chosen by the user.
pub fn canonical(path: &str) -> AppResult<PathBuf> {
    let path = Path::new(path);
    if !path.is_absolute() {
        return Err(AppError::Validation(
            "Pilih folder dengan path lengkap".into(),
        ));
    }
    let canonical = path.canonicalize()?;
    if !canonical.is_dir() {
        return Err(AppError::Validation(
            "Path yang dipilih bukan folder".into(),
        ));
    }
    Ok(canonical)
}
