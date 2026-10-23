use super::session_tests::{dungeon16, draw, goblin_ref, session_with_goblin};
use super::*;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// A disk that can be switched to "full". While it is full, every write fails.
#[derive(Clone)]
struct FullDisk(Arc<AtomicBool>);

impl FullDisk {
    fn new(full: bool) -> FullDisk {
        FullDisk(Arc::new(AtomicBool::new(full)))
    }

    fn set_full(&self, full: bool) {
        self.0.store(full, Ordering::SeqCst);
    }
}

impl FileWriter for FullDisk {
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        if self.0.load(Ordering::SeqCst) {
            return Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "No space left on device",
            ));
        }
        DiskWriter.write(path, bytes)
    }
}

// Checks the spec scenario: when the disk is full the status shows the error with
// the reason, and the changes stay in the editor (in memory).
#[test]
fn a_full_disk_gives_an_error_and_keeps_the_data() {
    let parent = tempfile::tempdir().unwrap();
    let mut session = session_with_goblin(parent.path());
    session.set_writer(Arc::new(FullDisk::new(true)));
    session.apply("user", vec![draw(0, 'N')]).unwrap();

    let status = session.flush();

    let SaveStatus::Error(reason) = status else {
        panic!("expected an error status, got {status:?}");
    };
    assert!(reason.contains("No space left on device"), "{reason}");
    assert_eq!(session.status(), SaveStatus::Error(reason));
    let pixels = &session.project.root.folders[0].sprites[0].layers[0].frames[0].pixels;
    assert_eq!(pixels[0], b'N');
}

// Checks that retry saves everything once the problem is solved.
#[test]
fn retry_saves_after_the_problem_is_solved() {
    let parent = tempfile::tempdir().unwrap();
    let disk = FullDisk::new(true);
    let mut session = session_with_goblin(parent.path());
    session.set_writer(Arc::new(disk.clone()));
    session.apply("user", vec![draw(0, 'N')]).unwrap();
    assert!(matches!(session.flush(), SaveStatus::Error(_)));

    disk.set_full(false);
    assert_eq!(session.retry(), SaveStatus::Saved);

    let root = parent.path().join("Kit");
    let sprite = read_sprite(&root, &goblin_ref(), &dungeon16()).unwrap();
    assert_eq!(sprite.layers[0].frames[0].pixels[0], b'N');
}

// Checks that new changes after an error are not lost and are saved with the old ones.
#[test]
fn changes_after_an_error_are_kept_too() {
    let parent = tempfile::tempdir().unwrap();
    let disk = FullDisk::new(true);
    let mut session = session_with_goblin(parent.path());
    session.set_writer(Arc::new(disk.clone()));
    assert!(matches!(session.flush(), SaveStatus::Error(_)));

    session.apply("user", vec![draw(3, 'R')]).unwrap();
    assert!(matches!(session.status(), SaveStatus::Error(_)));
    disk.set_full(false);
    session.retry();

    let root = parent.path().join("Kit");
    let sprite = read_sprite(&root, &goblin_ref(), &dungeon16()).unwrap();
    assert_eq!(sprite.layers[0].frames[0].pixels[3], b'R');
}

// Checks that the error leaves the old file valid: a failed save does not damage the sprite on disk.
#[test]
fn a_failed_save_keeps_the_old_file() {
    let parent = tempfile::tempdir().unwrap();
    let disk = FullDisk::new(false);
    let mut session = session_with_goblin(parent.path());
    session.set_writer(Arc::new(disk.clone()));
    session.flush();
    session.apply("user", vec![draw(0, 'N')]).unwrap();
    disk.set_full(true);

    assert!(matches!(session.flush(), SaveStatus::Error(_)));

    let root = parent.path().join("Kit");
    let sprite = read_sprite(&root, &goblin_ref(), &dungeon16()).unwrap();
    assert_eq!(sprite.layers[0].frames[0].pixels, vec![b'.'; 16]);
}

// Checks the same through the background autosave: the status turns to error,
// and the retry action saves and sets "saved".
#[test]
fn autosave_shows_the_error_and_retry_works() {
    let parent = tempfile::tempdir().unwrap();
    let disk = FullDisk::new(true);
    let mut session = session_with_goblin(parent.path());
    session.set_writer(Arc::new(disk.clone()));
    let autosave = Autosave::start(session, Duration::from_millis(20));
    autosave.edit(|s| s.apply("user", vec![draw(0, 'N')]).unwrap());

    let start = Instant::now();
    while !matches!(autosave.status(), SaveStatus::Error(_)) {
        assert!(start.elapsed() < Duration::from_secs(5), "no error status");
        sleep(Duration::from_millis(5));
    }

    disk.set_full(false);
    assert_eq!(autosave.retry(), SaveStatus::Saved);
    assert_eq!(autosave.status(), SaveStatus::Saved);
}
