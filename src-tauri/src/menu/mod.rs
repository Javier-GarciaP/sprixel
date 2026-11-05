//! The entry of the app in the applications menu (see the `app-menu-entries`
//! spec): a `.desktop` file on Linux and a Start Menu shortcut on Windows.
//! macOS needs nothing, the app bundle is the entry.

pub mod linux;
pub mod windows;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The icon that the Linux entry shows. It travels inside the app.
pub const ICON_PNG: &[u8] = include_bytes!("../../icons/128x128@2x.png");

/// The system whose rules apply. Tests pick it, the app uses its own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Platform {
    Linux,
    Windows,
    MacOs,
}

/// Why the entry is being registered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Request {
    /// The app is starting. The person may have opted out.
    AtStart,
    /// `--register-menu`: the person asked for it.
    Explicit,
}

/// What a registration did.
#[derive(Debug, PartialEq)]
pub enum Outcome {
    Created(PathBuf),
    Updated(PathBuf),
    AlreadyOk,
    Skipped(&'static str),
    NotNeeded,
}

/// Why the entry could not be written.
#[derive(Debug)]
pub enum MenuError {
    /// The path of the app cannot be written safely in the entry.
    BadPath(String),
    Io(io::Error),
}

impl From<io::Error> for MenuError {
    fn from(error: io::Error) -> Self {
        MenuError::Io(error)
    }
}

/// Everything a registration reads from the computer.
pub struct Env {
    pub home: PathBuf,
    /// The AppImage that is running (Linux only).
    pub appimage: Option<PathBuf>,
    pub exe: PathBuf,
    /// `%APPDATA%` and `%LOCALAPPDATA%` (Windows only).
    pub app_data: PathBuf,
    pub local_app_data: PathBuf,
    /// True in a build for people, false in a debug build.
    pub release_build: bool,
}

/// Runs a PowerShell script (Windows only).
pub type Runner<'a> = &'a dyn Fn(&str) -> io::Result<()>;

/// The file that means "do not register the entry by yourself".
pub fn opt_out_path(home: &Path) -> PathBuf {
    home.join(".sprixel").join("no-menu-entry")
}

/// Writes `bytes` to `path` only when the file is missing or different. It
/// writes a temporary file and renames it, so a half-written file is never
/// seen. Returns true when it wrote.
pub(crate) fn write_if_different(path: &Path, bytes: &[u8]) -> io::Result<bool> {
    if fs::read(path).is_ok_and(|current| current == bytes) {
        return Ok(false);
    }
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let name = path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let temporary = path.with_file_name(format!(".{name}.tmp"));
    fs::write(&temporary, bytes)?;
    fs::rename(&temporary, path)?;
    Ok(true)
}

/// Registers the entry for `platform`. Pure: everything it reads comes in
/// `env`, and PowerShell is `run`, so every rule can be tested.
pub fn register_for(
    platform: Platform,
    request: Request,
    env: &Env,
    run: Runner,
) -> Result<Outcome, MenuError> {
    if platform == Platform::MacOs {
        return Ok(Outcome::NotNeeded);
    }
    let opt_out = opt_out_path(&env.home);
    match request {
        Request::AtStart if opt_out.exists() => return Ok(Outcome::Skipped("you chose no menu entry")),
        Request::AtStart => {}
        // The person asked: that cancels an earlier "no".
        Request::Explicit => {
            let _ = fs::remove_file(&opt_out);
        }
    }
    match platform {
        Platform::Linux => {
            let app = match (&env.appimage, request) {
                (Some(path), _) => path.clone(),
                (None, Request::AtStart) => return Ok(Outcome::Skipped("not running as an installed AppImage")),
                (None, Request::Explicit) => env.exe.clone(),
            };
            linux::register(&env.home, &app)
        }
        Platform::Windows => {
            if request == Request::AtStart && !env.release_build {
                return Ok(Outcome::Skipped("this is a debug build"));
            }
            windows::register(&env.app_data, &env.local_app_data, &env.exe, run)
        }
        Platform::MacOs => Ok(Outcome::NotNeeded),
    }
}

/// True when the program was started with `--register-menu`.
pub fn wants_register_only(args: impl IntoIterator<Item = String>) -> bool {
    args.into_iter().skip(1).any(|arg| arg == "--register-menu")
}

/// One short line that says what a registration did.
pub fn describe(result: &Result<Outcome, MenuError>) -> String {
    match result {
        Ok(Outcome::Created(path)) => format!("menu entry created: {}", path.display()),
        Ok(Outcome::Updated(path)) => format!("menu entry updated: {}", path.display()),
        Ok(Outcome::AlreadyOk) => "menu entry already correct".into(),
        Ok(Outcome::Skipped(why)) => format!("menu entry skipped: {why}"),
        Ok(Outcome::NotNeeded) => "macOS needs no menu entry: the app is in ~/Applications".into(),
        Err(MenuError::BadPath(why)) => format!("menu entry not registered: {why}"),
        Err(MenuError::Io(error)) => format!("menu entry not registered: {error}. Run the app with --register-menu to try again"),
    }
}

fn platform_here() -> Platform {
    if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::MacOs
    } else {
        Platform::Linux
    }
}

