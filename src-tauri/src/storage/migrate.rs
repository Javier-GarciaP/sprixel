use super::atomic::DiskWriter;
use super::error::StorageError;
use super::files::{manifest_path, read_text, write_text};
use super::format::{FORMAT_VERSION, canonical_text, version_of};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Manifest,
    Palette,
    Sprite,
}

/// Upgrades the JSON of one file from version `from` to `from + 1`.
pub struct Step {
    pub from: u32,
    pub run: fn(&mut Value, FileKind) -> Result<(), String>,
}

/// The list of steps that bring old files to `FORMAT_VERSION`. The app has no
/// older version yet, so `Migrator::app()` is empty. Tests build their own.
pub struct Migrator {
    steps: Vec<Step>,
}

/// What a migration did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub from_version: u32,
    /// The folder in `.sprixel/backups/` with the files before the migration.
    pub backup: PathBuf,
}

impl Migrator {
    pub fn app() -> Migrator {
        Migrator { steps: Vec::new() }
    }

    pub fn new(steps: Vec<Step>) -> Migrator {
        Migrator { steps }
    }

    /// Returns the text of the file in the current version, or `None` if it
    /// already is.
    fn upgrade_text(&self, text: &str, kind: FileKind) -> Result<Option<String>, String> {
        let mut value: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let mut version = version_of(&value)?;
        if version == FORMAT_VERSION {
            return Ok(None);
        }
        if version > FORMAT_VERSION {
            return Err(format!(
                "saved with a newer Sprixel (format version {version}); update the app"
            ));
        }
        while version < FORMAT_VERSION {
            let step = self
                .steps
                .iter()
                .find(|s| s.from == version)
                .ok_or(format!("no migration from format version {version}"))?;
            (step.run)(&mut value, kind)?;
            version += 1;
            value["formatVersion"] = Value::from(version);
        }
        canonical_text(kind, value).map(Some)
    }
}

/// Brings a project folder to the current format version.
///
/// - Current version: nothing happens, returns `None`.
/// - Newer version: `NewerVersion` and nothing is touched.
/// - Older version: the project files are copied to `.sprixel/backups/`,
///   then every file is migrated. A file that cannot be migrated (damaged)
///   is left as it is, to be reported when the project opens. The manifest is
///   written last, so an interrupted migration runs again on the next open.
pub fn migrate_project(root: &Path, migrator: &Migrator) -> Result<Option<Migration>, StorageError> {
    let manifest = manifest_path(root);
    let text = read_text(&manifest)?;
    let value: Value = serde_json::from_str(&text).map_err(|e| StorageError::invalid(&manifest, e.to_string()))?;
    let from_version = version_of(&value).map_err(|r| StorageError::invalid(&manifest, r))?;
    if from_version > FORMAT_VERSION {
        return Err(StorageError::NewerVersion {
            found: from_version,
            current: FORMAT_VERSION,
        });
    }
    let new_manifest = match migrator
        .upgrade_text(&text, FileKind::Manifest)
        .map_err(|r| StorageError::invalid(&manifest, r))?
    {
        Some(new_text) => new_text,
        None => return Ok(None),
    };

    let backup = make_backup(root, from_version)?;
    let mut files = Vec::new();
    collect(&root.join("palettes"), ".palette.json", FileKind::Palette, &mut files);
    collect(&root.join("sprites"), ".sprite.json", FileKind::Sprite, &mut files);
    for (path, kind) in files {
        let upgraded = read_text(&path).ok().and_then(|t| migrator.upgrade_text(&t, kind).ok());
        if let Some(Some(new_text)) = upgraded {
            write_text(&DiskWriter, &path, &new_text)?;
        }
    }
    write_text(&DiskWriter, &manifest, &new_manifest)?;
    Ok(Some(Migration {
        from_version,
        backup,
    }))
}

// Lists the files with a suffix under a folder (the folder may not exist).
fn collect(dir: &Path, suffix: &str, kind: FileKind, found: &mut Vec<(PathBuf, FileKind)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, suffix, kind, found);
        } else if path.to_string_lossy().ends_with(suffix) {
            found.push((path, kind));
        }
    }
}

fn make_backup(root: &Path, from_version: u32) -> Result<PathBuf, StorageError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let folder = root.join(".sprixel").join("backups");
    let mut backup = folder.join(format!("v{from_version}-{seconds}"));
    let mut n = 2;
    while backup.exists() {
        backup = folder.join(format!("v{from_version}-{seconds}-{n}"));
        n += 1;
    }
    copy_dir(root, &backup, &["sprixel.json", "palettes", "sprites"])?;
    Ok(backup)
}

// Copies the listed entries of `from` into `to`.
fn copy_dir(from: &Path, to: &Path, only: &[&str]) -> Result<(), StorageError> {
    fs::create_dir_all(to).map_err(|e| StorageError::io(to, &e))?;
    for name in only {
        copy_entry(&from.join(name), &to.join(name))?;
    }
    Ok(())
}

fn copy_entry(from: &Path, to: &Path) -> Result<(), StorageError> {
    if from.is_dir() {
        fs::create_dir_all(to).map_err(|e| StorageError::io(to, &e))?;
        let entries = fs::read_dir(from).map_err(|e| StorageError::io(from, &e))?;
        for entry in entries.flatten() {
            copy_entry(&entry.path(), &to.join(entry.file_name()))?;
        }
    } else if from.is_file() {
        fs::copy(from, to).map_err(|e| StorageError::io(from, &e))?;
    }
    Ok(())
}
