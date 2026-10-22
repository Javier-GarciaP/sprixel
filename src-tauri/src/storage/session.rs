use super::atomic::{DiskWriter, FileWriter};
use super::error::StorageError;
use super::files::{write_manifest, write_palette, write_sprite};
use super::format::Manifest;
use crate::document::{DocError, Operation, Project, Target, validate_name};
use crate::history::History;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// What the interface shows about saving.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveStatus {
    Saved,
    /// There are changes that are not on the disk yet.
    Saving,
    /// The last save failed. The text is the reason (no space, no permission...).
    Error(String),
}

/// An open project: the document, its history and what still has to be
/// written to the disk.
pub struct Session {
    root: PathBuf,
    pub project: Project,
    pub history: History,
    /// The parts of the project changed since they were last saved.
    pending: BTreeSet<Target>,
    writer: Arc<dyn FileWriter>,
    status: Arc<Mutex<SaveStatus>>,
}

impl Session {
    /// Creates the project folder `parent/name` with a manifest, `palettes/`,
    /// `sprites/` and `.sprixel/`. The folder must not exist.
    pub fn create(parent: &Path, name: &str) -> Result<Session, StorageError> {
        let root = parent.join(name);
        validate_name(name).map_err(|e| StorageError::invalid(&root, e.message))?;
        if root.exists() {
            return Err(StorageError::invalid(&root, "this folder already exists"));
        }
        for folder in ["palettes", "sprites", ".sprixel"] {
            let path = root.join(folder);
            fs::create_dir_all(&path).map_err(|e| StorageError::io(&path, &e))?;
        }
        let root = fs::canonicalize(&root).map_err(|e| StorageError::io(&root, &e))?;
        let manifest = Manifest {
            name: name.to_string(),
            active_palette: None,
        };
        write_manifest(&DiskWriter, &root, &manifest)?;
        Ok(Session::new(root, Project::new(name)))
    }

    fn new(root: PathBuf, project: Project) -> Session {
        Session {
            root,
            project,
            history: History::new(),
            pending: BTreeSet::new(),
            writer: Arc::new(DiskWriter),
            status: Arc::new(Mutex::new(SaveStatus::Saved)),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Replaces the way files are written. Tests use it to simulate a full disk.
    pub fn set_writer(&mut self, writer: Arc<dyn FileWriter>) {
        self.writer = writer;
    }

    pub fn status(&self) -> SaveStatus {
        self.status.lock().unwrap().clone()
    }

    /// A handle to read the status without locking the session.
    pub(super) fn status_handle(&self) -> Arc<Mutex<SaveStatus>> {
        self.status.clone()
    }

    /// Applies the operations of one action as one history entry.
    pub fn apply(&mut self, author: &str, ops: Vec<Operation>) -> Result<(), DocError> {
        let touched = self.history.apply(&mut self.project, author, ops)?;
        self.changed(touched);
        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), DocError> {
        let touched = self.history.undo(&mut self.project)?;
        self.changed(touched);
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), DocError> {
        let touched = self.history.redo(&mut self.project)?;
        self.changed(touched);
        Ok(())
    }

    pub fn undo_last_ai_action(&mut self) -> Result<(), DocError> {
        let touched = self.history.undo_last_ai_action(&mut self.project)?;
        self.changed(touched);
        Ok(())
    }

    fn changed(&mut self, touched: Vec<Target>) {
        if touched.is_empty() {
            return;
        }
        self.pending.extend(touched);
        let mut status = self.status.lock().unwrap();
        if *status == SaveStatus::Saved {
            *status = SaveStatus::Saving;
        }
    }

    /// Writes everything that changed. Returns the new status.
    pub fn flush(&mut self) -> SaveStatus {
        if self.pending.is_empty() {
            return self.status();
        }
        self.set_status(SaveStatus::Saving);
        for target in self.pending.clone() {
            if let Err(error) = self.save(&target) {
                let status = SaveStatus::Error(error.to_string());
                self.set_status(status.clone());
                return status;
            }
            self.pending.remove(&target);
        }
        self.set_status(SaveStatus::Saved);
        SaveStatus::Saved
    }

    fn set_status(&self, status: SaveStatus) {
        *self.status.lock().unwrap() = status;
    }

    // Saves one part of the project as it is now in memory.
    fn save(&self, target: &Target) -> Result<(), StorageError> {
        let writer = &*self.writer;
        match target {
            Target::Sprite(sprite_ref) => {
                if let Some(sprite) = self.project.find_sprite(sprite_ref) {
                    write_sprite(writer, &self.root, sprite_ref, sprite)?;
                }
            }
            Target::Palette(name) => {
                if let Some(palette) = self.project.palette(name) {
                    write_palette(writer, &self.root, palette)?;
                }
            }
            Target::Folder(path) => {
                if self.project.has_folder(path) {
                    let mut folder = self.root.join("sprites");
                    folder.extend(path);
                    fs::create_dir_all(&folder).map_err(|e| StorageError::io(&folder, &e))?;
                }
            }
        }
        Ok(())
    }
}
