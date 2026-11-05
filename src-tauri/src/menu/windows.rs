//! The Windows entry: a shortcut in the Start Menu of the user.

use super::{MenuError, Outcome, Runner};
use std::fs;
use std::path::{Path, PathBuf};

pub fn shortcut_path(app_data: &Path) -> PathBuf {
    app_data.join("Microsoft").join("Windows").join("Start Menu").join("Programs").join("Sprixel.lnk")
}

/// Keeps the path of the exe that made the shortcut. A `.lnk` is awkward to
/// read, so this small file says if the shortcut is still right.
pub fn marker_path(local_app_data: &Path) -> PathBuf {
    local_app_data.join("Sprixel").join("menu-entry.txt")
}

/// A single quote would end a PowerShell string, so it is doubled.
fn quote(text: &str) -> String {
    text.replace('\'', "''")
}

/// The PowerShell script that makes the shortcut. The icon is the one inside
/// the exe.
pub fn shortcut_script(shortcut: &Path, exe: &Path) -> String {
    let exe = exe.to_string_lossy();
    // Cut by hand: a Windows path must be read the same way on every system.
    let folder = exe.rsplit_once(['\\', '/']).map_or("", |(folder, _)| folder);
    format!(
        "$s = (New-Object -ComObject WScript.Shell).CreateShortcut('{}')\n$s.TargetPath = '{}'\n$s.WorkingDirectory = '{}'\n$s.IconLocation = '{},0'\n$s.Description = 'Pixel art for game sprites'\n$s.Save()\n",
        quote(&shortcut.to_string_lossy()),
        quote(&exe),
        quote(folder),
        quote(&exe),
    )
}

/// Makes the shortcut when it is missing or made for another exe. `run`
/// executes the PowerShell script.
pub fn register(
    app_data: &Path,
    local_app_data: &Path,
    exe: &Path,
    run: Runner,
) -> Result<Outcome, MenuError> {
    let shortcut = shortcut_path(app_data);
    let marker = marker_path(local_app_data);
    let exe_text = exe.to_string_lossy();
    let right = shortcut.exists() && fs::read_to_string(&marker).is_ok_and(|text| text == exe_text);
    if right {
        return Ok(Outcome::AlreadyOk);
    }
    let existed = shortcut.exists() || marker.exists();
    run(&shortcut_script(&shortcut, exe))?;
    super::write_if_different(&marker, exe_text.as_bytes())?;
    Ok(if existed { Outcome::Updated(shortcut) } else { Outcome::Created(shortcut) })
}
