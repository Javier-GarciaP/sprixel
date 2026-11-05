//! The Linux entry: `~/.local/share/applications/sprixel.desktop` and the icon.

use super::{ICON_PNG, MenuError, Outcome, write_if_different};
use std::path::{Path, PathBuf};

pub fn desktop_path(home: &Path) -> PathBuf {
    home.join(".local/share/applications/sprixel.desktop")
}

pub fn icon_path(home: &Path) -> PathBuf {
    home.join(".local/share/icons/hicolor/256x256/apps/sprixel.png")
}

/// The text of the `.desktop` file for the app at `exec`. A path that cannot
/// be written safely (a line break, or a character that has a meaning inside
/// the quotes of `Exec`) is refused, so an odd `APPIMAGE` value can never add
/// a line or run a command.
pub fn desktop_text(exec: &Path) -> Result<String, MenuError> {
    let path = exec
        .to_str()
        .ok_or_else(|| MenuError::BadPath("the path of the app is not valid text".into()))?;
    if path.chars().any(|c| matches!(c, '\n' | '\r' | '\0' | '"' | '`' | '$' | '\\')) {
        return Err(MenuError::BadPath(format!("the path of the app has a character that cannot be used in a menu entry: {path:?}")));
    }
    // In `Exec`, `%` starts a code, so a real one is written twice.
    let quoted = path.replace('%', "%%");
    Ok(format!(
        "[Desktop Entry]\nType=Application\nName=Sprixel\nComment=Pixel art for game sprites\nExec=\"{quoted}\"\nIcon=sprixel\nTerminal=false\nCategories=Graphics;\n"
    ))
}

/// Writes the entry and the icon for the app at `appimage`. Writes only what
/// is missing or different.
pub fn register(home: &Path, appimage: &Path) -> Result<Outcome, MenuError> {
    let text = desktop_text(appimage)?;
    let desktop = desktop_path(home);
    let existed = desktop.exists();
    let entry_changed = write_if_different(&desktop, text.as_bytes())?;
    let icon_changed = write_if_different(&icon_path(home), ICON_PNG)?;
    Ok(match (entry_changed || icon_changed, existed) {
        (false, _) => Outcome::AlreadyOk,
        (true, false) => Outcome::Created(desktop),
        (true, true) => Outcome::Updated(desktop),
    })
}
