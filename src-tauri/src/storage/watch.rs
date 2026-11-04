use super::error::StorageError;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher as _};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};

/// Watches a project folder and collects the paths of files that changed.
/// It does not decide what a change means: the session does.
pub(super) struct Watcher {
    // Kept alive: the watching stops when it is dropped.
    _watcher: RecommendedWatcher,
    changed: Receiver<PathBuf>,
}

impl Watcher {
    pub fn start(root: &Path) -> Result<Watcher, StorageError> {
        let (sender, changed) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
            let Ok(event) = event else { return };
            // Reading a file is not a change.
            if event.kind.is_access() {
                return;
            }
            for path in event.paths {
                let _ = sender.send(path);
            }
        })
        .map_err(|e| StorageError::invalid(root, e.to_string()))?;
        watcher
            .watch(root, RecursiveMode::Recursive)
            .map_err(|e| StorageError::invalid(root, e.to_string()))?;
        Ok(Watcher {
            _watcher: watcher,
            changed,
        })
    }

    /// The paths reported since the last call.
    pub fn drain(&self) -> Vec<PathBuf> {
        self.changed.try_iter().collect()
    }
}
