use super::session_tests::{draw, goblin_ref, session_with_goblin};
use super::*;
use crate::document::{Operation, SpriteRef};
use std::fs;
use std::path::PathBuf;

fn root_of(parent: &std::path::Path) -> PathBuf {
    fs::canonicalize(parent.join("Kit")).unwrap()
}

fn trash_files(root: &std::path::Path) -> usize {
    fn count(dir: &std::path::Path) -> usize {
        let Ok(entries) = fs::read_dir(dir) else { return 0 };
        entries
            .flatten()
            .map(|e| if e.path().is_dir() { count(&e.path()) } else { 1 })
            .sum()
    }
    count(&root.join(".sprixel/trash"))
}

fn rename(sprite: SpriteRef, name: &str) -> Operation {
    Operation::RenameSprite {
        sprite,
        name: name.to_string(),
    }
}

// Checks the spec scenario "Rename with F2": after a flush the sprite file has
// the new name, the old file is gone, the content is the same, and the trash
// has nothing (a rename is not a delete).
#[test]
fn renaming_a_sprite_renames_its_file() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    let root = root_of(parent.path());
    let old = sprite_path(&root, &goblin_ref());
    let text = fs::read_to_string(&old).unwrap();

    session.apply("user", vec![rename(goblin_ref(), "orc")]).unwrap();
    assert_eq!(session.flush(), SaveStatus::Saved);

    let new = sprite_path(&root, &SpriteRef::new(&["Personajes"], "orc"));
    assert!(!old.exists());
    assert_eq!(fs::read_to_string(new).unwrap(), text);
    assert_eq!(trash_files(&root), 0);
}

// Checks that undoing a rename moves the file back.
#[test]
fn undoing_a_rename_moves_the_file_back() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    let root = root_of(parent.path());
    session.apply("user", vec![rename(goblin_ref(), "orc")]).unwrap();
    session.flush();

    session.undo().unwrap();
    session.flush();

    assert!(sprite_path(&root, &goblin_ref()).exists());
    assert!(!sprite_path(&root, &SpriteRef::new(&["Personajes"], "orc")).exists());
    assert_eq!(trash_files(&root), 0);
}

// Checks that a change made after the rename and before the save still reaches
// the renamed file (the pending save follows the sprite).
#[test]
fn a_change_before_the_save_goes_to_the_new_file() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    let root = root_of(parent.path());
    session.apply("user", vec![draw(0, 'R')]).unwrap();
    session.apply("user", vec![rename(goblin_ref(), "orc")]).unwrap();

    session.flush();

    let new = SpriteRef::new(&["Personajes"], "orc");
    let palette = session.project.palettes[0].clone();
    let saved = read_sprite(&root, &new, &palette).unwrap();
    assert_eq!(saved.layers[0].frames[0].pixels[0], b'R');
    assert!(!sprite_path(&root, &goblin_ref()).exists());
}

// Checks that renaming a sprite that was never saved just writes the new file.
#[test]
fn renaming_an_unsaved_sprite_writes_only_the_new_file() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    let root = root_of(parent.path());
    session.apply("user", vec![rename(goblin_ref(), "orc")]).unwrap();

    assert_eq!(session.flush(), SaveStatus::Saved);

    assert!(sprite_path(&root, &SpriteRef::new(&["Personajes"], "orc")).exists());
    assert!(!sprite_path(&root, &goblin_ref()).exists());
}

// Checks that a rename that only changes the case works on the disk too and
// leaves one file (this is the tricky case on Windows and macOS).
#[test]
fn a_rename_that_only_changes_the_case_leaves_one_file() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    let root = root_of(parent.path());

    session.apply("user", vec![rename(goblin_ref(), "Goblin")]).unwrap();
    assert_eq!(session.flush(), SaveStatus::Saved);

    let names: Vec<String> = fs::read_dir(root.join("sprites/Personajes"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["Goblin.sprite.json"]);
}

// Checks that renaming a folder moves the folder with the sprites inside it,
// and that nothing goes to the trash.
#[test]
fn renaming_a_folder_moves_its_files() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    let root = root_of(parent.path());

    session
        .apply(
            "user",
            vec![Operation::RenameFolder {
                path: vec!["Personajes".to_string()],
                name: "Heroes".to_string(),
            }],
        )
        .unwrap();
    assert_eq!(session.flush(), SaveStatus::Saved);

    assert!(!root.join("sprites/Personajes").exists());
    assert!(root.join("sprites/Heroes/goblin.sprite.json").is_file());
    assert_eq!(trash_files(&root), 0);

    session.undo().unwrap();
    session.flush();
    assert!(root.join("sprites/Personajes/goblin.sprite.json").is_file());
    assert!(!root.join("sprites/Heroes").exists());
}

// Checks that a sprite changed before a folder rename is saved inside the
// renamed folder, not in a new folder with the old name.
#[test]
fn a_change_before_a_folder_rename_goes_to_the_new_folder() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    let root = root_of(parent.path());
    session.apply("user", vec![draw(0, 'R')]).unwrap();
    session
        .apply(
            "user",
            vec![Operation::RenameFolder {
                path: vec!["Personajes".to_string()],
                name: "Heroes".to_string(),
            }],
        )
        .unwrap();

    session.flush();

    assert!(!root.join("sprites/Personajes").exists());
    let text = fs::read_to_string(root.join("sprites/Heroes/goblin.sprite.json")).unwrap();
    assert!(text.contains("R..."));
}

// Checks that a duplicate is a new file and the original stays.
#[test]
fn a_duplicate_writes_a_new_file() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    let root = root_of(parent.path());

    session
        .apply(
            "user",
            vec![Operation::DuplicateSprite {
                sprite: goblin_ref(),
                name: "goblin copy".to_string(),
            }],
        )
        .unwrap();
    session.flush();

    assert!(sprite_path(&root, &goblin_ref()).exists());
    assert!(root.join("sprites/Personajes/goblin copy.sprite.json").exists());
}

// Checks that moving a sprite to another folder moves its file there (and
// creates nothing in the trash), and that undo moves it back.
#[test]
fn moving_a_sprite_moves_its_file() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session
        .apply(
            "user",
            vec![Operation::CreateFolder {
                parent: vec![],
                name: "Enemigos".to_string(),
            }],
        )
        .unwrap();
    session.flush();
    let root = root_of(parent.path());
    let text = fs::read_to_string(sprite_path(&root, &goblin_ref())).unwrap();

    session
        .apply(
            "user",
            vec![Operation::MoveSprite {
                sprite: goblin_ref(),
                to: vec!["Enemigos".to_string()],
                index: None,
            }],
        )
        .unwrap();
    assert_eq!(session.flush(), SaveStatus::Saved);

    assert!(!sprite_path(&root, &goblin_ref()).exists());
    let moved = root.join("sprites/Enemigos/goblin.sprite.json");
    assert_eq!(fs::read_to_string(moved).unwrap(), text);
    assert_eq!(trash_files(&root), 0);

    session.undo().unwrap();
    session.flush();
    assert!(sprite_path(&root, &goblin_ref()).exists());
    assert!(!root.join("sprites/Enemigos/goblin.sprite.json").exists());
}
