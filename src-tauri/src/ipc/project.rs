//! The open project. The Rust core owns it; the UI only asks to create, open
//! or close it and reads the save status. The functions here do not need a
//! window, so the tests can call them directly.

use crate::document::validate_name;
use crate::storage::{Autosave, DEFAULT_DEBOUNCE, SaveStatus, Session, StorageError};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;

struct OpenProject {
    // Dropping it flushes the last save.
    autosave: Autosave,
}

/// The managed state: the project that is open, if any.
#[derive(Default)]
pub struct ProjectState(Mutex<Option<OpenProject>>);

/// What the UI needs after creating or opening a project.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    /// Files that could not be loaded. The project opens without them.
    pub damaged_files: usize,
}

/// Errors cross to the UI as stable codes, so the UI can translate them.
/// The codes are `folder_exists`, `not_a_project`, `invalid_name` and `io`.
#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ProjectError {
    /// The folder `parent/name` already exists.
    FolderExists,
    /// The folder has no `sprixel.json`.
    NotAProject,
    /// The name cannot be used as a folder name. `message` says why.
    InvalidName { message: String },
    /// Anything else. `message` is the message of the system.
    Io { message: String },
}

impl From<StorageError> for ProjectError {
    fn from(error: StorageError) -> Self {
        ProjectError::Io {
            message: error.to_string(),
        }
    }
}

/// The save status as the UI reads it.
#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum StatusInfo {
    Saved,
    Saving,
    Error { reason: String },
}

impl From<SaveStatus> for StatusInfo {
    fn from(status: SaveStatus) -> Self {
        match status {
            SaveStatus::Saved => StatusInfo::Saved,
            SaveStatus::Saving => StatusInfo::Saving,
            SaveStatus::Error(reason) => StatusInfo::Error { reason },
        }
    }
}

fn info(session: &Session) -> ProjectInfo {
    ProjectInfo {
        name: session.project.name.clone(),
        path: session.root().to_string_lossy().into_owned(),
        damaged_files: session.damaged().len(),
    }
}

// Makes the session the open project. The project that was open is dropped,
// which saves its last changes.
fn install(state: &ProjectState, session: Session) {
    let autosave = Autosave::start(session, DEFAULT_DEBOUNCE);
    *state.0.lock().unwrap() = Some(OpenProject { autosave });
}

/// Creates `parent/name` and opens it. Nothing is created when the name is
/// not valid or the folder already exists, and the open project stays open.
pub fn create_project(
    state: &ProjectState,
    parent: &Path,
    name: &str,
) -> Result<ProjectInfo, ProjectError> {
    validate_name(name).map_err(|e| ProjectError::InvalidName { message: e.message })?;
    if parent.join(name).exists() {
        return Err(ProjectError::FolderExists);
    }
    let session = Session::create(parent, name)?;
    let result = info(&session);
    install(state, session);
    Ok(result)
}

/// Opens a project folder. The folder must have a `sprixel.json`.
pub fn open_project(state: &ProjectState, path: &Path) -> Result<ProjectInfo, ProjectError> {
    if !path.join("sprixel.json").is_file() {
        return Err(ProjectError::NotAProject);
    }
    // Close the open project first: it may be the same folder, and its last
    // save must be on the disk before the folder is read.
    close_project(state);
    let session = Session::open(path)?;
    let result = info(&session);
    install(state, session);
    Ok(result)
}

/// Looks at the open project without changing it. `None` when none is open.
pub fn read<R>(state: &ProjectState, look: impl FnOnce(&Session) -> R) -> Option<R> {
    let open = state.0.lock().unwrap();
    open.as_ref().map(|p| p.autosave.read(look))
}

/// Changes the open project and restarts the save timer. `None` when none is open.
pub fn edit<R>(state: &ProjectState, change: impl FnOnce(&mut Session) -> R) -> Option<R> {
    let open = state.0.lock().unwrap();
    open.as_ref().map(|p| p.autosave.edit(change))
}

pub fn close_project(state: &ProjectState) {
    state.0.lock().unwrap().take();
}

/// The save status of the open project, or `None` when no project is open.
pub fn project_status(state: &ProjectState) -> Option<StatusInfo> {
    let open = state.0.lock().unwrap();
    open.as_ref().map(|p| p.autosave.status().into())
}

