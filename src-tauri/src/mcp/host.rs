//! What the MCP server needs from the app. The app gives it through Tauri
//! (`TauriHost`); the tests give it with plain values, so no window is needed.

use crate::ipc::editor::EditorState;
use crate::ipc::preferences::RecentProject;
use crate::ipc::project::{ProjectError, ProjectInfo, ProjectState};
use serde_json::Value;
use std::path::Path;

pub trait Host: Send + Sync + 'static {
    /// The open project.
    fn project(&self) -> &ProjectState;
    /// Who is writing now, and the Stop request.
    fn agents(&self) -> &EditorState;
    /// Tells the UI something happened (`project://changed`, `agent-activity`).
    fn emit(&self, event: &str, payload: Value);
    /// The recent projects of the person, newest first.
    fn recent_projects(&self) -> Vec<RecentProject>;
    /// Opens a project folder like the person does: the open project is
    /// closed and saved, and the folder enters the recent list.
    fn open_project(&self, path: &Path) -> Result<ProjectInfo, ProjectError>;
}

/// The host of the real app: the states are managed by Tauri.
pub struct TauriHost {
    pub app: tauri::AppHandle,
}

impl Host for TauriHost {
    fn project(&self) -> &ProjectState {
        let state: tauri::State<ProjectState> = tauri::Manager::state(&self.app);
        state.inner()
    }

    fn agents(&self) -> &EditorState {
        let state: tauri::State<EditorState> = tauri::Manager::state(&self.app);
        state.inner()
    }

    fn emit(&self, event: &str, payload: Value) {
        let _ = tauri::Emitter::emit(&self.app, event, payload);
    }

    fn recent_projects(&self) -> Vec<RecentProject> {
        crate::ipc::recent_projects(&self.app)
    }

    fn open_project(&self, path: &Path) -> Result<ProjectInfo, ProjectError> {
        let info = crate::ipc::project::open_project(self.project(), path)?;
        // A list that cannot be saved must not undo a project that did open.
        let _ = crate::ipc::change(&self.app, |data| {
            crate::ipc::preferences::add_recent(
                data,
                RecentProject { path: info.path.clone(), name: info.name.clone() },
            )
        });
        Ok(info)
    }
}
