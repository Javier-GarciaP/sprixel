//! What the Agents tab asks of the MCP server: the clients and their
//! approval. The server decides; these commands only pass the answers of the
//! person to it.

use crate::mcp::{ActivityRow, ClientInfo, Core, Permissions};
use std::sync::Arc;
use tauri::Manager;

/// The server of the app, or the reason it is not there (it did not start).
fn core(app: &tauri::AppHandle) -> Result<Arc<Core>, String> {
    app.try_state::<Arc<Core>>()
        .map(|core| core.inner().clone())
        .ok_or_else(|| "the MCP server is not running".to_string())
}

/// The clients that connected, with their state.
#[tauri::command]
pub fn mcp_clients(app: tauri::AppHandle) -> Result<Vec<ClientInfo>, String> {
    Ok(core(&app)?.clients())
}

/// Allow (`true`) or reject (`false`) a client that waits.
#[tauri::command]
pub fn mcp_answer_client(
    app: tauri::AppHandle,
    name: String,
    version: String,
    allow: bool,
) -> Result<bool, String> {
    core(&app)?.answer_client(&name, &version, allow).map_err(|e| e.to_string())
}

/// Forget a client. It has to ask again.
#[tauri::command]
pub fn mcp_remove_client(app: tauri::AppHandle, name: String, version: String) -> Result<bool, String> {
    core(&app)?.remove_client(&name, &version).map_err(|e| e.to_string())
}

/// The permissions agents have now.
#[tauri::command]
pub fn mcp_permissions(app: tauri::AppHandle) -> Result<Permissions, String> {
    Ok(core(&app)?.permissions())
}

/// Saves the permissions. They apply to the next call of every agent.
#[tauri::command]
pub fn mcp_set_permissions(app: tauri::AppHandle, permissions: Permissions) -> Result<(), String> {
    core(&app)?.set_permissions(permissions).map_err(|e| e.to_string())
}

/// What the agents did in this session, newest first. Only the most recent AI
/// action can have `can_undo`.
#[tauri::command]
pub fn mcp_activity(app: tauri::AppHandle) -> Result<Vec<ActivityRow>, String> {
    Ok(core(&app)?.activity())
}

/// Stop an agent: its work in progress is cancelled and it cannot write.
#[tauri::command]
pub fn mcp_stop_agent(app: tauri::AppHandle, name: String) -> Result<(), String> {
    core(&app)?.stop_agent(&name);
    Ok(())
}

/// Let a stopped agent continue.
#[tauri::command]
pub fn mcp_resume_agent(app: tauri::AppHandle, name: String) -> Result<(), String> {
    core(&app)?.resume_agent(&name);
    Ok(())
}
