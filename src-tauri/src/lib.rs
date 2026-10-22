#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_menu_event(|app, event| menu::forward(app, &event.id().0))
        .invoke_handler(tauri::generate_handler![
            ipc::set_menu,
            ipc::load_preferences,
            ipc::save_settings,
            ipc::add_recent_project,
            ipc::remove_recent_project,
            ipc::folder_exists
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// The shell modules (the storage modules are declared at the top of the file).
mod ipc;
mod menu;

#[cfg(test)]
mod tests {
    // Checks that the Rust test setup works and the package keeps its name.
    #[test]
    fn package_is_named_sprixel() {
        assert_eq!(env!("CARGO_PKG_NAME"), "sprixel");
    }
}
