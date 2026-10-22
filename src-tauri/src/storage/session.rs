use super::atomic::{DiskWriter, FileWriter};
use super::error::StorageError;
use super::files::{
    palette_path, sprite_path, write_manifest, write_palette, write_sprite,
};
use super::format::{
    palette_from_text, palette_to_text, sprite_from_text, sprite_palette_name, sprite_to_text,
};
use super::watch::Watcher;
use super::format::Manifest;
use super::trash::Trash;
use crate::document::{
    DocError, Folder, Operation, Project, SpriteRef, Target, validate_name,
};
use crate::history::History;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap};
use std::hash::{Hash, Hasher};
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

/// A change found in the project files made outside the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalChange {
    /// The file changed, was added, and the project now shows it.
    Reloaded(Target),
    /// The file was deleted, and the project does not have it anymore.
    Removed(Target),
    /// The file changed but the app has unsaved changes for it. The user
    /// chooses with `Session::resolve_conflict`.
    Conflict(Target),
    /// The file is not valid. The project keeps what it had in memory.
    Damaged { path: PathBuf, reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictChoice {
    /// Take the version on disk.
    Reload,
    /// Keep the version of the app and write it over the file.
    Keep,
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
    /// A fingerprint of each project file as the app last saw or wrote it
    /// (`None` means it did not exist). A file event whose file still has
    /// this fingerprint is the app's own write, or nothing new.
    known: HashMap<PathBuf, Option<u64>>,
    watcher: Option<Watcher>,
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
            known: HashMap::new(),
            watcher: None,
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

    /// Starts watching the project folder for changes made outside the app.
    pub fn watch(&mut self) -> Result<(), StorageError> {
        self.watcher = Some(Watcher::start(&self.root)?);
        Ok(())
    }

    /// Looks at the file events since the last call. A changed file with no
    /// unsaved changes is reloaded; one with unsaved changes is reported as a
    /// conflict. The app's own writes are ignored. Any reload clears the
    /// history, because its inverses do not know about the new content.
    pub fn poll_external(&mut self) -> Vec<ExternalChange> {
        let Some(watcher) = &self.watcher else {
            return Vec::new();
        };
        let mut paths = BTreeSet::new();
        for path in watcher.drain() {
            add_with_files_inside(&path, &mut paths);
        }
        let mut changes = Vec::new();
        for path in paths {
            let Some(target) = self.project_file(&path) else {
                continue;
            };
            let now = file_hash(&path);
            if self.known.get(&path).copied().flatten() == now {
                continue;
            }
            self.known.insert(path.clone(), now);
            if self.pending.contains(&target) {
                changes.push(ExternalChange::Conflict(target));
            } else if let Some(change) = self.load_from_disk(&target, &path) {
                changes.push(change);
            }
        }
        changes
    }

    /// Applies the choice of the user for a conflict.
    pub fn resolve_conflict(
        &mut self,
        target: &Target,
        choice: ConflictChoice,
    ) -> Option<ExternalChange> {
        match choice {
            ConflictChoice::Keep => {
                self.changed(vec![target.clone()]);
                None
            }
            ConflictChoice::Reload => {
                self.pending.remove(target);
                let path = self.file_of(target)?;
                self.known.insert(path.clone(), file_hash(&path));
                let change = self.load_from_disk(target, &path);
                if self.pending.is_empty() && self.status() == SaveStatus::Saving {
                    self.set_status(SaveStatus::Saved);
                }
                change
            }
        }
    }

    // What a path inside the project is: a sprite file, a palette file or
    // something else (the manifest, the trash, a temporary file).
    fn project_file(&self, path: &Path) -> Option<Target> {
        let relative = path.strip_prefix(&self.root).ok()?;
        let parts: Vec<String> = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        match parts.as_slice() {
            [dir, file] if dir == "palettes" => file
                .strip_suffix(".palette.json")
                .map(|name| Target::Palette(name.to_string())),
            [dir, folder @ .., file] if dir == "sprites" => {
                file.strip_suffix(".sprite.json").map(|name| {
                    Target::Sprite(SpriteRef {
                        folder: folder.to_vec(),
                        name: name.to_string(),
                    })
                })
            }
            _ => None,
        }
    }

    fn file_of(&self, target: &Target) -> Option<PathBuf> {
        match target {
            Target::Sprite(sprite) => Some(sprite_path(&self.root, sprite)),
            Target::Palette(name) => Some(palette_path(&self.root, name)),
            Target::Folder(_) => None,
        }
    }

    // Makes the project match the file on disk. Returns what happened, if anything.
    fn load_from_disk(&mut self, target: &Target, path: &Path) -> Option<ExternalChange> {
        let result = match target {
            Target::Sprite(sprite) => self.reload_sprite(sprite, path),
            Target::Palette(name) => self.reload_palette(name, path),
            Target::Folder(_) => return None,
        };
        match result {
            Ok(true) => {
                self.history.clear();
                Some(if path.exists() {
                    ExternalChange::Reloaded(target.clone())
                } else {
                    ExternalChange::Removed(target.clone())
                })
            }
            Ok(false) => None,
            Err(reason) => Some(ExternalChange::Damaged {
                path: path.to_path_buf(),
                reason,
            }),
        }
    }

    // Returns true if the project changed.
    fn reload_sprite(&mut self, sprite_ref: &SpriteRef, path: &Path) -> Result<bool, String> {
        if !path.exists() {
            return Ok(self.project.remove_sprite(sprite_ref));
        }
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let palette_name = sprite_palette_name(&text)?;
        let palette = self
            .project
            .palette(&palette_name)
            .ok_or(format!("palette {palette_name:?} does not exist"))?;
        let sprite = sprite_from_text(&sprite_ref.name, &text, palette)?;
        self.project.put_sprite(&sprite_ref.folder, sprite);
        Ok(true)
    }

    fn reload_palette(&mut self, name: &str, path: &Path) -> Result<bool, String> {
        if !path.exists() {
            return self.project.remove_palette(name).map_err(|e| e.to_string());
        }
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let palette = palette_from_text(name, &text)?;
        self.project.put_palette(palette).map_err(|e| e.to_string())?;
        Ok(true)
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
                self.known.insert(path.clone(), file_hash(&path));
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
                self.known.insert(path.clone(), file_hash(&path));
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

fn file_hash(path: &Path) -> Option<u64> {
    let bytes = fs::read(path).ok()?;
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    Some(hasher.finish())
}

// A new folder can get its files before the system starts watching it, so
// the files of a reported folder are looked at too.
fn add_with_files_inside(path: &Path, paths: &mut BTreeSet<PathBuf>) {
    paths.insert(path.to_path_buf());
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            add_with_files_inside(&entry.path(), paths);
        }
    }
}
