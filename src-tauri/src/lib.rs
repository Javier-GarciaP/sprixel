use tauri::Manager;

mod analysis;
mod document;
mod export;
mod history;
mod mcp;
mod render;
mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // There is no window menu: every command is in the project menu inside the
    // app. Tauri 2 only creates a default menu on macOS (see `App::build`), so
    // Linux and Windows get no menu bar. macOS keeps that default menu on
    // purpose: macOS always shows a menu bar, and its Edit menu is what makes
    // Cmd+C, Cmd+V and Cmd+Q work in text fields.
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(ipc::project::ProjectState::default())
        .manage(ipc::editor::EditorState::default())
        .setup(|app| {
            // The MCP server starts with the app. If it cannot start (for
            // example the home folder is read-only) the app still works, only
            // without AI agents.
            match mcp::start(app.handle().clone()) {
                Ok((core, server)) => {
                    app.manage(core);
                    app.manage(std::sync::Mutex::new(Some(server)));
                }
                Err(error) => eprintln!("[mcp] the server did not start: {error}"),
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::frames::frames_info_of,
            ipc::frames::frames_insert,
            ipc::frames::frames_delete,
            ipc::frames::frames_move,
            ipc::frames::tag_add,
            ipc::frames::tag_set,
            ipc::frames::tag_remove,
            ipc::grid::grid_text_of,
            ipc::grid::grid_check,
            ipc::grid::grid_apply,
            ipc::palettes::palette_info_of,
            ipc::palettes::palette_apply,
            ipc::palettes::palette_presets,
            ipc::palettes::palette_create,
            ipc::palettes::palette_import,
            ipc::editor::editor_sprite,
            ipc::editor::editor_frame,
            ipc::editor::editor_apply,
            ipc::editor::editor_history,
            ipc::editor::agent_activity,
            ipc::editor::agent_stop,
            ipc::editor::agent_playback_done,
            ipc::editor::debug_agent_write,
            ipc::mcp::mcp_clients,
            ipc::mcp::mcp_answer_client,
            ipc::mcp::mcp_remove_client,
            ipc::mcp::mcp_permissions,
            ipc::mcp::mcp_set_permissions,
            ipc::mcp::mcp_activity,
            ipc::mcp::mcp_stop_agent,
            ipc::mcp::mcp_resume_agent,
            ipc::load_preferences,
            ipc::save_settings,
            ipc::add_recent_project,
            ipc::remove_recent_project,
            ipc::folder_exists,
            ipc::default_project_location,
            ipc::create_project,
            ipc::open_project,
            ipc::close_project,
            ipc::project_status,
            ipc::retry_save,
            ipc::project_tree,
            ipc::sprite_frame,
            ipc::check_name,
            ipc::create_sprite,
            ipc::create_folder,
            ipc::rename_sprite,
            ipc::rename_folder,
            ipc::duplicate_sprite,
            ipc::delete_sprites,
            ipc::delete_folder,
            ipc::move_sprites,
            ipc::project_undo,
            ipc::project_redo,
            ipc::export_summary,
            ipc::export_preview,
            ipc::export_run,
            ipc::export_reveal
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // Stop the server when the app ends, so `runtime.json` is removed.
            if let tauri::RunEvent::Exit = event {
                if let Some(server) = app.try_state::<std::sync::Mutex<Option<mcp::Server>>>() {
                    server.lock().unwrap().take();
                }
            }
        });
}

// The shell modules (the storage modules are declared at the top of the file).
mod ipc;

#[cfg(test)]
mod tests {
    // Checks that the Rust test setup works and the package keeps its name.
    #[test]
    fn package_is_named_sprixel() {
        assert_eq!(env!("CARGO_PKG_NAME"), "sprixel");
    }
}
