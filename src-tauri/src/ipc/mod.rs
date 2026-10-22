//! Tauri commands the UI calls.

use crate::menu::{self, CommandSpec};

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
