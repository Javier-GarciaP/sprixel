#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // There is no window menu: every command is in the project menu inside the
    // app. Tauri 2 only creates a default menu on macOS (see `App::build`), so
    // Linux and Windows get no menu bar. macOS keeps that default menu on
    // purpose: macOS always shows a menu bar, and its Edit menu is what makes
    // Cmd+C, Cmd+V and Cmd+Q work in text fields.
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
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

#[cfg(test)]
mod tests {
    // Checks that the Rust test setup works and the package keeps its name.
    #[test]
    fn package_is_named_sprixel() {
        assert_eq!(env!("CARGO_PKG_NAME"), "sprixel");
    }
}
