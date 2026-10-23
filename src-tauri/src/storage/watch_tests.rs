use super::session_tests::{dungeon16, draw, goblin_ref, session_with_goblin};
use super::*;
use crate::document::{Operation, SpriteRef, Target};
use std::fs;
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Polls for outside changes until `done` is satisfied by the changes found
/// so far, or fails after a timeout. Events from the system arrive at a
/// random time, so there is no fixed sleep.
fn collect_changes(
    session: &mut Session,
    done: impl Fn(&[ExternalChange]) -> bool,
) -> Vec<ExternalChange> {
    let start = Instant::now();
    let mut all = Vec::new();
    while !done(&all) {
        assert!(start.elapsed() < Duration::from_secs(10), "no change seen: {all:?}");
        all.extend(session.poll_external());
        sleep(Duration::from_millis(20));
    }
    all
}

fn goblin_target() -> Target {
    Target::Sprite(goblin_ref())
}

fn saved_session(parent: &std::path::Path) -> Session {
    let mut session = session_with_goblin(parent);
    session.flush();
    session.watch().unwrap();
    session
}

/// The text of the goblin file as if another program had drawn `key` at (x, 0).
fn goblin_text_with(session: &Session, x: usize, key: u8) -> String {
    let mut sprite = session.project.root.folders[0].sprites[0].clone();
    sprite.layers[0].frames[0].pixels[x] = key;
    sprite_to_text(&sprite)
}

fn goblin_pixel(session: &Session, x: usize) -> u8 {
    session.project.root.folders[0].sprites[0].layers[0].frames[0].pixels[x]
}

// Checks the spec scenario (like a `git pull`): a sprite file changes on disk and
// the app has no pending changes for it, so the sprite is reloaded and shows the new pixels.
#[test]
fn an_outside_change_reloads_the_sprite() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = saved_session(parent.path());
    let revision = session.project.revision;
    let text = goblin_text_with(&session, 2, b'R');

    fs::write(sprite_path(session.root(), &goblin_ref()), text).unwrap();

    let changes = collect_changes(&mut session, |c| !c.is_empty());
    assert_eq!(changes, vec![ExternalChange::Reloaded(goblin_target())]);
    assert_eq!(goblin_pixel(&session, 2), b'R');
    assert!(session.project.revision > revision);
    assert!(session.history.entries().is_empty(), "old undo steps do not fit the new file");
}

// Checks that the app's own writes (autosave, trash moves) are not reported as outside changes.
#[test]
fn own_writes_are_ignored() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = saved_session(parent.path());
    // Own writes: a stroke is saved, a sprite is created and another is deleted.
    let orc = Operation::CreateSprite {
        folder: vec![],
        name: "orc".to_string(),
        width: 2,
        height: 2,
        palette: "dungeon16".to_string(),
    };
    session.apply("user", vec![draw(0, 'N'), orc]).unwrap();
    session.flush();
    session
        .apply("user", vec![Operation::DeleteSprite { sprite: goblin_ref() }])
        .unwrap();
    session.flush();
    // A real outside change comes after. Events keep their order, so when it
    // is seen, the events of our own writes have been seen too.
    let orc_ref = SpriteRef::new(&[], "orc");
    let mut sprite = session.project.find_sprite(&orc_ref).unwrap().clone();
    sprite.layers[0].frames[0].pixels[0] = b'P';
    fs::write(sprite_path(session.root(), &orc_ref), sprite_to_text(&sprite)).unwrap();

    let changes = collect_changes(&mut session, |c| !c.is_empty());

    assert_eq!(changes, vec![ExternalChange::Reloaded(Target::Sprite(orc_ref))]);
}

// Checks that with unsaved changes an outside change gives a notice (a conflict)
// and the app version is kept until the user chooses.
#[test]
fn an_outside_change_with_unsaved_changes_is_a_conflict() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = saved_session(parent.path());
    let text = goblin_text_with(&session, 3, b'R');
    session.apply("user", vec![draw(0, 'N')]).unwrap();
    fs::write(sprite_path(session.root(), &goblin_ref()), text).unwrap();

    let changes = collect_changes(&mut session, |c| !c.is_empty());

    assert_eq!(changes, vec![ExternalChange::Conflict(goblin_target())]);
    assert_eq!(goblin_pixel(&session, 0), b'N');
    assert_eq!(goblin_pixel(&session, 3), b'.');
}

