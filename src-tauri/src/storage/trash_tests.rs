use super::session_tests::{draw, goblin_ref, session_with_goblin};
use super::*;
use crate::document::{Operation, SpriteRef};
use std::fs;
use std::path::{Path, PathBuf};

/// Every file below the trash folder of a project.
fn trash_files(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, found);
            } else {
                found.push(path);
            }
        }
    }
    let mut found = Vec::new();
    walk(&root.join(".sprixel/trash"), &mut found);
    found
}

fn delete_goblin() -> Operation {
    Operation::DeleteSprite {
        sprite: goblin_ref(),
    }
}

// Checks the spec scenario: a deleted sprite is moved to `.sprixel/trash/` (not erased),
// and Undo puts the same file back in its original folder with the same content.
#[test]
fn delete_a_sprite_and_undo() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.apply("user", vec![draw(1, 'N')]).unwrap();
    session.flush();
    let root = session.root().to_path_buf();
    let file = sprite_path(&root, &goblin_ref());
    let before = fs::read(&file).unwrap();

    session.apply("user", vec![delete_goblin()]).unwrap();
    assert_eq!(session.flush(), SaveStatus::Saved);
    assert!(!file.exists());
    let trashed = trash_files(&root);
    assert_eq!(trashed.len(), 1);
    assert_eq!(fs::read(&trashed[0]).unwrap(), before);

    session.undo().unwrap();
    assert_eq!(session.flush(), SaveStatus::Saved);
    assert_eq!(fs::read(&file).unwrap(), before);
    assert!(trash_files(&root).is_empty(), "the file was moved back, not copied");
}

// Checks that redoing the delete moves the file to the trash again.
#[test]
fn redo_a_delete_trashes_the_file_again() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    let root = session.root().to_path_buf();
    session.apply("user", vec![delete_goblin()]).unwrap();
    session.flush();
    session.undo().unwrap();
    session.flush();

    session.redo().unwrap();
    session.flush();

    assert!(!sprite_path(&root, &goblin_ref()).exists());
    assert_eq!(trash_files(&root).len(), 1);
}

// Checks that deleting a folder moves the folder with all its sprites, and Undo brings them back.
#[test]
fn delete_a_folder_and_undo() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    let orc = Operation::CreateSprite {
        folder: vec!["Personajes".to_string()],
        name: "orc".to_string(),
        width: 2,
        height: 2,
        palette: "dungeon16".to_string(),
    };
    session.apply("user", vec![orc]).unwrap();
    session.flush();
    let root = session.root().to_path_buf();
    let goblin_before = fs::read(sprite_path(&root, &goblin_ref())).unwrap();
    let orc_ref = SpriteRef::new(&["Personajes"], "orc");
    let orc_before = fs::read(sprite_path(&root, &orc_ref)).unwrap();

    let delete = Operation::DeleteFolder {
        path: vec!["Personajes".to_string()],
    };
    session.apply("user", vec![delete]).unwrap();
    session.flush();
    assert!(!root.join("sprites/Personajes").exists());
    assert_eq!(trash_files(&root).len(), 2);

    session.undo().unwrap();
    assert_eq!(session.flush(), SaveStatus::Saved);
    assert_eq!(fs::read(sprite_path(&root, &goblin_ref())).unwrap(), goblin_before);
    assert_eq!(fs::read(sprite_path(&root, &orc_ref)).unwrap(), orc_before);
    assert!(trash_files(&root).is_empty());
}

// Checks that deleting something that was never saved works and puts nothing in the trash.
#[test]
fn deleting_an_unsaved_sprite_is_not_an_error() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.apply("user", vec![delete_goblin()]).unwrap();

    assert_eq!(session.flush(), SaveStatus::Saved);

    assert!(trash_files(session.root()).is_empty());
}

// Checks that a new sprite with the name of a deleted one is saved as new
// content, and the deleted one stays in the trash.
#[test]
fn a_new_sprite_with_a_deleted_name_does_not_bring_the_old_one_back() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.apply("user", vec![draw(0, 'N')]).unwrap();
    session.flush();
    let root = session.root().to_path_buf();
    let old = fs::read(sprite_path(&root, &goblin_ref())).unwrap();
    session.apply("user", vec![delete_goblin()]).unwrap();
    session.flush();

    let again = Operation::CreateSprite {
        folder: vec!["Personajes".to_string()],
        name: "goblin".to_string(),
        width: 4,
        height: 4,
        palette: "dungeon16".to_string(),
    };
    session.apply("user", vec![again]).unwrap();
    session.flush();

    let new = fs::read(sprite_path(&root, &goblin_ref())).unwrap();
    assert_ne!(new, old);
    let trashed = trash_files(&root);
    assert_eq!(trashed.len(), 1);
    assert_eq!(fs::read(&trashed[0]).unwrap(), old);
}

// Checks that a new empty folder with the name of a deleted folder does not bring its files back.
#[test]
fn a_new_folder_with_a_deleted_name_stays_empty() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    let root = session.root().to_path_buf();
    let delete = Operation::DeleteFolder {
        path: vec!["Personajes".to_string()],
    };
    session.apply("user", vec![delete]).unwrap();
    session.flush();

    let create = Operation::CreateFolder {
        parent: vec![],
        name: "Personajes".to_string(),
    };
    session.apply("user", vec![create]).unwrap();
    session.flush();

    assert!(root.join("sprites/Personajes").is_dir());
    assert_eq!(fs::read_dir(root.join("sprites/Personajes")).unwrap().count(), 0);
    assert_eq!(trash_files(&root).len(), 1);
}

// Checks that a deleted palette goes to the trash too and Undo restores it.
#[test]
fn delete_a_palette_and_undo() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = Session::create(parent.path(), "Kit").unwrap();
    let create = Operation::CreatePalette {
        palette: super::session_tests::dungeon16(),
        index: None,
    };
    session.apply("user", vec![create]).unwrap();
    session.flush();
    let root = session.root().to_path_buf();
    let file = palette_path(&root, "dungeon16");
    let before = fs::read(&file).unwrap();

    let delete = Operation::DeletePalette {
        name: "dungeon16".to_string(),
    };
    session.apply("user", vec![delete]).unwrap();
    session.flush();
    assert!(!file.exists());
    assert_eq!(trash_files(&root).len(), 1);

    session.undo().unwrap();
    session.flush();
    assert_eq!(fs::read(&file).unwrap(), before);
}
