//! The native window menu. The list of commands comes from the UI (the same
//! registry as the project menu), so there is one source of names and shortcuts.

use serde::Deserialize;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Runtime};

/// Event sent to the UI when a menu command is chosen. The payload is the command id.
pub const COMMAND_EVENT: &str = "menu-command";

#[derive(Debug, Deserialize, PartialEq)]
pub struct CommandSpec {
    pub id: String,
    pub label: String,
    /// Tauri accelerator, for example "CmdOrCtrl+N".
    pub accelerator: Option<String>,
    pub enabled: bool,
    /// A separator line goes above this command.
    pub separator: bool,
}

/// What the File menu holds, in order.
#[derive(Debug, PartialEq)]
pub enum Entry<'a> {
    Separator,
    Command(&'a CommandSpec),
}

/// Expands the "separator above" flags into explicit separator entries.
pub fn entries(commands: &[CommandSpec]) -> Vec<Entry<'_>> {
    let mut list = Vec::new();
    for (index, command) in commands.iter().enumerate() {
        if command.separator && index > 0 {
            list.push(Entry::Separator);
        }
        list.push(Entry::Command(command));
    }
    list
}

/// Builds the menu: File (the project commands) and Edit (so copy and paste
/// keep working in text fields). macOS also gets its app menu.
pub fn build<R: Runtime>(
    app: &AppHandle<R>,
    file_title: &str,
    edit_title: &str,
    commands: &[CommandSpec],
) -> tauri::Result<Menu<R>> {
    let file = Submenu::new(app, file_title, true)?;
    for entry in entries(commands) {
        match entry {
            Entry::Separator => file.append(&PredefinedMenuItem::separator(app)?)?,
            Entry::Command(c) => file.append(&MenuItem::with_id(
                app,
                &c.id,
                &c.label,
                c.enabled,
                c.accelerator.as_deref(),
            )?)?,
        }
    }

    let edit = Submenu::with_items(
        app,
        edit_title,
        true,
        &[
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;

    let menu = Menu::new(app)?;
    #[cfg(target_os = "macos")]
    menu.append(&Submenu::with_items(
        app,
        "Sprixel",
        true,
        &[
            &PredefinedMenuItem::about(app, None, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::quit(app, None)?,
        ],
    )?)?;
    menu.append(&file)?;
    menu.append(&edit)?;
    Ok(menu)
}

/// Forwards a menu click to the UI.
pub fn forward<R: Runtime>(app: &AppHandle<R>, id: &str) {
    let _ = app.emit(COMMAND_EVENT, id.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(id: &str, separator: bool) -> CommandSpec {
        CommandSpec {
            id: id.into(),
            label: id.into(),
            accelerator: None,
            enabled: true,
            separator,
        }
    }

    // Checks that a separator flag becomes a separator entry above that
    // command, and never at the very top of the menu.
    #[test]
    fn separators_go_above_flagged_commands() {
        let commands = [spec("a", true), spec("b", false), spec("c", true)];
        let list = entries(&commands);

        assert_eq!(list.len(), 4);
        assert_eq!(list[0], Entry::Command(&commands[0]));
        assert_eq!(list[2], Entry::Separator);
        assert_eq!(list[3], Entry::Command(&commands[2]));
    }

    // Checks that the UI payload (camelCase JSON) is read into command specs.
    #[test]
    fn reads_the_ui_payload() {
        let json = r#"[{"id":"project.new","label":"New project",
            "accelerator":"CmdOrCtrl+N","enabled":false,"separator":false}]"#;
        let commands: Vec<CommandSpec> = serde_json::from_str(json).unwrap();

        assert_eq!(commands[0].id, "project.new");
        assert_eq!(commands[0].accelerator.as_deref(), Some("CmdOrCtrl+N"));
        assert!(!commands[0].enabled);
    }
}