// Checks the "reload" choice of a conflict: the file on disk wins and nothing is left to save.
#[test]
fn resolving_a_conflict_with_reload_takes_the_disk_version() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = saved_session(parent.path());
    let text = goblin_text_with(&session, 3, b'R');
    session.apply("user", vec![draw(0, 'N')]).unwrap();
    fs::write(sprite_path(session.root(), &goblin_ref()), &text).unwrap();
    collect_changes(&mut session, |c| !c.is_empty());

    let change = session.resolve_conflict(&goblin_target(), ConflictChoice::Reload);

    assert_eq!(change, Some(ExternalChange::Reloaded(goblin_target())));
    assert_eq!(goblin_pixel(&session, 3), b'R');
    assert_eq!(goblin_pixel(&session, 0), b'.');
    assert_eq!(session.flush(), SaveStatus::Saved);
    assert_eq!(fs::read_to_string(sprite_path(session.root(), &goblin_ref())).unwrap(), text);
}

// Checks the "keep" choice of a conflict: the app version is written over the file on disk.
#[test]
fn resolving_a_conflict_with_keep_writes_the_app_version() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = saved_session(parent.path());
    let text = goblin_text_with(&session, 3, b'R');
    session.apply("user", vec![draw(0, 'N')]).unwrap();
    fs::write(sprite_path(session.root(), &goblin_ref()), text).unwrap();
    collect_changes(&mut session, |c| !c.is_empty());

    session.resolve_conflict(&goblin_target(), ConflictChoice::Keep);
    session.flush();

    let on_disk = read_sprite(session.root(), &goblin_ref(), &dungeon16()).unwrap();
    assert_eq!(on_disk.layers[0].frames[0].pixels[0], b'N');
    assert_eq!(on_disk.layers[0].frames[0].pixels[3], b'.');
}

// Checks that a sprite file deleted outside (a checkout) is removed from the project.
#[test]
fn an_outside_delete_removes_the_sprite() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = saved_session(parent.path());

    fs::remove_file(sprite_path(session.root(), &goblin_ref())).unwrap();

    let changes = collect_changes(&mut session, |c| !c.is_empty());
    assert_eq!(changes, vec![ExternalChange::Removed(goblin_target())]);
    assert!(session.project.find_sprite(&goblin_ref()).is_none());
}

// Checks that a new sprite file added outside, in a new folder, appears in the project with its folder.
#[test]
fn an_outside_new_file_is_added() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = saved_session(parent.path());
    let mut sprite = session.project.root.folders[0].sprites[0].clone();
    sprite.name = "slime".to_string();
    let slime = SpriteRef::new(&["Enemigos"], "slime");

    write_sprite(&DiskWriter, session.root(), &slime, &sprite).unwrap();

    let changes = collect_changes(&mut session, |c| !c.is_empty());
    assert_eq!(changes, vec![ExternalChange::Reloaded(Target::Sprite(slime.clone()))]);
    assert!(session.project.find_sprite(&slime).is_some());
}

// Checks that a file that became invalid outside is reported as damaged with its path,
// and the sprite in memory is kept.
#[test]
fn an_outside_broken_file_is_reported_and_the_sprite_is_kept() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = saved_session(parent.path());
    let path = sprite_path(session.root(), &goblin_ref());

    fs::write(&path, "{ not json").unwrap();

    let changes = collect_changes(&mut session, |c| !c.is_empty());
    let [ExternalChange::Damaged { path: damaged, .. }] = changes.as_slice() else {
        panic!("expected one damaged file, got {changes:?}");
    };
    assert_eq!(damaged, &path);
    assert!(session.project.find_sprite(&goblin_ref()).is_some());
}

// Checks that a palette changed outside is reloaded, unless a sprite would break.
#[test]
fn an_outside_palette_change_is_checked_against_the_sprites() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = saved_session(parent.path());
    let path = palette_path(session.root(), "dungeon16");

    // A palette that adds one more color is fine.
    let mut bigger = dungeon16();
    bigger.colors.push(crate::document::Color {
        key: 'Z',
        name: "extra".to_string(),
        hex: "#000000".to_string(),
    });
    fs::write(&path, palette_to_text(&bigger)).unwrap();
    let changes = collect_changes(&mut session, |c| !c.is_empty());
    assert_eq!(changes, vec![ExternalChange::Reloaded(Target::Palette("dungeon16".to_string()))]);
    assert_eq!(session.project.palettes[0].colors.len(), 5);

    // Draw with the new color, then another program removes it: the sprite would break.
    session.apply("user", vec![draw(0, 'Z')]).unwrap();
    session.flush();
    fs::write(&path, palette_to_text(&dungeon16())).unwrap();
    let changes = collect_changes(&mut session, |c| !c.is_empty());
    assert!(matches!(changes.as_slice(), [ExternalChange::Damaged { .. }]), "{changes:?}");
    assert_eq!(session.project.palettes[0].colors.len(), 5);
}
