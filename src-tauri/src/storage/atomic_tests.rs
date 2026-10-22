use super::*;
use crate::document::{Palette, Sprite, SpriteRef};
use std::fs;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

/// Writes the first half of the bytes to the temporary file and then fails,
/// like a disk that breaks in the middle of a save.
struct BreaksHalfway;

impl FileWriter for BreaksHalfway {
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        fs::write(path, &bytes[..bytes.len() / 2])?;
        Err(io::Error::other("the disk broke"))
    }
}

/// Writes the first half and then panics, like a process that is killed
/// (the code after the panic, including any clean-up, never runs).
struct DiesHalfway;

impl FileWriter for DiesHalfway {
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        fs::write(path, &bytes[..bytes.len() / 2])?;
        panic!("the process was killed");
    }
}

fn names_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn dungeon_palette() -> Palette {
    Palette {
        name: "dungeon16".to_string(),
        colors: vec![],
    }
}

// Checks that a normal write puts the full new content in place and leaves no temporary file.
#[test]
fn a_write_replaces_the_file_and_leaves_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("goblin.sprite.json");
    fs::write(&path, "old").unwrap();

    write_atomic(&DiskWriter, &path, b"new content").unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "new content");
    assert_eq!(names_in(dir.path()), ["goblin.sprite.json"]);
}

// Checks that a write works when the file does not exist yet.
#[test]
fn a_write_creates_a_new_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("new.sprite.json");
    write_atomic(&DiskWriter, &path, b"hello").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
}

// Checks that when the writer fails halfway, the error is returned, the old file
// is still the full old version, and the half-written temporary file is removed.
#[test]
fn a_failed_write_keeps_the_old_file_intact() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("goblin.sprite.json");
    fs::write(&path, "the full old version").unwrap();

    let result = write_atomic(&BreaksHalfway, &path, b"the full new version");

    assert!(result.is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "the full old version");
    assert_eq!(names_in(dir.path()), ["goblin.sprite.json"]);
}

// Checks that a failed first save leaves no file at all (not a half file).
#[test]
fn a_failed_first_write_leaves_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("goblin.sprite.json");
    assert!(write_atomic(&BreaksHalfway, &path, b"abcdef").is_err());
    assert!(!path.exists());
}

// Checks the "process killed mid-save" case with a panic that skips the clean-up:
// the sprite file is still the full old version and still reads as a valid sprite.
#[test]
fn a_killed_writer_leaves_the_old_valid_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let sprite = Sprite::new("goblin", 2, 2, "dungeon16").unwrap();
    let sprite_ref = SpriteRef::new(&[], "goblin");
    write_sprite(&DiskWriter, root, &sprite_ref, &sprite).unwrap();
    let old_text = fs::read_to_string(sprite_path(root, &sprite_ref)).unwrap();

    let mut changed = sprite.clone();
    changed.layers[0].frames[0].pixels[0] = b'N';
    let killed = catch_unwind(AssertUnwindSafe(|| {
        let _ = write_sprite(&DiesHalfway, root, &sprite_ref, &changed);
    }));

    assert!(killed.is_err(), "the writer should have been killed");
    assert_eq!(fs::read_to_string(sprite_path(root, &sprite_ref)).unwrap(), old_text);
    let reread = read_sprite(root, &sprite_ref, &dungeon_palette_with_n()).unwrap();
    assert_eq!(reread, sprite);
    // The half-written temporary file stays, but it is not a sprite file.
    let leftovers = names_in(&root.join("sprites"));
    assert_eq!(leftovers, ["goblin.sprite.json", "goblin.sprite.json.tmp"]);

    // The next save works and replaces the stale temporary file.
    write_sprite(&DiskWriter, root, &sprite_ref, &changed).unwrap();
    assert_eq!(names_in(&root.join("sprites")), ["goblin.sprite.json"]);
    let saved = read_sprite(root, &sprite_ref, &dungeon_palette_with_n()).unwrap();
    assert_eq!(saved, changed);
}

fn dungeon_palette_with_n() -> Palette {
    let mut palette = dungeon_palette();
    palette.colors.push(crate::document::Color {
        key: 'N',
        name: "black".to_string(),
        hex: "#000000".to_string(),
    });
    palette
}
