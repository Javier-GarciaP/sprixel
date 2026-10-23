use super::error::DocError;

const MAX_NAME_CHARS: usize = 200;
const INVALID_CHARS: [char; 9] = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
const RESERVED: [&str; 4] = ["CON", "PRN", "AUX", "NUL"];

/// Checks that a sprite, folder or palette name works as a file name on
/// Linux, Windows and macOS.
pub fn validate_name(name: &str) -> Result<(), DocError> {
    let bad = |reason: &str| Err(DocError::new("invalid_name", format!("{name:?} {reason}")));
    if name.trim().is_empty() {
        return bad("is empty");
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return bad("is too long");
    }
    if name.chars().any(|c| c.is_control() || INVALID_CHARS.contains(&c)) {
        return bad("has a character that is not allowed in file names");
    }
    // Windows removes spaces and dots at the end, and "." and ".." are folders.
    if name.starts_with(' ') || name.ends_with(' ') || name.ends_with('.') {
        return bad("starts with a space or ends with a space or a dot");
    }
    if is_reserved_on_windows(name) {
        return bad("is a reserved name on Windows");
    }
    Ok(())
}

/// Two names are the same when they differ only in upper and lower case.
pub fn same_name(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

// Windows reserves these names with or without an extension ("CON.txt").
fn is_reserved_on_windows(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).to_uppercase();
    if RESERVED.contains(&stem.as_str()) {
        return true;
    }
    let numbered = |prefix: &str| {
        stem.strip_prefix(prefix)
            .is_some_and(|n| matches!(n.as_bytes(), [b'1'..=b'9']))
    };
    numbered("COM") || numbered("LPT")
}
