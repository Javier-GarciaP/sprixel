#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    // Checks that the Rust test setup works and the package keeps its name.
    #[test]
    fn package_is_named_sprixel() {
        assert_eq!(env!("CARGO_PKG_NAME"), "sprixel");
    }
}
