use super::error::StorageError;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// The trash of a project: `.sprixel/trash/<id>/<path inside the project>`.
/// Deleted files are moved here, not erased, so Undo can move them back.
pub(super) struct Trash {
    root: PathBuf,
    /// Where each deleted path went (the latest one if it was deleted twice).
    moved: HashMap<PathBuf, PathBuf>,
}

impl Trash {
    pub fn new(root: &Path) -> Trash {
        Trash {
            root: root.to_path_buf(),
            moved: HashMap::new(),
        }
    }

    /// Moves a file or folder to the trash. Does nothing if it does not exist
    /// (it was never saved).
    pub fn throw_away(&mut self, path: &Path) -> Result<(), StorageError> {
        if !path.exists() {
            return Ok(());
        }
        let relative = path.strip_prefix(&self.root).unwrap_or(path);
        let destination = self.root.join(".sprixel/trash").join(unique_id()).join(relative);
        move_path(path, &destination)?;
        self.moved.insert(path.to_path_buf(), destination);
        Ok(())
    }

    /// If `path` was thrown away and is missing now, and the trashed copy
    /// passes `is_same`, moves it back and returns true.
    pub fn take_back(
        &mut self,
        path: &Path,
        is_same: impl FnOnce(&Path) -> bool,
    ) -> Result<bool, StorageError> {
        if path.exists() {
            return Ok(false);
        }
        let Some(trashed) = self.moved.get(path) else {
            return Ok(false);
        };
        if !is_same(trashed) {
            return Ok(false);
        }
        move_path(trashed, path)?;
        self.moved.remove(path);
        Ok(true)
    }
}

fn unique_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos}-{}", COUNTER.fetch_add(1, Ordering::SeqCst))
}

fn move_path(from: &Path, to: &Path) -> Result<(), StorageError> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|e| StorageError::io(parent, &e))?;
    }
    fs::rename(from, to).map_err(|e| StorageError::io(from, &e))
}
