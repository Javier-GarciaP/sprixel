use super::*;
use crate::document::{Color, Operation, Palette, PixelChange, SpriteRef};
use std::fs;

pub(super) fn dungeon16() -> Palette {
    Palette {
        name: "dungeon16".to_string(),
        colors: "NPAR"
            .chars()
            .map(|key| Color {
                key,
                name: format!("color {key}"),
                hex: "#112233".to_string(),
            })
            .collect(),
    }
}

/// Creates a project called "Kit" with the palette, a folder "Personajes"
/// and a 4x4 sprite "goblin" inside it, all still unsaved.
pub(super) fn session_with_goblin(parent: &std::path::Path) -> Session {
    let mut session = Session::create(parent, "Kit").unwrap();
    let ops = vec![
        Operation::CreatePalette {
            palette: dungeon16(),
            index: None,
        },
        Operation::CreateFolder {
            parent: vec![],
            name: "Personajes".to_string(),
        },
        Operation::CreateSprite {
            folder: vec!["Personajes".to_string()],
            name: "goblin".to_string(),
            width: 4,
            height: 4,
            palette: "dungeon16".to_string(),
        },
    ];
    session.apply("user", ops).unwrap();
    session
}

pub(super) fn goblin_ref() -> SpriteRef {
    SpriteRef::new(&["Personajes"], "goblin")
}

pub(super) fn draw(x: u32, key: char) -> Operation {
    Operation::SetPixels {
        sprite: goblin_ref(),
        layer: 0,
        frame: 1,
        pixels: vec![PixelChange { x, y: 0, key }],
    }
}

// Checks that creating a project makes the folder with a valid manifest,
// `palettes/`, `sprites/` and `.sprixel/`.
#[test]
fn creating_a_project_makes_the_folder_layout() {
    let parent = tempfile::tempdir().unwrap();

    let session = Session::create(parent.path(), "Dungeon Kit").unwrap();

    let root = parent.path().join("Dungeon Kit");
    assert_eq!(read_manifest(&root).unwrap().name, "Dungeon Kit");
    assert!(root.join("palettes").is_dir());
    assert!(root.join("sprites").is_dir());
    assert!(root.join(".sprixel").is_dir());
    assert_eq!(session.project.name, "Dungeon Kit");
    assert_eq!(session.status(), SaveStatus::Saved);
}

// Checks that a project cannot be created over an existing folder or with an invalid name.
#[test]
fn creating_a_project_checks_the_folder_and_the_name() {
    let parent = tempfile::tempdir().unwrap();
    fs::create_dir(parent.path().join("Taken")).unwrap();
    assert!(Session::create(parent.path(), "Taken").is_err());
    assert!(Session::create(parent.path(), "CON").is_err());
    assert!(Session::create(parent.path(), "a/b").is_err());
}

// Checks that changes wait in memory with the status "saving", and a flush writes
// the sprite, the palette and the folder, and sets the status to "saved".
#[test]
fn flush_writes_what_changed() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    let root = parent.path().join("Kit");
    assert_eq!(session.status(), SaveStatus::Saving);
    assert!(!sprite_path(&root, &goblin_ref()).exists());

    assert_eq!(session.flush(), SaveStatus::Saved);

    let palette = read_palette(&palette_path(&root, "dungeon16")).unwrap();
    assert_eq!(palette, session.project.palettes[0]);
    let sprite = read_sprite(&root, &goblin_ref(), &palette).unwrap();
    assert_eq!(sprite, session.project.root.folders[0].sprites[0]);
}

// Checks that an empty folder is kept on disk too.
#[test]
fn flush_creates_empty_folders() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = Session::create(parent.path(), "Kit").unwrap();
    let op = Operation::CreateFolder {
        parent: vec![],
        name: "Vacia".to_string(),
    };
    session.apply("user", vec![op]).unwrap();
    session.flush();
    assert!(parent.path().join("Kit/sprites/Vacia").is_dir());
}

// Checks that only changed files are written again: a sprite that did not change is not rewritten.
#[test]
fn flush_does_not_rewrite_unchanged_files() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    let root = parent.path().join("Kit");
    session.flush();
    fs::remove_file(sprite_path(&root, &goblin_ref())).unwrap();

    session.flush();

    assert!(!sprite_path(&root, &goblin_ref()).exists());
}

// Checks that undo is saved too: after undoing a stroke and flushing, the file has the old pixels.
#[test]
fn undo_is_saved() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    session.apply("user", vec![draw(0, 'N')]).unwrap();
    session.flush();
    session.undo().unwrap();
    session.flush();

    let root = parent.path().join("Kit");
    let sprite = read_sprite(&root, &goblin_ref(), &dungeon16()).unwrap();
    assert_eq!(sprite.layers[0].frames[0].pixels, vec![b'.'; 16]);
}

// Checks that a rejected change does not mark anything as unsaved.
#[test]
fn a_rejected_change_stays_saved() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    assert!(session.apply("user", vec![draw(0, 'X')]).is_err());
    assert_eq!(session.status(), SaveStatus::Saved);
}
