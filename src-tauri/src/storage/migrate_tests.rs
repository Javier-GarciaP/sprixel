use super::*;
use crate::document::{Palette, SpriteRef};
use serde_json::Value;
use std::fs;
use std::path::Path;

// A made-up "version 0" to test the mechanism: the manifest called its name
// "title", palette colors called their hex "color" and tags used "start" and
// "end". The real app has no older version yet, so its list of steps is empty.
fn rename(object: &mut Value, from: &str, to: &str) -> Result<(), String> {
    let map = object.as_object_mut().ok_or("not an object")?;
    let value = map.remove(from).ok_or(format!("missing {from}"))?;
    map.insert(to.to_string(), value);
    Ok(())
}

fn version_0_to_1(value: &mut Value, kind: FileKind) -> Result<(), String> {
    match kind {
        FileKind::Manifest => rename(value, "title", "name"),
        FileKind::Palette => {
            for color in value["colors"].as_array_mut().ok_or("no colors")? {
                rename(color, "color", "hex")?;
            }
            Ok(())
        }
        FileKind::Sprite => {
            for tag in value["tags"].as_array_mut().ok_or("no tags")? {
                rename(tag, "start", "from")?;
                rename(tag, "end", "to")?;
            }
            Ok(())
        }
    }
}

pub(super) fn test_migrator() -> Migrator {
    Migrator::new(vec![Step {
        from: 0,
        run: version_0_to_1,
    }])
}

pub(super) const MANIFEST_V0: &str = r##"{ "formatVersion": 0, "title": "Old Kit" }"##;
const PALETTE_V0: &str = r##"{ "formatVersion": 0, "colors": [ { "key": "N", "name": "black", "color": "#000000" } ] }"##;
const SPRITE_V0: &str = r##"{ "formatVersion": 0, "width": 2, "height": 1, "palette": "dungeon16",
  "frameDurations": [100], "layers": [ { "name": "Layer 1", "frames": [ [ "N." ] ] } ],
  "tags": [ { "name": "idle", "start": 1, "end": 1, "fps": 12, "mode": "loop" } ] }"##;

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

pub(super) fn old_project(root: &Path) {
    write(root, "sprixel.json", MANIFEST_V0);
    write(root, "palettes/dungeon16.palette.json", PALETTE_V0);
    write(root, "sprites/Personajes/hero.sprite.json", SPRITE_V0);
}

fn backups(root: &Path) -> Vec<std::path::PathBuf> {
    match fs::read_dir(root.join(".sprixel/backups")) {
        Ok(entries) => entries.map(|e| e.unwrap().path()).collect(),
        Err(_) => vec![],
    }
}

// Checks that a project already in the current version is not touched and gets no backup.
#[test]
fn a_current_project_is_not_migrated() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_manifest(&DiskWriter, root, &Manifest { name: "Kit".into(), active_palette: None }).unwrap();
    let before = fs::read_to_string(root.join("sprixel.json")).unwrap();

    assert_eq!(migrate_project(root, &test_migrator()).unwrap(), None);

    assert_eq!(fs::read_to_string(root.join("sprixel.json")).unwrap(), before);
    assert!(backups(root).is_empty());
}

// Checks the older-project path: a backup of the old files is made first,
// then every file is migrated and the project reads normally.
#[test]
fn an_older_project_is_backed_up_and_migrated() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    old_project(root);

    let migration = migrate_project(root, &test_migrator()).unwrap().unwrap();

    assert_eq!(migration.from_version, 0);
    assert_eq!(backups(root), vec![migration.backup.clone()]);
    assert!(migration.backup.starts_with(root.join(".sprixel/backups")));
    let backed_up = fs::read_to_string(migration.backup.join("sprixel.json")).unwrap();
    assert_eq!(backed_up, MANIFEST_V0);
    assert!(migration.backup.join("palettes/dungeon16.palette.json").is_file());
    assert!(migration.backup.join("sprites/Personajes/hero.sprite.json").is_file());

    assert_eq!(read_manifest(root).unwrap().name, "Old Kit");
    let palette: Palette = read_palette(&palette_path(root, "dungeon16")).unwrap();
    assert_eq!(palette.colors[0].hex, "#000000");
    let hero = read_sprite(root, &SpriteRef::new(&["Personajes"], "hero"), &palette).unwrap();
    assert_eq!((hero.tags[0].from, hero.tags[0].to), (1, 1));
}

// Checks that migrating a second time does nothing (no second backup).
#[test]
fn migrating_twice_is_safe() {
    let dir = tempfile::tempdir().unwrap();
    old_project(dir.path());
    migrate_project(dir.path(), &test_migrator()).unwrap();
    assert_eq!(migrate_project(dir.path(), &test_migrator()).unwrap(), None);
    assert_eq!(backups(dir.path()).len(), 1);
}

// Checks that a project with a newer version is refused, not modified, gets no backup,
// and the message asks to update the app.
#[test]
fn a_newer_project_is_refused_and_not_modified() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let newer = r##"{ "formatVersion": 2, "name": "Future" }"##;
    write(root, "sprixel.json", newer);

    let error = migrate_project(root, &test_migrator()).unwrap_err();

    assert_eq!(error, StorageError::NewerVersion { found: 2, current: 1 });
    assert!(error.to_string().contains("Update Sprixel"));
    assert_eq!(fs::read_to_string(root.join("sprixel.json")).unwrap(), newer);
    assert!(backups(root).is_empty());
}

// Checks that a damaged file does not stop the migration of the other files,
// and stays untouched so it is reported as damaged later.
#[test]
fn a_broken_file_does_not_stop_the_migration() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    old_project(root);
    write(root, "sprites/broken.sprite.json", "{ not json");

    migrate_project(root, &test_migrator()).unwrap();

    assert_eq!(
        fs::read_to_string(root.join("sprites/broken.sprite.json")).unwrap(),
        "{ not json"
    );
    assert_eq!(read_manifest(root).unwrap().name, "Old Kit");
}

// Checks that an older project with no migration step for its version fails
// clearly and keeps its files.
#[test]
fn an_older_project_without_a_migration_step_fails() {
    let dir = tempfile::tempdir().unwrap();
    old_project(dir.path());

    let error = migrate_project(dir.path(), &Migrator::app()).unwrap_err();

    assert!(error.to_string().contains("no migration"), "{error}");
    let manifest = fs::read_to_string(dir.path().join("sprixel.json")).unwrap();
    assert_eq!(manifest, MANIFEST_V0);
}

// Checks that reading one file with the wrong version gives a clear reason.
#[test]
fn reading_a_file_with_another_version_explains_why() {
    let newer = manifest_from_text(r##"{ "formatVersion": 7, "name": "x" }"##).unwrap_err();
    assert!(newer.contains("newer Sprixel"), "{newer}");
    let older = manifest_from_text(MANIFEST_V0).unwrap_err();
    assert!(older.contains("migration"), "{older}");
    let none = manifest_from_text(r##"{ "name": "x" }"##).unwrap_err();
    assert!(none.contains("formatVersion"), "{none}");
}
