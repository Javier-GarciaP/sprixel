use super::atomic::{DiskWriter, FileWriter};
use super::error::StorageError;
use super::files::{
    palette_path, sprite_path, write_manifest, write_palette, write_sprite,
};
use super::format::{palette_to_text, sprite_to_text};
use super::format::Manifest;
use super::trash::Trash;
use crate::document::{DocError, Folder, Operation, Project, Target, validate_name};
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
    trash: Trash,
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
        let trash = Trash::new(&root);
        Session {
            root,
            project,
            history: History::new(),
            pending: BTreeSet::new(),
            writer: Arc::new(DiskWriter),
            trash,
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

    /// The "Retry" action after a save error. The changes are still in
    /// memory, so this tries to write them again.
    pub fn retry(&mut self) -> SaveStatus {
        self.flush()
    }

    fn set_status(&self, status: SaveStatus) {
        *self.status.lock().unwrap() = status;
    }

    // Brings the disk in line with one part of the project as it is in memory:
    // written when it exists, moved to the trash when it was deleted, and moved
    // back from the trash when a deletion was undone.
    fn save(&mut self, target: &Target) -> Result<(), StorageError> {
        let writer = self.writer.clone();
        match target {
            Target::Sprite(sprite_ref) => {
                let path = sprite_path(&self.root, sprite_ref);
                match self.project.find_sprite(sprite_ref) {
                    Some(sprite) => {
                        let text = sprite_to_text(sprite);
                        self.trash.take_back(&path, |trashed| has_text(trashed, &text))?;
                        write_sprite(&*writer, &self.root, sprite_ref, sprite)?;
                    }
                    None => self.trash.throw_away(&path)?,
                }
            }
            Target::Palette(name) => {
                let path = palette_path(&self.root, name);
                match self.project.palette(name) {
                    Some(palette) => {
                        let text = palette_to_text(palette);
                        self.trash.take_back(&path, |trashed| has_text(trashed, &text))?;
                        write_palette(&*writer, &self.root, palette)?;
                    }
                    None => self.trash.throw_away(&path)?,
                }
            }
            Target::Folder(names) => {
                let mut path = self.root.join("sprites");
                path.extend(names);
                match self.project.find_folder(names) {
                    Some(folder) => {
                        let wanted = names_in_folder(folder, "");
                        self.trash
                            .take_back(&path, |trashed| names_on_disk(trashed) == wanted)?;
                        fs::create_dir_all(&path).map_err(|e| StorageError::io(&path, &e))?;
                    }
                    None => self.trash.throw_away(&path)?,
                }
            }
        }
        Ok(())
    }
}

fn has_text(path: &Path, text: &str) -> bool {
    fs::read_to_string(path).is_ok_and(|saved| saved == text)
}

// The sprites and folders inside a folder, as lowercase relative paths. It is
// used to tell if a trashed folder is the one that Undo wants back.
fn names_in_folder(folder: &Folder, prefix: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for sprite in &folder.sprites {
        names.insert(format!("{prefix}{}.sprite.json", sprite.name).to_lowercase());
    }
    for child in &folder.folders {
        names.insert(format!("{prefix}{}", child.name).to_lowercase());
        names.extend(names_in_folder(child, &format!("{prefix}{}/", child.name)));
    }
    names
}

fn names_on_disk(dir: &Path) -> BTreeSet<String> {
    fn walk(dir: &Path, prefix: &str, names: &mut BTreeSet<String>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            if path.is_dir() {
                names.insert(format!("{prefix}{name}").to_lowercase());
                walk(&path, &format!("{prefix}{name}/"), names);
            } else if name.ends_with(".sprite.json") {
                names.insert(format!("{prefix}{name}").to_lowercase());
            }
        }
    }
    let mut names = BTreeSet::new();
    walk(dir, "", &mut names);
    names
}
