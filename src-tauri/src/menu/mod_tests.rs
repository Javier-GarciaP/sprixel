use super::*;
use std::cell::Cell;
use std::fs;

fn env(home: &Path) -> Env {
    Env {
        home: home.to_path_buf(),
        appimage: Some(home.join("bin").join("sprixel-app")),
        exe: home.join("Sprixel.exe"),
        app_data: home.join("Roaming"),
        local_app_data: home.join("Local"),
        release_build: true,
    }
}

fn never(_: &str) -> io::Result<()> {
    Ok(())
}

// Checks the Linux path of the dispatcher: an AppImage start registers.
#[cfg(unix)]
#[test]
fn linux_registers_at_start_when_running_as_an_appimage() {
    let home = tempfile::tempdir().unwrap();

    let outcome = register_for(Platform::Linux, Request::AtStart, &env(home.path()), &never).unwrap();

    assert!(matches!(outcome, Outcome::Created(_)));
}

// Checks that a run that is not an AppImage (a dev run, `npm run tauri dev`)
// never writes an entry on its own.
#[test]
fn linux_does_not_register_at_start_outside_an_appimage() {
    let home = tempfile::tempdir().unwrap();
    let mut e = env(home.path());
    e.appimage = None;

    let outcome = register_for(Platform::Linux, Request::AtStart, &e, &never).unwrap();

    assert!(matches!(outcome, Outcome::Skipped(_)));
    assert!(!linux::desktop_path(home.path()).exists());
}

// Checks that a debug build on Windows never writes a shortcut on its own.
#[test]
fn windows_does_not_register_a_debug_build_at_start() {
    let home = tempfile::tempdir().unwrap();
    let mut e = env(home.path());
    e.release_build = false;
    let runs = Cell::new(0);

    let outcome = register_for(Platform::Windows, Request::AtStart, &e, &|_| {
        runs.set(runs.get() + 1);
        Ok(())
    })
    .unwrap();

    assert!(matches!(outcome, Outcome::Skipped(_)));
    assert_eq!(runs.get(), 0);
}

// Checks that macOS needs nothing: the app bundle is the entry.
#[test]
fn macos_needs_no_entry() {
    let home = tempfile::tempdir().unwrap();

    let outcome = register_for(Platform::MacOs, Request::Explicit, &env(home.path()), &never).unwrap();

    assert_eq!(outcome, Outcome::NotNeeded);
}

// Checks the scenario "The person said no in the installer": with the file
// there, the start writes nothing.
#[test]
fn the_opt_out_file_stops_the_start_registration() {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join(".sprixel")).unwrap();
    fs::write(opt_out_path(home.path()), "").unwrap();

    let outcome = register_for(Platform::Linux, Request::AtStart, &env(home.path()), &never).unwrap();

    assert!(matches!(outcome, Outcome::Skipped(_)));
    assert!(!linux::desktop_path(home.path()).exists());
}

// Checks the scenario "The person asks for it later": `--register-menu`
// registers and removes the opt-out file.
#[cfg(unix)]
#[test]
fn an_explicit_request_registers_and_removes_the_opt_out_file() {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join(".sprixel")).unwrap();
    fs::write(opt_out_path(home.path()), "").unwrap();

    let outcome = register_for(Platform::Linux, Request::Explicit, &env(home.path()), &never).unwrap();

    assert!(matches!(outcome, Outcome::Created(_)));
    assert!(!opt_out_path(home.path()).exists());
}

// Checks that an explicit request outside an AppImage still works, using the
// path of the running program, so the install script never fails here.
#[cfg(unix)]
#[test]
fn an_explicit_request_without_an_appimage_uses_the_running_program() {
    let home = tempfile::tempdir().unwrap();
    let mut e = env(home.path());
    e.appimage = None;
    e.exe = home.path().join("sprixel-app");

    let outcome = register_for(Platform::Linux, Request::Explicit, &e, &never).unwrap();

    assert!(matches!(outcome, Outcome::Created(_)));
    let text = fs::read_to_string(linux::desktop_path(home.path())).unwrap();
    assert!(text.contains(&format!("Exec=\"{}\"", e.exe.display())));
}

// Checks the argument: only `--register-menu` is recognized.
#[test]
fn only_the_register_menu_argument_is_recognized() {
    assert!(wants_register_only(["sprixel", "--register-menu"].map(String::from)));
    assert!(!wants_register_only(["sprixel"].map(String::from)));
    assert!(!wants_register_only(["sprixel", "--other"].map(String::from)));
}

// Checks the text printed for each outcome: one short line that says what was
// done (or why nothing was).
#[test]
fn every_outcome_has_one_short_line() {
    let line = |o: &Result<Outcome, MenuError>| describe(o);

    assert!(line(&Ok(Outcome::Created("/a".into()))).contains("created"));
    assert!(line(&Ok(Outcome::Updated("/a".into()))).contains("updated"));
    assert!(line(&Ok(Outcome::AlreadyOk)).contains("already"));
    assert!(line(&Ok(Outcome::NotNeeded)).contains("macOS"));
    assert!(line(&Err(MenuError::BadPath("x".into()))).contains("not registered"));
    assert!(!line(&Ok(Outcome::AlreadyOk)).contains('\n'));
}
