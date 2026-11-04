use super::*;
use std::fs;
use std::path::{Path, PathBuf};

// Copies a fixture project from `testdata` into a temp folder, so a test can
// open it (and save it) without touching the fixture.
fn fixture(name: &str, into: &Path) -> PathBuf {
    let from = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/storage/testdata").join(name);
    let to = into.join(name);
    copy(&from, &to);
    to
}

fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy(&entry.path(), &target);
        } else {
            fs::write(target, fs::read_to_string(entry.path()).unwrap()).unwrap();
        }
    }
}

// Checks that the first files ever written (no `formatVersion`, no
// `frameDurations`, no `anchors`) open as version 1 and lose nothing.
#[test]
fn files_without_a_version_open_as_version_1() {
    let dir = tempfile::tempdir().unwrap();
    let root = fixture("old-kit-no-version", dir.path());

    let session = Session::open(&root).unwrap();

    assert!(session.damaged().is_empty(), "{:?}", session.damaged());
    assert!(session.migration().is_none());
    let sprite = session.project.find_sprite(&crate::document::SpriteRef::new(&["Hero"], "walker")).unwrap();
    assert_eq!(sprite.frame_durations, vec![100, 100]);
    assert!(sprite.anchors.is_empty());
    assert_eq!(sprite.tags.len(), 1);
    assert_eq!(sprite.layers[0].frames[1].pixels, b".RR.");
}

// Checks that a project saved by version 1 before anchors existed opens and
// keeps its durations (anchors are an optional field).
#[test]
fn a_version_1_file_without_anchors_opens() {
    let dir = tempfile::tempdir().unwrap();
    let root = fixture("old-kit-v1-no-anchors", dir.path());

    let session = Session::open(&root).unwrap();

    assert!(session.damaged().is_empty(), "{:?}", session.damaged());
    let sprite = session.project.find_sprite(&crate::document::SpriteRef::new(&[], "blob")).unwrap();
    assert_eq!(sprite.frame_durations, vec![100, 250]);
    assert!(sprite.anchors.is_empty());
    assert!(sprite.parts.is_empty(), "parts are optional: old files have none");
}

// Checks that the listing (used by the project browser) also reads files
// without a version or without frame durations.
#[test]
fn the_listing_reads_old_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = fixture("old-kit-no-version", dir.path());

    let listing = list_project(&root).unwrap();

    assert!(listing.damaged.is_empty(), "{:?}", listing.damaged);
    assert_eq!(listing.sprites.len(), 1);
    assert_eq!(listing.sprites[0].frames, 2);
}

// Checks that a file without a version is written with `formatVersion` on the
// next save, and the saved project opens again with the same content.
#[test]
fn an_old_file_is_upgraded_on_the_next_save() {
    let dir = tempfile::tempdir().unwrap();
    let root = fixture("old-kit-no-version", dir.path());
    let mut session = Session::open(&root).unwrap();
    let target = crate::document::SpriteRef::new(&["Hero"], "walker");
    let sprite = session.project.find_sprite(&target).unwrap().clone();
    let op = crate::document::Operation::SetFrameDuration { sprite: target.clone(), frame: 1, duration_ms: 200 };
    session.apply("user", vec![op]).unwrap();
    assert_eq!(session.flush(), SaveStatus::Saved);

    let text = fs::read_to_string(root.join("sprites/Hero/walker.sprite.json")).unwrap();
    assert!(text.contains("\"formatVersion\": 1"), "{text}");
    assert!(text.contains("\"frameDurations\""), "{text}");
    let again = Session::open(&root).unwrap();
    let saved = again.project.find_sprite(&target).unwrap();
    assert_eq!(saved.frame_durations, vec![200, 100]);
    assert_eq!(saved.layers, sprite.layers);
    assert_eq!(saved.tags, sprite.tags);
}

// Checks that a version that is not a number is a damaged manifest, not "version 1".
#[test]
fn a_version_that_is_not_a_number_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let root = fixture("old-kit-no-version", dir.path());
    fs::write(root.join("sprixel.json"), r#"{ "formatVersion": "two", "name": "x" }"#).unwrap();

    let error = Session::open(&root).err().unwrap();

    assert!(error.to_string().contains("formatVersion"), "{error}");
}
