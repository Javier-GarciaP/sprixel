use super::windows::*;
use super::*;
use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::Path;

// A fake PowerShell: it records the scripts and, like the real one, creates
// the shortcut file.
struct Fake {
    scripts: RefCell<Vec<String>>,
    shortcut: PathBuf,
}

impl Fake {
    fn new(app_data: &Path) -> Self {
        Fake { scripts: RefCell::new(Vec::new()), shortcut: shortcut_path(app_data) }
    }
    fn run(&self, script: &str) -> io::Result<()> {
        self.scripts.borrow_mut().push(script.to_string());
        fs::create_dir_all(self.shortcut.parent().unwrap())?;
        fs::write(&self.shortcut, "lnk")
    }
}

fn dirs() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let app_data = root.path().join("Roaming");
    let local = root.path().join("Local");
    (root, app_data, local)
}

// Checks the text of the PowerShell script: the target, the icon of the exe,
// the folder to work in and the save.
#[test]
fn the_script_targets_the_exe_and_uses_its_icon() {
    let script = shortcut_script(
        Path::new("C:\\Menu\\Sprixel.lnk"),
        Path::new("C:\\Apps\\Sprixel\\Sprixel.exe"),
    );

    assert!(script.contains("CreateShortcut('C:\\Menu\\Sprixel.lnk')"));
    assert!(script.contains("TargetPath = 'C:\\Apps\\Sprixel\\Sprixel.exe'"));
    assert!(script.contains("IconLocation = 'C:\\Apps\\Sprixel\\Sprixel.exe,0'"));
    assert!(script.contains("WorkingDirectory = 'C:\\Apps\\Sprixel'"));
    assert!(script.trim_end().ends_with(".Save()"));
}

// Checks that a single quote in a path cannot end the PowerShell string early
// and run something else: it is doubled.
#[test]
fn a_single_quote_in_a_path_is_doubled() {
    let script = shortcut_script(Path::new("C:\\M\\S.lnk"), Path::new("C:\\O'Neil\\Sprixel.exe"));

    assert!(script.contains("TargetPath = 'C:\\O''Neil\\Sprixel.exe'"));
}

// Checks the first registration: PowerShell runs once and the marker keeps
// the path of the exe.
#[test]
fn the_first_registration_creates_the_shortcut_and_the_marker() {
    let (_root, app_data, local) = dirs();
    let fake = Fake::new(&app_data);
    let exe = Path::new("C:\\Sprixel\\Sprixel.exe");

    let outcome = register(&app_data, &local, exe, &|s| fake.run(s)).unwrap();

    assert_eq!(outcome, Outcome::Created(shortcut_path(&app_data)));
    assert_eq!(fake.scripts.borrow().len(), 1);
    assert_eq!(fs::read_to_string(marker_path(&local)).unwrap(), "C:\\Sprixel\\Sprixel.exe");
}

// Checks "Entry already correct": nothing runs, so the app does not start a
// PowerShell every time it opens.
#[test]
fn a_correct_shortcut_does_not_start_powershell_again() {
    let (_root, app_data, local) = dirs();
    let fake = Fake::new(&app_data);
    let exe = Path::new("C:\\Sprixel\\Sprixel.exe");
    register(&app_data, &local, exe, &|s| fake.run(s)).unwrap();

    let outcome = register(&app_data, &local, exe, &|s| fake.run(s)).unwrap();

    assert_eq!(outcome, Outcome::AlreadyOk);
    assert_eq!(fake.scripts.borrow().len(), 1);
}

// Checks "App moved": a different exe path rewrites the same shortcut.
#[test]
fn a_moved_exe_rewrites_the_shortcut() {
    let (_root, app_data, local) = dirs();
    let fake = Fake::new(&app_data);
    register(&app_data, &local, Path::new("C:\\A\\Sprixel.exe"), &|s| fake.run(s)).unwrap();

    let outcome =
        register(&app_data, &local, Path::new("C:\\B\\Sprixel.exe"), &|s| fake.run(s)).unwrap();

    assert_eq!(outcome, Outcome::Updated(shortcut_path(&app_data)));
    assert_eq!(fs::read_to_string(marker_path(&local)).unwrap(), "C:\\B\\Sprixel.exe");
}

// Checks that a deleted shortcut is made again even if the marker matches.
#[test]
fn a_deleted_shortcut_is_made_again() {
    let (_root, app_data, local) = dirs();
    let fake = Fake::new(&app_data);
    let exe = Path::new("C:\\Sprixel\\Sprixel.exe");
    register(&app_data, &local, exe, &|s| fake.run(s)).unwrap();
    fs::remove_file(shortcut_path(&app_data)).unwrap();

    let outcome = register(&app_data, &local, exe, &|s| fake.run(s)).unwrap();

    assert_eq!(outcome, Outcome::Updated(shortcut_path(&app_data)));
    assert_eq!(fake.scripts.borrow().len(), 2);
}

// Checks that a failing PowerShell is an error and no marker is written, so
// the next start tries again.
#[test]
fn a_failing_powershell_is_an_error_and_leaves_no_marker() {
    let (_root, app_data, local) = dirs();
    let exe = Path::new("C:\\Sprixel\\Sprixel.exe");

    let result = register(&app_data, &local, exe, &|_| Err(io::Error::other("no powershell")));

    assert!(matches!(result, Err(MenuError::Io(_))));
    assert!(!marker_path(&local).exists());
}
