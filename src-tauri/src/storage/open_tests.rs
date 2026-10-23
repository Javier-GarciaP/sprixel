use super::migrate_tests::{old_project, test_migrator};
use super::session_tests::{dungeon16, draw, goblin_ref, session_with_goblin};
use super::*;
use crate::document::{Operation, SpriteRef, Target};
use std::fs;
use std::path::Path;

/// A saved project "Kit" with the goblin sprite (and a second sprite "orc"
/// in the root), closed. Returns the project folder.
fn saved_project(parent: &Path) -> std::path::PathBuf {
    let mut session = session_with_goblin(parent);
    session.apply("user", vec![draw(1, 'N')]).unwrap();
    let orc = Operation::CreateSprite {
        folder: vec![],
        name: "orc".to_string(),
        width: 2,
        height: 2,
        palette: "dungeon16".to_string(),
    };
    let empty = Operation::CreateFolder {
        parent: vec![],
        name: "Vacia".to_string(),
    };
    session.apply("user", vec![orc, empty]).unwrap();
    assert_eq!(session.flush(), SaveStatus::Saved);
    session.root().to_path_buf()
}

fn orc_ref() -> SpriteRef {
    SpriteRef::new(&[], "orc")
}

// Checks that a saved project opens with the same content, including empty folders.
#[test]
fn a_saved_project_opens_with_the_same_content() {
    let parent = tempfile::tempdir().unwrap();
    let root = saved_project(parent.path());
    let original = session_content(&Session::open(&root).unwrap());

    let reopened = Session::open(&root).unwrap();

    assert_eq!(session_content(&reopened), original);
    assert!(reopened.damaged().is_empty());
    assert!(reopened.migration().is_none());
    assert_eq!(reopened.project.name, "Kit");
    assert_eq!(reopened.project.palettes.len(), 1);
    assert_eq!(reopened.project.root.folders.len(), 2);
    assert_eq!(reopened.status(), SaveStatus::Saved);
}

fn session_content(session: &Session) -> crate::document::Project {
    session.project.clone()
}

// Checks that a project opened from disk can be edited and saved like a new one.
#[test]
fn an_opened_project_can_be_edited_and_saved() {
    let parent = tempfile::tempdir().unwrap();
    let root = saved_project(parent.path());
    let mut session = Session::open(&root).unwrap();

    session.apply("user", vec![draw(3, 'R')]).unwrap();
    session.flush();

    let on_disk = read_sprite(&root, &goblin_ref(), &dungeon16()).unwrap();
    assert_eq!(on_disk.layers[0].frames[0].pixels[3], b'R');
    assert_eq!(on_disk.layers[0].frames[0].pixels[1], b'N');
}

// Checks the spec scenario: a sprite file with invalid JSON does not stop the project from
// opening; it is listed as damaged with its path and the error; the other sprites load.
#[test]
fn a_project_with_one_broken_sprite_still_opens() {
    let parent = tempfile::tempdir().unwrap();
    let root = saved_project(parent.path());
    let broken = sprite_path(&root, &goblin_ref());
    fs::write(&broken, "{ \"formatVersion\": 1, oops").unwrap();

    let session = Session::open(&root).unwrap();

    assert_eq!(session.damaged().len(), 1);
    assert_eq!(session.damaged()[0].path, broken);
    assert!(!session.damaged()[0].reason.is_empty());
    assert!(session.project.find_sprite(&goblin_ref()).is_none());
    assert!(session.project.find_sprite(&orc_ref()).is_some());
}

// Checks that a file that is valid JSON but breaks a rule (a row of the wrong length,
// a key outside the palette) is damaged too, with the reason.
#[test]
fn a_sprite_that_breaks_a_rule_is_damaged() {
    let parent = tempfile::tempdir().unwrap();
    let root = saved_project(parent.path());
    let path = sprite_path(&root, &orc_ref());
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, text.replacen("\"..\"", "\".Z\"", 1)).unwrap();

    let session = Session::open(&root).unwrap();

    assert_eq!(session.damaged().len(), 1);
    assert!(session.damaged()[0].reason.contains("unknown_key"), "{:?}", session.damaged());
    assert!(session.project.find_sprite(&goblin_ref()).is_some());
}

// Checks that a broken palette is damaged, the sprites that use it are damaged
// (their palette is missing), and the project still opens.
#[test]
fn a_broken_palette_damages_its_sprites_only() {
    let parent = tempfile::tempdir().unwrap();
    let root = saved_project(parent.path());
    fs::write(palette_path(&root, "dungeon16"), "not json").unwrap();

    let session = Session::open(&root).unwrap();

    assert_eq!(session.damaged().len(), 3, "{:?}", session.damaged());
    assert!(session.project.palettes.is_empty());
    assert!(session.project.root.sprites.is_empty());
    // The empty folder is still there.
    assert!(session.project.find_folder(&["Vacia".to_string()]).is_some());
}

// Checks that a folder whose name is not valid is listed as damaged, not loaded.
#[test]
fn a_folder_with_an_invalid_name_is_damaged() {
    let parent = tempfile::tempdir().unwrap();
    let root = saved_project(parent.path());
    fs::create_dir(root.join("sprites").join("a".repeat(201))).unwrap();

    let session = Session::open(&root).unwrap();

    assert_eq!(session.damaged().len(), 1);
    assert!(session.damaged()[0].reason.contains("too long"), "{:?}", session.damaged());
}

