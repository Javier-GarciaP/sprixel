//! Tauri commands the UI calls.

pub mod preferences;

use crate::menu::{self, CommandSpec};
use preferences::{Data, RecentProject, Settings};
use std::path::PathBuf;
use tauri::Manager;

/// Replaces the native window menu. The UI calls it at start and whenever the
/// language or the available commands change.
#[tauri::command]
pub fn set_menu(
    app: tauri::AppHandle,
    file_title: String,
    edit_title: String,
    commands: Vec<CommandSpec>,
) -> Result<(), String> {
    let menu = menu::build(&app, &file_title, &edit_title, &commands).map_err(|e| e.to_string())?;
    app.set_menu(menu).map_err(|e| e.to_string())?;
    Ok(())
}

/// The preferences file, in the app config folder (for example
/// `~/.config/com.sprixel.app/preferences.json` on Linux).
fn preferences_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let folder = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(folder.join("preferences.json"))
}

fn change(app: &tauri::AppHandle, edit: impl FnOnce(&mut Data)) -> Result<Data, String> {
    let file = preferences_file(app)?;
    let mut data = preferences::load(&file);
    edit(&mut data);
    preferences::save(&file, &data).map_err(|e| e.to_string())?;
    Ok(data)
}

#[tauri::command]
pub fn load_preferences(app: tauri::AppHandle) -> Result<Data, String> {
    Ok(preferences::load(&preferences_file(&app)?))
}

#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    change(&app, |data| data.settings = settings).map(|_| ())
}

#[tauri::command]
pub fn add_recent_project(
    app: tauri::AppHandle,
    path: String,
    name: String,
) -> Result<Vec<RecentProject>, String> {
    change(&app, |data| preferences::add_recent(data, RecentProject { path, name }))
        .map(|data| data.recent_projects)
}

#[tauri::command]
pub fn remove_recent_project(
    app: tauri::AppHandle,
    path: String,
) -> Result<Vec<RecentProject>, String> {
    change(&app, |data| preferences::remove_recent(data, &path)).map(|data| data.recent_projects)
}

/// Used to tell the user that a recent project folder was moved or deleted.
/// It only checks that the folder exists; it never reads project files.
#[tauri::command]
pub fn folder_exists(path: String) -> bool {
    std::path::Path::new(&path).is_dir()
}
