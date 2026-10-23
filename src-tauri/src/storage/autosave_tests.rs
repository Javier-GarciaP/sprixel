use super::session_tests::{draw, goblin_ref, session_with_goblin};
use super::*;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Waits until the condition is true, or fails after the timeout.
fn wait_until(timeout: Duration, mut condition: impl FnMut() -> bool) {
    let start = Instant::now();
    while !condition() {
        assert!(start.elapsed() < timeout, "timed out");
        sleep(Duration::from_millis(5));
    }
}

fn pixels_on_disk(root: &Path) -> Vec<u8> {
    let sprite = read_sprite(root, &goblin_ref(), &super::session_tests::dungeon16()).unwrap();
    sprite.layers[0].frames[0].pixels.clone()
}

// Checks the spec scenario: after a stroke and one second of waiting, the file
// on disk has the stroke and the status is "saved".
#[test]
fn draw_and_wait_one_second() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("Kit");
    let autosave = Autosave::start(session_with_goblin(parent.path()), DEFAULT_DEBOUNCE);

    autosave.edit(|s| s.apply("user", vec![draw(1, 'N')]).unwrap());
    assert_eq!(autosave.status(), SaveStatus::Saving);
    wait_until(Duration::from_secs(1), || autosave.status() == SaveStatus::Saved);

    assert_eq!(pixels_on_disk(&root)[1], b'N');
}

/// A writer that counts how many times each sprite file is written.
struct CountingWriter(Arc<AtomicUsize>);

impl FileWriter for CountingWriter {
    fn write(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        if path.to_string_lossy().ends_with("goblin.sprite.json.tmp") {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
        DiskWriter.write(path, bytes)
    }
}

// Checks that many quick changes become one save after the quiet time (debounce).
#[test]
fn quick_changes_are_saved_once() {
    let parent = tempfile::tempdir().unwrap();
    let writes = Arc::new(AtomicUsize::new(0));
    let mut session = session_with_goblin(parent.path());
    session.set_writer(Arc::new(CountingWriter(writes.clone())));
    let autosave = Autosave::start(session, Duration::from_millis(500));

    for x in 0..4 {
        autosave.edit(|s| s.apply("user", vec![draw(x, 'N')]).unwrap());
    }
    assert_eq!(writes.load(Ordering::SeqCst), 0);
    wait_until(Duration::from_secs(5), || autosave.status() == SaveStatus::Saved);

    assert_eq!(writes.load(Ordering::SeqCst), 1);
    assert_eq!(&pixels_on_disk(&parent.path().join("Kit"))[..4], b"NNNN");
}

// Checks that closing the autosave (the app closing) saves what is pending at once,
// without waiting for the quiet time.
#[test]
fn dropping_the_autosave_does_a_final_flush() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("Kit");
    let autosave = Autosave::start(session_with_goblin(parent.path()), Duration::from_secs(60));
    autosave.edit(|s| s.apply("user", vec![draw(2, 'P')]).unwrap());
    assert!(!root.join("sprites/Personajes/goblin.sprite.json").exists());

    drop(autosave);

    assert_eq!(pixels_on_disk(&root)[2], b'P');
}

// Checks that with nothing changed the autosave writes nothing and stays "saved".
#[test]
fn autosave_without_changes_writes_nothing() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.flush();
    let autosave = Autosave::start(session, Duration::from_millis(10));
    sleep(Duration::from_millis(50));
    assert_eq!(autosave.status(), SaveStatus::Saved);
    drop(autosave);
    let sprites = parent.path().join("Kit/sprites/Personajes");
    assert_eq!(fs::read_dir(sprites).unwrap().count(), 1);
}