// Checks that two sprite files whose names differ only in case are not both loaded.
// (Skipped where the file system cannot hold both files, like Windows and macOS.)
#[test]
fn names_that_differ_only_in_case_are_not_loaded_twice() {
    let parent = tempfile::tempdir().unwrap();
    let root = saved_project(parent.path());
    let original = sprite_path(&root, &orc_ref());
    let twin = original.with_file_name("ORC.sprite.json");
    // Where file names ignore case, the twin is the same file as the original.
    // Copying a file onto itself fails on Windows, so skip before copying.
    if twin.exists() {
        return;
    }
    fs::copy(&original, &twin).unwrap();
    if fs::read_dir(root.join("sprites")).unwrap().filter(|e| {
        e.as_ref().unwrap().file_name().to_string_lossy().to_lowercase() == "orc.sprite.json"
    }).count() < 2 {
        return;
    }

    let session = Session::open(&root).unwrap();

    assert_eq!(session.damaged().len(), 1);
    assert!(session.damaged()[0].reason.contains("name_exists"));
}

// Checks that a leftover temporary file and the `.sprixel` folder are not treated as project content.
#[test]
fn temporary_files_and_internal_data_are_ignored() {
    let parent = tempfile::tempdir().unwrap();
    let root = saved_project(parent.path());
    fs::write(root.join("sprites/orc.sprite.json.tmp"), "half a fi").unwrap();
    fs::write(root.join("sprites/notes.txt"), "hello").unwrap();
    fs::create_dir_all(root.join(".sprixel/trash/x")).unwrap();
    fs::write(root.join(".sprixel/trash/x/old.sprite.json"), "{").unwrap();

    let session = Session::open(&root).unwrap();

    assert!(session.damaged().is_empty());
}

// Checks that saving other changes does not touch a damaged file: the user can still repair it.
#[test]
fn saving_does_not_touch_damaged_files() {
    let parent = tempfile::tempdir().unwrap();
    let root = saved_project(parent.path());
    let broken = sprite_path(&root, &goblin_ref());
    fs::write(&broken, "{ broken").unwrap();
    let mut session = Session::open(&root).unwrap();

    session.apply("user", vec![Operation::SetPixels {
        sprite: orc_ref(),
        layer: 0,
        frame: 1,
        pixels: vec![crate::document::PixelChange { x: 0, y: 0, key: 'N' }],
    }]).unwrap();
    session.flush();

    assert_eq!(fs::read_to_string(&broken).unwrap(), "{ broken");
}

// Checks that a folder that is not a project cannot be opened, with an error naming the manifest.
#[test]
fn a_folder_without_a_manifest_does_not_open() {
    let parent = tempfile::tempdir().unwrap();
    let error = Session::open(parent.path()).err().unwrap();
    assert!(error.to_string().contains("sprixel.json"), "{error}");
}

// Checks that a project saved by a newer Sprixel is refused and its files are not modified.
#[test]
fn a_newer_project_is_refused() {
    let parent = tempfile::tempdir().unwrap();
    let root = saved_project(parent.path());
    let manifest = root.join("sprixel.json");
    let newer = fs::read_to_string(&manifest).unwrap().replace("\"formatVersion\": 1", "\"formatVersion\": 9");
    fs::write(&manifest, &newer).unwrap();

    let error = Session::open(&root).err().unwrap();

    assert_eq!(error, StorageError::NewerVersion { found: 9, current: 1 });
    assert_eq!(fs::read_to_string(&manifest).unwrap(), newer);
    assert!(!root.join(".sprixel/backups").exists());
}

// Checks the spec scenario: an older project is backed up, migrated and opens normally.
#[test]
fn an_older_project_is_migrated_and_opens() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("Old Kit");
    old_project(&root);

    let session = Session::open_with(&root, &test_migrator()).unwrap();

    let migration = session.migration().unwrap();
    assert!(migration.backup.join("sprixel.json").is_file());
    assert_eq!(session.project.name, "Old Kit");
    assert!(session.damaged().is_empty(), "{:?}", session.damaged());
    let hero = SpriteRef::new(&["Personajes"], "hero");
    assert_eq!(session.project.find_sprite(&hero).unwrap().tags[0].name, "idle");
}

// Checks that files read at open are known, so rewriting a file with the same
// content is not reported as an outside change.
#[test]
fn files_read_at_open_are_not_reported_as_changes() {
    let parent = tempfile::tempdir().unwrap();
    let root = saved_project(parent.path());
    let mut session = Session::open(&root).unwrap();
    session.watch().unwrap();
    let goblin_file = sprite_path(&root, &goblin_ref());
    fs::write(&goblin_file, fs::read(&goblin_file).unwrap()).unwrap();
    // A real change comes after the harmless rewrite.
    let orc_file = sprite_path(&root, &orc_ref());
    let mut orc = session.project.find_sprite(&orc_ref()).unwrap().clone();
    orc.layers[0].frames[0].pixels[0] = b'A';
    fs::write(&orc_file, sprite_to_text(&orc)).unwrap();

    let start = std::time::Instant::now();
    let mut changes = Vec::new();
    while changes.is_empty() {
        assert!(start.elapsed() < std::time::Duration::from_secs(10), "no change seen");
        changes.extend(session.poll_external());
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    assert_eq!(changes, vec![ExternalChange::Reloaded(Target::Sprite(orc_ref()))]);
}
