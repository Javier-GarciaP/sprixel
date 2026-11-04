use super::*;
use crate::storage::{DiskWriter, FileWriter};
use std::io;
use std::path::Path;

fn file(name: &str, text: &str) -> ExportFile {
    ExportFile { name: name.to_string(), bytes: text.as_bytes().to_vec() }
}

/// A writer that works for `ok` files and then fails with `kind`.
struct FailsAfter {
    ok: usize,
    kind: io::ErrorKind,
    done: std::sync::atomic::AtomicUsize,
}

impl FailsAfter {
    fn new(ok: usize, kind: io::ErrorKind) -> FailsAfter {
        FailsAfter { ok, kind, done: 0.into() }
    }
}

impl FileWriter for FailsAfter {
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        if self.done.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= self.ok {
            return Err(io::Error::new(self.kind, "disk says no"));
        }
        DiskWriter.write(path, bytes)
    }
}

fn names_in(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

// Checks the spec scenario: an output folder outside the project is refused
// with `outside_project`, and nothing is written anywhere.
#[test]
fn a_folder_outside_the_project_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project");
    std::fs::create_dir(&root).unwrap();
    for bad in ["../../etc", "..", "a/../../b", "/etc", "\\etc", "C:/x", "C:\\x", "a/..\\..\\b"] {
        let error = write_files(&DiskWriter, &root, bad, &[file("a.png", "x")]).unwrap_err();
        assert_eq!(error.code, "outside_project", "{bad}");
    }
    assert_eq!(names_in(&root), Vec::<String>::new());
    assert_eq!(names_in(dir.path()), ["project"]);
}

// Checks that an empty folder and the reserved `.sprixel` folder are refused.
#[test]
fn empty_and_reserved_folders_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    for bad in ["", ".", ".sprixel", ".Sprixel/trash"] {
        let error = write_files(&DiskWriter, dir.path(), bad, &[file("a.png", "x")]).unwrap_err();
        assert_eq!(error.code, "invalid_path", "{bad}");
    }
}

// Checks that a symbolic link inside the project that points outside is
// refused too (it is a way around the `..` check).
#[cfg(unix)]
#[test]
fn a_link_that_points_outside_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project");
    let outside = dir.path().join("elsewhere");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("link")).unwrap();
    let error = write_files(&DiskWriter, &root, "link/out", &[file("a.png", "x")]).unwrap_err();
    assert_eq!(error.code, "outside_project");
    assert_eq!(names_in(&outside), Vec::<String>::new());
}

// Checks the normal case: the folder is created, the files are written, no
// temporary folder stays, and the result says where.
#[test]
fn writes_the_files_and_leaves_no_temporary_folder() {
    let dir = tempfile::tempdir().unwrap();
    let files = [file("kit.png", "png"), file("kit.json", "json")];
    let written = write_files(&DiskWriter, dir.path(), "assets/sprites", &files).unwrap();
    assert_eq!(written.output, "assets/sprites");
    assert_eq!(written.files, ["kit.png", "kit.json"]);
    let folder = dir.path().join("assets").join("sprites");
    assert_eq!(written.folder, folder);
    assert_eq!(names_in(&folder), ["kit.json", "kit.png"]);
    assert_eq!(std::fs::read_to_string(folder.join("kit.png")).unwrap(), "png");
}

// Checks that files of an earlier export with the same names are replaced, and
// other files in the folder are left alone.
#[test]
fn replaces_files_of_an_earlier_export() {
    let dir = tempfile::tempdir().unwrap();
    write_files(&DiskWriter, dir.path(), "out", &[file("kit.png", "old")]).unwrap();
    std::fs::write(dir.path().join("out").join("mine.txt"), "keep").unwrap();
    write_files(&DiskWriter, dir.path(), "out", &[file("kit.png", "new")]).unwrap();
    assert_eq!(std::fs::read_to_string(dir.path().join("out/kit.png")).unwrap(), "new");
    assert_eq!(names_in(&dir.path().join("out")), ["kit.png", "mine.txt"]);
}

// Checks the failed write case: the disk fails on the second file. The first
// file is not left in the folder, the earlier export is untouched, and the
// error says why (`no_space`).
#[test]
fn a_failed_write_leaves_no_partial_files() {
    let dir = tempfile::tempdir().unwrap();
    write_files(&DiskWriter, dir.path(), "out", &[file("kit.png", "old")]).unwrap();
    let files = [file("kit.png", "new"), file("kit.json", "json")];
    let writer = FailsAfter::new(1, io::ErrorKind::StorageFull);
    let error = write_files(&writer, dir.path(), "out", &files).unwrap_err();
    assert_eq!(error.code, "no_space");
    assert_eq!(names_in(&dir.path().join("out")), ["kit.png"]);
    assert_eq!(std::fs::read_to_string(dir.path().join("out/kit.png")).unwrap(), "old");
}

// Checks that a folder that did not exist is not left behind after a failure,
// and that a missing permission gives `no_permission`.
#[test]
fn a_failure_does_not_leave_a_new_empty_folder() {
    let dir = tempfile::tempdir().unwrap();
    let writer = FailsAfter::new(0, io::ErrorKind::PermissionDenied);
    let error = write_files(&writer, dir.path(), "out", &[file("a.png", "x")]).unwrap_err();
    assert_eq!(error.code, "no_permission");
    assert!(!dir.path().join("out").exists());
}
