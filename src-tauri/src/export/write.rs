//! Writes the exported files into the project folder, all or nothing.

use super::{ExportError, ExportFile};
use crate::storage::FileWriter;
use std::io;
use std::path::{Path, PathBuf};

/// The default output folder, inside the project.
pub const DEFAULT_OUTPUT: &str = "assets/sprites";

/// Where the files went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    /// The output folder on the disk.
    pub folder: PathBuf,
    /// The output folder as the user wrote it, with `/` as separator.
    pub output: String,
    /// The names of the files written.
    pub files: Vec<String>,
}

/// The folder `output` (written relative to the project) as a path inside
/// `root`. It never leaves the project: no absolute path, no drive, no `..`,
/// and no symbolic link that points outside.
pub fn resolve_output(root: &Path, output: &str) -> Result<(PathBuf, String), ExportError> {
    let outside = || ExportError::new("outside_project", "the output folder must be inside the project");
    if output.contains(':') || output.starts_with(['/', '\\']) {
        return Err(outside());
    }
    let mut folder = root.to_path_buf();
    let mut parts = Vec::new();
    for part in output.split(['/', '\\']).filter(|p| !p.is_empty() && *p != ".") {
        if part == ".." {
            return Err(outside());
        }
        // `.sprixel` holds the trash and the temporary files of the app.
        if parts.is_empty() && part.eq_ignore_ascii_case(".sprixel") {
            return Err(ExportError::new("invalid_path", "the folder .sprixel is reserved"));
        }
        folder.push(part);
        parts.push(part);
    }
    if parts.is_empty() {
        return Err(ExportError::new("invalid_path", "the output folder is empty"));
    }
    // A link inside the project can point outside: look at the real path of
    // the deepest folder that already exists.
    let existing = folder.ancestors().find(|p| p.exists()).unwrap_or(root);
    let real_root = root.canonicalize().map_err(io_error)?;
    if !existing.canonicalize().map_err(io_error)?.starts_with(&real_root) {
        return Err(outside());
    }
    Ok((folder, parts.join("/")))
}

/// Maps a system error to a stable code the interface can translate.
fn io_error(error: io::Error) -> ExportError {
    let code = match error.kind() {
        io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem => "no_permission",
        io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded => "no_space",
        _ => "io",
    };
    ExportError::new(code, error.to_string())
}

/// Writes `files` into the folder `output` of the project at `root`.
///
/// The files are first written to a temporary folder inside the output folder
/// and then renamed, so when anything fails no file is left half written, and
/// the files of an earlier export are replaced only when all of the new ones
/// are ready.
pub fn write_files(
    writer: &dyn FileWriter,
    root: &Path,
    output: &str,
    files: &[ExportFile],
) -> Result<Written, ExportError> {
    let (folder, output) = resolve_output(root, output)?;
    let existed = folder.exists();
    std::fs::create_dir_all(&folder).map_err(io_error)?;
    let staging = folder.join(".sprixel-export-tmp");
    let result = stage_and_move(writer, &staging, &folder, files);
    let _ = std::fs::remove_dir_all(&staging);
    if let Err(error) = result {
        if !existed {
            // Only removes the folder when it is empty.
            let _ = std::fs::remove_dir(&folder);
        }
        return Err(error);
    }
    Ok(Written { folder, output, files: files.iter().map(|f| f.name.clone()).collect() })
}

fn stage_and_move(
    writer: &dyn FileWriter,
    staging: &Path,
    folder: &Path,
    files: &[ExportFile],
) -> Result<(), ExportError> {
    let _ = std::fs::remove_dir_all(staging);
    std::fs::create_dir(staging).map_err(io_error)?;
    for file in files {
        writer.write(&staging.join(&file.name), &file.bytes).map_err(io_error)?;
    }
    for file in files {
        std::fs::rename(staging.join(&file.name), folder.join(&file.name)).map_err(io_error)?;
    }
    Ok(())
}
