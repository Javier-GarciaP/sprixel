//! What the MCP server needs from the app. The app gives it through Tauri
//! (`TauriHost`); the tests give it with plain values, so no window is needed.

use crate::ipc::editor::EditorState;
use crate::ipc::project::ProjectState;
use serde_json::Value;

pub trait Host: Send + Sync + 'static {
    /// The open project.
    fn project(&self) -> &ProjectState;
    /// Who is writing now, and the Stop request.
    fn agents(&self) -> &EditorState;
    /// Tells the UI something happened (`project://changed`, `agent-activity`).
    fn emit(&self, event: &str, payload: Value);
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
}
