//! Tauri commands the UI calls.

pub mod browser;
pub mod preferences;
pub mod project;

use browser::{CHANGED_EVENT, CoreError, FrameInfo, SpriteTarget, TreeInfo};
use preferences::{Data, RecentProject, Settings};
use project::{ProjectError, ProjectInfo, ProjectState, StatusInfo};
use std::path::PathBuf;
use tauri::Manager;

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

/// The folder suggested in the New project dialog: `~/Proyectos`.
#[tauri::command]
pub fn default_project_location(app: tauri::AppHandle) -> Result<String, String> {
    let home = app.path().home_dir().map_err(|e| e.to_string())?;
    Ok(home.join("Proyectos").to_string_lossy().into_owned())
}

#[tauri::command]
pub fn create_project(
    state: tauri::State<ProjectState>,
    parent: String,
    name: String,
) -> Result<ProjectInfo, ProjectError> {
    project::create_project(&state, std::path::Path::new(&parent), &name)
}

#[tauri::command]
pub fn open_project(
    state: tauri::State<ProjectState>,
    path: String,
) -> Result<ProjectInfo, ProjectError> {
    project::open_project(&state, std::path::Path::new(&path))
}

#[tauri::command]
pub fn close_project(state: tauri::State<ProjectState>) {
    project::close_project(&state);
}

#[tauri::command]
pub fn project_status(state: tauri::State<ProjectState>) -> Option<StatusInfo> {
    project::project_status(&state)
}

#[tauri::command]
pub fn retry_save(state: tauri::State<ProjectState>) -> Option<StatusInfo> {
    project::retry_save(&state)
}

/// Sends `project://changed` after a change, then gives the value to the UI.
/// A failed change sends nothing.
fn finish<T>(
    app: &tauri::AppHandle,
    result: Result<(T, browser::ChangedEvent), CoreError>,
) -> Result<T, CoreError> {
    let (value, event) = result?;
    let _ = tauri::Emitter::emit(app, CHANGED_EVENT, event);
    Ok(value)
}

#[tauri::command]
pub fn project_tree(state: tauri::State<ProjectState>) -> Result<TreeInfo, CoreError> {
    browser::project_tree(&state)
}

#[tauri::command]
pub fn sprite_frame(
    state: tauri::State<ProjectState>,
    folder: Vec<String>,
    name: String,
    frame: u32,
) -> Result<FrameInfo, CoreError> {
    browser::sprite_frame(&state, &folder, &name, frame)
}

#[tauri::command]
pub fn check_name(
    state: tauri::State<ProjectState>,
    parent: Vec<String>,
    name: String,
    current: Option<String>,
) -> Result<Option<&'static str>, CoreError> {
    browser::check_name(&state, &parent, &name, current.as_deref())
}

#[tauri::command]
pub fn create_sprite(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    folder: Vec<String>,
    name: String,
    width: u32,
    height: u32,
    palette: Option<String>,
) -> Result<SpriteTarget, CoreError> {
    finish(
        &app,
        browser::create_sprite(&state, &folder, &name, width, height, palette.as_deref()),
    )
}

#[tauri::command]
pub fn create_folder(
    app: tauri::AppHandle,
    state: tauri::State<ProjectState>,
    parent: Vec<String>,
    name: String,
) -> Result<Vec<String>, CoreError> {
    finish(&app, browser::create_folder(&state, &parent, &name))
}
