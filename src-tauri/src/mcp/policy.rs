//! The permissions the user gives to agents. They are checked in one place,
//! before a tool runs, inside the core: closing a switch in the interface is
//! enough, and no client can skip it.

use super::tool::{Need, ToolError};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const PERMISSIONS_FILE: &str = "permissions.json";

/// The switches of the Agents tab. They apply to every client at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Permissions {
    /// Create sprites and folders and add frames.
    pub create: bool,
    /// Change pixels, animations and palette colors.
    pub edit: bool,
    /// Move sprites to the trash. Off by default.
    pub delete: bool,
    /// Write export files into the project.
    pub export: bool,
    /// Open another project in the app. Off by default: it can open any folder.
    pub open_project: bool,
    /// Ask the user before a new client may connect.
    pub ask_on_connect: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Permissions { create: true, edit: true, delete: false, export: true, open_project: false, ask_on_connect: true }
    }
}

impl Permissions {
    /// Reads `permissions.json`. A missing or damaged file gives the
    /// defaults, which are the safe ones (delete off, ask on).
    pub fn load(data_dir: &Path) -> Permissions {
        std::fs::read_to_string(data_dir.join(PERMISSIONS_FILE))
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, data_dir: &Path) -> std::io::Result<()> {
        let text = serde_json::to_vec_pretty(self)?;
        super::token::write_private(&data_dir.join(PERMISSIONS_FILE), &text)
    }

    /// Checks that a tool that needs `need` may run. The error says which
    /// setting to change.
    pub fn check(&self, need: Need) -> Result<(), ToolError> {
        let (allowed, key, label) = match need {
            Need::Nothing => return Ok(()),
            Need::Create => (self.create, "create", "Create sprites"),
            Need::Edit => (self.edit, "edit", "Edit pixels"),
            Need::Delete => (self.delete, "delete", "Delete sprites"),
            Need::Export => (self.export, "export", "Export files"),
            Need::OpenProject => (self.open_project, "openProject", "Open projects"),
        };
        if allowed {
            return Ok(());
        }
        Err(ToolError::new(
            "permission_denied",
            format!("the permission \"{label}\" is off; ask the user to turn it on in the Agents tab of the Sprixel app"),
        )
        .with("permission", key))
    }
}
