use super::error::StorageError;
use super::format::{
    Manifest, manifest_from_text, manifest_to_text, palette_from_text, palette_to_text,
    sprite_from_text, sprite_to_text,
};
use crate::document::{Palette, Sprite, SpriteRef};
use std::fs;
use std::path::{Path, PathBuf};

const SPRITE_SUFFIX: &str = ".sprite.json";
const PALETTE_SUFFIX: &str = ".palette.json";

pub fn manifest_path(root: &Path) -> PathBuf {
    root.join("sprixel.json")
}

pub fn palette_path(root: &Path, name: &str) -> PathBuf {
    root.join("palettes").join(format!("{name}{PALETTE_SUFFIX}"))
}

/// The sprite folders mirror the project folders under `sprites/`.
pub fn sprite_path(root: &Path, sprite: &SpriteRef) -> PathBuf {
    let mut path = root.join("sprites");
    path.extend(&sprite.folder);
    path.join(format!("{}{SPRITE_SUFFIX}", sprite.name))
}

pub fn write_manifest(root: &Path, manifest: &Manifest) -> Result<(), StorageError> {
    write_text(&manifest_path(root), &manifest_to_text(manifest))
}

pub fn read_manifest(root: &Path) -> Result<Manifest, StorageError> {
    let path = manifest_path(root);
    manifest_from_text(&read_text(&path)?).map_err(|reason| StorageError::invalid(&path, reason))
}

pub fn write_palette(root: &Path, palette: &Palette) -> Result<(), StorageError> {
    write_text(&palette_path(root, &palette.name), &palette_to_text(palette))
}

/// Reads a palette file. The palette name is the file name without `.palette.json`.
pub fn read_palette(path: &Path) -> Result<Palette, StorageError> {
    let name = file_stem(path, PALETTE_SUFFIX);
    palette_from_text(&name, &read_text(path)?).map_err(|reason| StorageError::invalid(path, reason))
}

pub fn write_sprite(root: &Path, sprite_ref: &SpriteRef, sprite: &Sprite) -> Result<(), StorageError> {
    write_text(&sprite_path(root, sprite_ref), &sprite_to_text(sprite))
}

pub fn read_sprite(
    root: &Path,
    sprite_ref: &SpriteRef,
    palette: &Palette,
) -> Result<Sprite, StorageError> {
    let path = sprite_path(root, sprite_ref);
    sprite_from_text(&sprite_ref.name, &read_text(&path)?, palette)
        .map_err(|reason| StorageError::invalid(&path, reason))
}

fn file_stem(path: &Path, suffix: &str) -> String {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    name.strip_suffix(suffix).unwrap_or(&name).to_string()
}

fn read_text(path: &Path) -> Result<String, StorageError> {
    fs::read_to_string(path).map_err(|e| StorageError::io(path, &e))
}

fn write_text(path: &Path, text: &str) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| StorageError::io(parent, &e))?;
    }
    fs::write(path, text).map_err(|e| StorageError::io(path, &e))
}
