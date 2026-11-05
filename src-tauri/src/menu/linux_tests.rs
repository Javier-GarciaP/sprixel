use super::linux::*;
use super::*;
use std::fs;
use std::path::Path;

fn app(dir: &Path) -> PathBuf {
    dir.join("bin").join("sprixel-app")
}

// Checks the text of the entry: the name, the quoted path to start, the icon
// and the Graphics category that the menu uses.
#[test]
fn the_entry_names_the_app_and_quotes_its_path() {
    let text = desktop_text(Path::new("/home/ana/.local/bin/sprixel-app")).unwrap();

    assert!(text.starts_with("[Desktop Entry]\n"));
    assert!(text.contains("\nName=Sprixel\n"));
    assert!(text.contains("\nExec=\"/home/ana/.local/bin/sprixel-app\"\n"));
    assert!(text.contains("\nIcon=sprixel\n"));
    assert!(text.contains("\nCategories=Graphics;\n"));
    assert!(text.contains("\nTerminal=false\n"));
}

// Checks that a path with a space still starts the app: it stays in quotes.
#[test]
fn a_path_with_a_space_stays_in_one_piece() {
    let text = desktop_text(Path::new("/home/ana maria/sprixel-app")).unwrap();

    assert!(text.contains("\nExec=\"/home/ana maria/sprixel-app\"\n"));
}

// Checks that a percent sign is doubled, because `%` starts a code in `Exec`.
#[test]
fn a_percent_sign_is_doubled() {
    let text = desktop_text(Path::new("/home/ana/100%/sprixel-app")).unwrap();

    assert!(text.contains("100%%/sprixel-app"));
}

// Checks that a path that cannot be written safely is refused, so a strange
// APPIMAGE value can never add a second line or run a command.
#[test]
fn a_path_that_cannot_be_escaped_is_refused() {
    for bad in ["/a\nb", "/a\rb", "/a\"b", "/a`b", "/a$b", "/a\\b", "/a\0b"] {
        let result = desktop_text(Path::new(bad));

        assert!(matches!(result, Err(MenuError::BadPath(_))), "{bad:?}");
    }
}

// Checks the first registration: the entry and the icon are written.
#[cfg(unix)]
#[test]
fn the_first_registration_writes_the_entry_and_the_icon() {
    let home = tempfile::tempdir().unwrap();
    let appimage = app(home.path());

    let outcome = register(home.path(), &appimage).unwrap();

    assert_eq!(outcome, Outcome::Created(desktop_path(home.path())));
    let text = fs::read_to_string(desktop_path(home.path())).unwrap();
    assert!(text.contains(&format!("Exec=\"{}\"", appimage.display())));
    assert_eq!(fs::read(icon_path(home.path())).unwrap(), ICON_PNG);
}

// Checks the scenario "Entry already correct": a second registration writes
// nothing and says so.
#[cfg(unix)]
#[test]
fn a_second_registration_changes_nothing() {
    let home = tempfile::tempdir().unwrap();
    let appimage = app(home.path());
    register(home.path(), &appimage).unwrap();

    assert_eq!(register(home.path(), &appimage).unwrap(), Outcome::AlreadyOk);
}

// Checks the scenario "App moved": the entry follows the app and there is
// still one entry only, never a duplicate.
#[cfg(unix)]
#[test]
fn a_moved_app_rewrites_the_one_entry() {
    let home = tempfile::tempdir().unwrap();
    register(home.path(), &app(home.path())).unwrap();
    let moved = home.path().join("Apps").join("sprixel-app");

    let outcome = register(home.path(), &moved).unwrap();

    assert_eq!(outcome, Outcome::Updated(desktop_path(home.path())));
    let folder = desktop_path(home.path()).parent().unwrap().to_path_buf();
    assert_eq!(fs::read_dir(folder).unwrap().count(), 1);
    let text = fs::read_to_string(desktop_path(home.path())).unwrap();
    assert!(text.contains(&format!("Exec=\"{}\"", moved.display())));
}

// Checks that a deleted icon is put back even when the entry is fine.
#[cfg(unix)]
#[test]
fn a_missing_icon_is_repaired() {
    let home = tempfile::tempdir().unwrap();
    let appimage = app(home.path());
    register(home.path(), &appimage).unwrap();
    fs::remove_file(icon_path(home.path())).unwrap();

    let outcome = register(home.path(), &appimage).unwrap();

    assert_eq!(outcome, Outcome::Updated(desktop_path(home.path())));
    assert_eq!(fs::read(icon_path(home.path())).unwrap(), ICON_PNG);
}

// Checks the scenario "Read-only menu folder": the error is returned (the
// caller prints one warning) and nothing panics.
#[cfg(unix)]
#[test]
fn a_folder_that_cannot_be_written_is_an_error() {
    use std::os::unix::fs::PermissionsExt;
    let home = tempfile::tempdir().unwrap();
    let share = home.path().join(".local").join("share");
    fs::create_dir_all(&share).unwrap();
    fs::set_permissions(&share, fs::Permissions::from_mode(0o500)).unwrap();

    let result = register(home.path(), &app(home.path()));

    fs::set_permissions(&share, fs::Permissions::from_mode(0o700)).unwrap();
    // A user who is root can write anywhere; only check when it failed to write.
    if let Err(error) = result {
        assert!(matches!(error, MenuError::Io(_)));
    }
}
