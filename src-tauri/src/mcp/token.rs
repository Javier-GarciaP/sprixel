//! The access token: the secret that every request must carry. It is random,
//! it lives in a file that only the current user can read, and it is compared
//! in constant time. It is never logged.

use super::random::random_hex;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// The token has 32 random bytes (256 bits), written as 64 hex characters.
/// The spec asks for at least 128 bits.
pub const TOKEN_BYTES: usize = 32;

/// The file name of the token inside the Sprixel data folder.
pub const TOKEN_FILE: &str = "token";

pub fn token_path(data_dir: &Path) -> PathBuf {
    data_dir.join(TOKEN_FILE)
}

/// Reads the token of the data folder, or makes a new one the first time (or
/// when the file is damaged). To rotate the token, delete the file.
pub fn load_or_create(data_dir: &Path) -> io::Result<String> {
    make_private_dir(data_dir)?;
    let path = token_path(data_dir);
    if let Ok(text) = fs::read_to_string(&path) {
        let token = text.trim().to_string();
        if is_valid(&token) {
            // The file may have been copied or changed by hand: close it again.
            restrict(&path)?;
            return Ok(token);
        }
    }
    let token = random_hex(TOKEN_BYTES);
    write_private(&path, token.as_bytes())?;
    Ok(token)
}

fn is_valid(token: &str) -> bool {
    token.len() == TOKEN_BYTES * 2 && token.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Compares two secrets in constant time: the time does not depend on where
/// the first different byte is, so a program cannot guess the token one
/// character at a time.
pub fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = (a.len() != b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        diff |= a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0);
    }
    diff == 0
}

/// Creates the data folder (and its parents) readable only by the user.
pub fn make_private_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    restrict(dir)
}

/// Writes a file that only the current user can read. The file is created
/// with the closed permissions from the start (no moment when others could
/// read it), then renamed over the old one.
pub fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    let _ = fs::remove_file(&temporary);
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let result = (|| {
        let mut file = options.open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        restrict(&temporary)?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

/// Makes a file or folder private to the current user: mode 0600 (0700 for a
/// folder) on Linux and macOS, and an access list with only the user on
/// Windows.
fn restrict(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if path.is_dir() { 0o700 } else { 0o600 };
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }
    #[cfg(windows)]
    {
        // `icacls` ships with Windows. `/inheritance:r` drops the rights
        // inherited from the parent folder; then only the user is granted.
        let user = std::env::var("USERNAME").map_err(|e| io::Error::other(e.to_string()))?;
        let grant = format!("{user}:(OI)(CI)F");
        let status = std::process::Command::new("icacls")
            .arg(path)
            .args(["/inheritance:r", "/grant:r", &grant])
            .stdout(std::process::Stdio::null())
            .status()?;
        if status.success() { Ok(()) } else { Err(io::Error::other("icacls failed")) }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Checks the "Token" requirement: the token is random, has at least 128
    // bits (here 256) and is stored in a file.
    #[test]
    fn a_new_token_is_long_random_and_stored() {
        let dir = tempfile::tempdir().unwrap();
        let token = load_or_create(dir.path()).unwrap();
        assert!(token.len() / 2 >= 16, "at least 128 bits");
        assert_eq!(token.len(), TOKEN_BYTES * 2);
        assert_eq!(fs::read_to_string(token_path(dir.path())).unwrap().trim(), token);

        let other = tempfile::tempdir().unwrap();
        assert_ne!(token, load_or_create(other.path()).unwrap());
    }

    // Checks that the token stays the same after a restart, that a damaged
    // file is replaced, and that deleting the file rotates the token.
    #[test]
    fn the_token_is_kept_replaced_when_damaged_and_rotated_by_deleting() {
        let dir = tempfile::tempdir().unwrap();
        let first = load_or_create(dir.path()).unwrap();
        assert_eq!(first, load_or_create(dir.path()).unwrap());

        fs::write(token_path(dir.path()), "abc").unwrap();
        let repaired = load_or_create(dir.path()).unwrap();
        assert_eq!(repaired.len(), TOKEN_BYTES * 2);
        assert_ne!(repaired, "abc");

        fs::remove_file(token_path(dir.path())).unwrap();
        assert_ne!(first, load_or_create(dir.path()).unwrap());
    }

    // Checks the permissions of the token file: only the user can read it.
    // On Linux and macOS the mode is exactly 0600, and the folder is 0700.
    #[cfg(unix)]
    #[test]
    fn the_token_file_is_private_to_the_user() {
        use std::os::unix::fs::PermissionsExt;
        let parent = tempfile::tempdir().unwrap();
        let dir = parent.path().join(".sprixel");

        load_or_create(&dir).unwrap();

        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&token_path(&dir)), 0o600);
        assert_eq!(mode(&dir), 0o700);
    }

    // Checks that a token file that was opened to everybody is closed again
    // when the app starts.
    #[cfg(unix)]
    #[test]
    fn an_open_token_file_is_closed_again() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        load_or_create(dir.path()).unwrap();
        let path = token_path(dir.path());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

        load_or_create(dir.path()).unwrap();

        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    }

    // Checks the constant-time comparison gives the right answers.
    #[test]
    fn same_compares_secrets() {
        assert!(same("abcdef", "abcdef"));
        assert!(!same("abcdef", "abcdeg"));
        assert!(!same("abcdef", "abcde"));
        assert!(!same("", "a"));
        assert!(same("", ""));
    }
}
