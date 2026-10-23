use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Writes the bytes of a file. It is a trait so that tests can make a write
/// fail halfway (a full disk, a crash) without breaking a real disk.
pub trait FileWriter: Send + Sync {
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;
}

/// The real writer: creates the file, writes and flushes it to the disk.
pub struct DiskWriter;

impl FileWriter for DiskWriter {
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(bytes)?;
        file.sync_all()
    }
}

/// Saves a file so that a crash or a power loss never leaves it half written:
/// the bytes go to `name.tmp` in the same folder, then the temporary file
/// replaces the old one with a rename. On failure the old file stays as it was.
pub fn write_atomic(writer: &dyn FileWriter, path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temporary = temporary_path(path);
    let result = writer
        .write(&temporary, bytes)
        .and_then(|()| std::fs::rename(&temporary, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}
