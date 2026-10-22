use super::error::StorageError;
use super::files::{hash_bytes, read_manifest};
use super::format::{palette_from_text, sprite_from_text, sprite_palette_name};
use crate::document::{Folder, Palette, Project, same_name, validate_name};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A file that could not be loaded, with the path and the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Damaged {
    pub path: PathBuf,
    pub reason: String,
}

pub(super) struct Loaded {
    pub project: Project,
    pub damaged: Vec<Damaged>,
    /// A fingerprint of each file that was read (see `Session`).
    pub known: HashMap<PathBuf, Option<u64>>,
}

/// Reads a whole project folder. Only a missing or invalid manifest stops the
/// load. A palette, sprite or folder that is not valid is listed as damaged
/// and the rest loads normally.
pub(super) fn load_project(root: &Path) -> Result<Loaded, StorageError> {
    let manifest = read_manifest(root)?;
    let mut project = Project::new(&manifest.name);
    project.active_palette = manifest.active_palette;
    let mut loader = Loader {
        damaged: Vec::new(),
        known: HashMap::new(),
    };
    project.palettes = loader.load_palettes(&root.join("palettes"));
    project.root = loader.load_folder(&root.join("sprites"), "", &project.palettes);
    Ok(Loaded {
        project,
        damaged: loader.damaged,
        known: loader.known,
    })
}

struct Loader {
    damaged: Vec<Damaged>,
    known: HashMap<PathBuf, Option<u64>>,
}

impl Loader {
    fn damage(&mut self, path: &Path, reason: impl Into<String>) {
        self.damaged.push(Damaged {
            path: path.to_path_buf(),
            reason: reason.into(),
        });
    }

    // Reads a file and remembers its fingerprint.
    fn read(&mut self, path: &Path) -> Option<String> {
        match fs::read_to_string(path) {
            Ok(text) => {
                self.known.insert(path.to_path_buf(), Some(hash_bytes(text.as_bytes())));
                Some(text)
            }
            Err(error) => {
                self.damage(path, error.to_string());
                None
            }
        }
    }

    fn load_palettes(&mut self, dir: &Path) -> Vec<Palette> {
        let mut palettes: Vec<Palette> = Vec::new();
        for path in sorted_entries(dir) {
            let Some(name) = file_name(&path).strip_suffix(".palette.json").map(str::to_string)
            else {
                continue;
            };
            let Some(text) = self.read(&path) else { continue };
            if let Err(error) = validate_name(&name) {
                self.damage(&path, error.to_string());
            } else if palettes.iter().any(|p| same_name(&p.name, &name)) {
                self.damage(&path, "name_exists: another palette has this name");
            } else {
                match palette_from_text(&name, &text) {
                    Ok(palette) => palettes.push(palette),
                    Err(reason) => self.damage(&path, reason),
                }
            }
        }
        palettes
    }

    fn load_folder(&mut self, dir: &Path, name: &str, palettes: &[Palette]) -> Folder {
        let mut folder = Folder::new(name);
        for path in sorted_entries(dir) {
            let entry_name = file_name(&path);
            if path.is_dir() {
                match check_name(&folder, &entry_name) {
                    Ok(()) => {
                        let child = self.load_folder(&path, &entry_name, palettes);
                        folder.folders.push(child);
                    }
                    Err(reason) => self.damage(&path, reason),
                }
            } else if let Some(sprite_name) = entry_name.strip_suffix(".sprite.json") {
                let Some(text) = self.read(&path) else { continue };
                let loaded = check_name(&folder, sprite_name)
                    .and_then(|()| sprite_from_file(sprite_name, &text, palettes));
                match loaded {
                    Ok(sprite) => folder.sprites.push(sprite),
                    Err(reason) => self.damage(&path, reason),
                }
            }
        }
        folder
    }
}

fn sprite_from_file(
    name: &str,
    text: &str,
    palettes: &[Palette],
) -> Result<crate::document::Sprite, String> {
    let palette_name = sprite_palette_name(text)?;
    let palette = palettes
        .iter()
        .find(|p| same_name(&p.name, &palette_name))
        .ok_or(format!("palette {palette_name:?} does not exist or is damaged"))?;
    sprite_from_text(name, text, palette)
}

// A name must be valid and not used by another sprite or folder in the folder.
fn check_name(folder: &Folder, name: &str) -> Result<(), String> {
    validate_name(name).map_err(|e| e.to_string())?;
    let taken = folder.sprites.iter().any(|s| same_name(&s.name, name))
        || folder.folders.iter().any(|f| same_name(&f.name, name));
    if taken {
        return Err(format!("name_exists: another sprite or folder is called {name:?}"));
    }
    Ok(())
}

fn file_name(path: &Path) -> String {
    path.file_name().unwrap_or_default().to_string_lossy().into_owned()
}

// The entries of a folder in a steady order (the system order is random).
// A folder that does not exist has no entries.
fn sorted_entries(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort_by_key(|p| file_name(p).to_lowercase());
    paths
}