/// Reads the computer. `None` when the home folder cannot be found.
fn env_here() -> Option<Env> {
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())?;
    let folder = |name: &str| std::env::var_os(name).map(PathBuf::from).unwrap_or_default();
    Some(Env {
        home,
        appimage: std::env::var_os("APPIMAGE").map(PathBuf::from).filter(|path| path.is_absolute()),
        exe: std::env::current_exe().ok()?,
        app_data: folder("APPDATA"),
        local_app_data: folder("LOCALAPPDATA"),
        release_build: !cfg!(debug_assertions),
    })
}

/// Runs a PowerShell script with no window.
#[cfg(windows)]
fn run_powershell(script: &str) -> io::Result<()> {
    use std::os::windows::process::CommandExt;
    const NO_WINDOW: u32 = 0x0800_0000;
    let status = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .creation_flags(NO_WINDOW)
        .status()?;
    if status.success() { Ok(()) } else { Err(io::Error::other(format!("powershell ended with {status}"))) }
}

#[cfg(not(windows))]
fn run_powershell(_script: &str) -> io::Result<()> {
    Err(io::Error::other("PowerShell is only used on Windows"))
}

/// After a new entry, tell the desktop to read it. Both tools may be missing:
/// the entry is found anyway the next time the menu is read.
fn refresh_linux_caches(home: &Path) {
    let quiet = |mut command: std::process::Command| {
        let _ = command
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    };
    let mut apps = std::process::Command::new("update-desktop-database");
    apps.arg(home.join(".local/share/applications"));
    quiet(apps);
    let mut icons = std::process::Command::new("gtk-update-icon-cache");
    icons.args(["-f", "-t"]).arg(home.join(".local/share/icons/hicolor"));
    quiet(icons);
}

fn register_here(request: Request) -> Result<Outcome, MenuError> {
    let env = env_here().ok_or_else(|| MenuError::BadPath("the home folder was not found".into()))?;
    let platform = platform_here();
    let outcome = register_for(platform, request, &env, &run_powershell)?;
    if platform == Platform::Linux && matches!(outcome, Outcome::Created(_) | Outcome::Updated(_)) {
        refresh_linux_caches(&env.home);
    }
    Ok(outcome)
}

/// `--register-menu`: registers, prints one line and returns. It never fails
/// the caller, a problem is only printed.
pub fn register_only() {
    println!("{}", describe(&register_here(Request::Explicit)));
}

/// At start, in the background so the window never waits for it. It says
/// something only when it changed the menu or could not.
pub fn register_at_start() {
    std::thread::spawn(|| {
        let result = register_here(Request::AtStart);
        if !matches!(result, Ok(Outcome::AlreadyOk | Outcome::Skipped(_) | Outcome::NotNeeded)) {
            eprintln!("[menu] {}", describe(&result));
        }
    });
}

#[cfg(test)]
mod linux_tests;
#[cfg(test)]
mod mod_tests;
#[cfg(test)]
mod windows_tests;
