use super::error::StorageError;
use super::files::read_manifest;
use super::format::FORMAT_VERSION;
use super::load::{Damaged, file_name, sorted_entries};
use crate::document::validate_name;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// What the project browser shows about one sprite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteInfo {
    pub folder: Vec<String>,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub palette: String,
    pub frames: usize,
}

/// A fast look at a project: names and sizes, no pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub name: String,
    pub palettes: Vec<String>,
    /// Every folder as a path from the root, parents before children.
    pub folders: Vec<Vec<String>>,
    pub sprites: Vec<SpriteInfo>,
    pub damaged: Vec<Damaged>,
}

// The start of a sprite file. The layers and tags are skipped, not loaded.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Header {
    format_version: u32,
    width: u32,
    height: u32,
    palette: String,
    frame_durations: Vec<u32>,
}

/// Lists a project without loading the pixels of its sprites, so a project
/// with hundreds of sprites lists fast. The pixels are checked later, when a
/// sprite is opened (`Session::open`). A file that is not valid JSON, or has no
/// valid header, is listed as damaged.
pub fn list_project(root: &Path) -> Result<Listing, StorageError> {
    let manifest = read_manifest(root)?;
    let mut listing = Listing {
        name: manifest.name,
        palettes: Vec::new(),
        folders: Vec::new(),
        sprites: Vec::new(),
        damaged: Vec::new(),
    };
    for path in sorted_entries(&root.join("palettes")) {
        if let Some(name) = file_name(&path).strip_suffix(".palette.json") {
            listing.palettes.push(name.to_string());
        }
    }
    list_folder(&root.join("sprites"), &mut Vec::new(), &mut listing);
    Ok(listing)
}

fn list_folder(dir: &Path, folder: &mut Vec<String>, listing: &mut Listing) {
    let mut used = BTreeSet::new();
    for path in sorted_entries(dir) {
        let name = file_name(&path);
        if path.is_dir() {
            match check(&name, &mut used) {
                Ok(()) => {
                    folder.push(name);
                    listing.folders.push(folder.clone());
                    list_folder(&path, folder, listing);
                    folder.pop();
                }
                Err(reason) => damage(listing, &path, reason),
            }
        } else if let Some(sprite_name) = name.strip_suffix(".sprite.json") {
            let info = check(sprite_name, &mut used).and_then(|()| read_header(&path));
            match info {
                Ok(header) => listing.sprites.push(SpriteInfo {
                    folder: folder.clone(),
                    name: sprite_name.to_string(),
                    width: header.width,
                    height: header.height,
                    palette: header.palette,
                    frames: header.frame_durations.len(),
                }),
                Err(reason) => damage(listing, &path, reason),
            }
        }
    }
}

fn damage(listing: &mut Listing, path: &PathBuf, reason: String) {
    listing.damaged.push(Damaged {
        path: path.clone(),
        reason,
    });
}

// A name must be valid and not used before in the folder (ignoring case).
fn check(name: &str, used: &mut BTreeSet<String>) -> Result<(), String> {
    validate_name(name).map_err(|e| e.to_string())?;
    if !used.insert(name.to_lowercase()) {
        return Err(format!("name_exists: another sprite or folder is called {name:?}"));
    }
    Ok(())
}

fn read_header(path: &Path) -> Result<Header, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let header: Header = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    if header.format_version != FORMAT_VERSION {
        return Err(format!(
            "format version {} is not {FORMAT_VERSION}",
            header.format_version
        ));
    }
    Ok(header)
}
