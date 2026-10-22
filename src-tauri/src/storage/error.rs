use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    /// The disk refused an operation (no space, no permission, missing file).
    Io { path: PathBuf, message: String },
    /// A file exists but its content is not valid.
    Invalid { path: PathBuf, reason: String },
}

impl StorageError {
    pub fn io(path: &Path, error: &std::io::Error) -> StorageError {
        StorageError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    }

    pub fn invalid(path: &Path, reason: impl Into<String>) -> StorageError {
        StorageError::Invalid {
            path: path.to_path_buf(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Io { path, message } => write!(f, "{}: {message}", path.display()),
            StorageError::Invalid { path, reason } => write!(f, "{}: {reason}", path.display()),
        }
    }
}

impl std::error::Error for StorageError {}