/// The "Retry" action after a save error.
pub fn retry_save(state: &ProjectState) -> Option<StatusInfo> {
    let open = state.0.lock().unwrap();
    open.as_ref().map(|p| p.autosave.retry().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Operation;
    use std::fs;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    // Checks the spec scenario: creating a project makes the folder, with its
    // manifest, and the project is open.
    #[test]
    fn create_makes_the_folder_and_opens_it() {
        let parent = tempfile::tempdir().unwrap();
        let state = ProjectState::default();

        let created = create_project(&state, parent.path(), "Dungeon Kit").unwrap();

        assert_eq!(created.name, "Dungeon Kit");
        assert!(parent.path().join("Dungeon Kit").join("sprixel.json").is_file());
        assert_eq!(created.damaged_files, 0);
        assert_eq!(project_status(&state), Some(StatusInfo::Saved));
    }

    // Checks that creating, closing and opening again gives the same project
    // (an end-to-end round trip through the storage layer).
    #[test]
    fn create_close_open_round_trip() {
        let parent = tempfile::tempdir().unwrap();
        let state = ProjectState::default();
        let created = create_project(&state, parent.path(), "Kit").unwrap();

        close_project(&state);
        assert_eq!(project_status(&state), None);

        let opened = open_project(&state, Path::new(&created.path)).unwrap();
        assert_eq!(opened, created);
        assert_eq!(project_status(&state), Some(StatusInfo::Saved));
    }

    // Checks the spec scenario: a folder that exists gives `folder_exists`
    // and nothing is changed inside it.
    #[test]
    fn create_in_an_existing_folder_is_refused() {
        let parent = tempfile::tempdir().unwrap();
        let existing = parent.path().join("Kit");
        fs::create_dir(&existing).unwrap();
        fs::write(existing.join("notes.txt"), "keep me").unwrap();
        let state = ProjectState::default();

        let result = create_project(&state, parent.path(), "Kit");

        assert_eq!(result, Err(ProjectError::FolderExists));
        assert!(!existing.join("sprixel.json").exists());
        assert_eq!(project_status(&state), None);
    }

    // Checks that a name that cannot be a folder name gives `invalid_name`
    // and creates nothing.
    #[test]
    fn create_with_a_bad_name_is_refused() {
        let parent = tempfile::tempdir().unwrap();
        let state = ProjectState::default();

        for bad in ["", "   ", "a/b", "CON"] {
            let result = create_project(&state, parent.path(), bad);
            assert!(
                matches!(result, Err(ProjectError::InvalidName { .. })),
                "{bad:?}"
            );
        }
        assert_eq!(fs::read_dir(parent.path()).unwrap().count(), 0);
    }

    // Checks the spec scenario: a folder without sprixel.json gives
    // `not_a_project` and nothing is opened.
    #[test]
    fn open_a_folder_that_is_not_a_project() {
        let folder = tempfile::tempdir().unwrap();
        let state = ProjectState::default();

        let result = open_project(&state, folder.path());

        assert_eq!(result, Err(ProjectError::NotAProject));
        assert_eq!(project_status(&state), None);
    }

    // Checks that a failed open does not close the project that is open.
    #[test]
    fn a_refused_open_keeps_the_open_project() {
        let parent = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let state = ProjectState::default();
        create_project(&state, parent.path(), "Kit").unwrap();

        assert!(open_project(&state, other.path()).is_err());

        assert_eq!(project_status(&state), Some(StatusInfo::Saved));
    }

    // Checks that opening a project while another is open closes the first
    // and leaves the second open.
    #[test]
    fn opening_a_project_closes_the_previous_one() {
        let parent = tempfile::tempdir().unwrap();
        let state = ProjectState::default();
        create_project(&state, parent.path(), "One").unwrap();
        let two = create_project(&state, parent.path(), "Two").unwrap();

        let reopened = open_project(&state, &parent.path().join("One")).unwrap();

        assert_eq!(reopened.name, "One");
        assert_ne!(reopened.path, two.path);
    }

    /// Waits until the condition is true, or fails after the timeout.
    fn wait_until(timeout: Duration, mut condition: impl FnMut() -> bool) {
        let start = Instant::now();
        while !condition() {
            assert!(start.elapsed() < timeout, "timed out");
            sleep(Duration::from_millis(5));
        }
    }

    // Checks that after a change the status is Saving and then becomes Saved
    // by itself (waiting by polling, not with a fixed sleep).
    #[test]
    fn the_status_becomes_saved_after_a_change() {
        let parent = tempfile::tempdir().unwrap();
        let state = ProjectState::default();
        create_project(&state, parent.path(), "Kit").unwrap();

        {
            let open = state.0.lock().unwrap();
            open.as_ref().unwrap().autosave.edit(|session| {
                session
                    .apply(
                        "user",
                        vec![Operation::CreateFolder {
                            parent: vec![],
                            name: "Personajes".to_string(),
                        }],
                    )
                    .unwrap()
            });
        }
        assert_eq!(project_status(&state), Some(StatusInfo::Saving));

        wait_until(Duration::from_secs(5), || {
            project_status(&state) == Some(StatusInfo::Saved)
        });
    }

    // Checks that closing the project saves the last change at once: the
    // folder is on the disk without waiting for the quiet time.
    #[test]
    fn closing_saves_the_last_change() {
        let parent = tempfile::tempdir().unwrap();
        let state = ProjectState::default();
        let created = create_project(&state, parent.path(), "Kit").unwrap();
        {
            let open = state.0.lock().unwrap();
            open.as_ref().unwrap().autosave.edit(|session| {
                session
                    .apply(
                        "user",
                        vec![Operation::CreateFolder {
                            parent: vec![],
                            name: "Personajes".to_string(),
                        }],
                    )
                    .unwrap()
            });
        }

        close_project(&state);

        assert!(Path::new(&created.path).join("sprites").join("Personajes").is_dir());
    }

    // Checks the wire format the UI reads: stable error codes and status tags.
    #[test]
    fn errors_and_status_serialize_to_stable_codes() {
        let json = |value: &dyn erased::Json| value.json();
        assert_eq!(json(&ProjectError::FolderExists), r#"{"code":"folder_exists"}"#);
        assert_eq!(json(&ProjectError::NotAProject), r#"{"code":"not_a_project"}"#);
        assert_eq!(
            json(&ProjectError::InvalidName { message: "x".into() }),
            r#"{"code":"invalid_name","message":"x"}"#
        );
        assert_eq!(
            json(&ProjectError::Io { message: "y".into() }),
            r#"{"code":"io","message":"y"}"#
        );
        assert_eq!(json(&StatusInfo::Saved), r#"{"status":"saved"}"#);
        assert_eq!(
            json(&StatusInfo::Error { reason: "full".into() }),
            r#"{"status":"error","reason":"full"}"#
        );
    }

    mod erased {
        pub trait Json {
            fn json(&self) -> String;
        }
        impl<T: serde::Serialize> Json for T {
            fn json(&self) -> String {
                serde_json::to_string(self).unwrap()
            }
        }
    }
}
