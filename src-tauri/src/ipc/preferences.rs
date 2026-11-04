//! Personal preferences and the recent projects list. They live in one small
//! JSON file in the app config folder of the user, never inside a project.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs, io};

/// How many recent projects are kept.
const MAX_RECENT: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: String,
    pub name: String,
}

/// The settings of the Settings screen. `None` means "not chosen yet", so the
/// UI uses its default (the system language, the system theme, grid on).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub language: Option<String>,
    pub theme: Option<String>,
    pub pixel_grid: Option<bool>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Data {
    pub settings: Settings,
    pub recent_projects: Vec<RecentProject>,
}

/// Reads the file. A missing or broken file gives the defaults, so the app
/// always starts.
pub fn load(file: &Path) -> Data {
    fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Writes the file atomically: a temporary file first, then a rename.
pub fn save(file: &Path, data: &Data) -> io::Result<()> {
    if let Some(folder) = file.parent() {
        fs::create_dir_all(folder)?;
    }
    let temporary = file.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_string_pretty(data)?)?;
    fs::rename(&temporary, file)
}

/// Puts a project first in the list. An older entry with the same path goes away.
pub fn add_recent(data: &mut Data, project: RecentProject) {
    data.recent_projects.retain(|p| p.path != project.path);
    data.recent_projects.insert(0, project);
    data.recent_projects.truncate(MAX_RECENT);
}

pub fn remove_recent(data: &mut Data, path: &str) {
    data.recent_projects.retain(|p| p.path != path);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(path: &str) -> RecentProject {
        RecentProject {
            path: path.into(),
            name: path.into(),
        }
    }

    fn temp_file(name: &str) -> std::path::PathBuf {
        let folder = std::env::temp_dir().join(format!("sprixel-test-{name}"));
        let _ = fs::remove_dir_all(&folder);
        folder.join("preferences.json")
    }

    // Checks the spec scenario: the most recent project is listed first.
    #[test]
    fn the_last_opened_project_goes_first() {
        let mut data = Data::default();
        add_recent(&mut data, project("/a"));
        add_recent(&mut data, project("/b"));
        add_recent(&mut data, project("/c"));

        let paths: Vec<_> = data.recent_projects.iter().map(|p| p.path.as_str()).collect();
        assert_eq!(paths, ["/c", "/b", "/a"]);
    }

    // Checks that opening a project again moves it to the top, with no duplicate.
    #[test]
    fn opening_again_moves_it_to_the_top() {
        let mut data = Data::default();
        add_recent(&mut data, project("/a"));
        add_recent(&mut data, project("/b"));
        add_recent(&mut data, project("/a"));

        let paths: Vec<_> = data.recent_projects.iter().map(|p| p.path.as_str()).collect();
        assert_eq!(paths, ["/a", "/b"]);
    }

    // Checks that the list keeps only the latest entries.
    #[test]
    fn the_list_is_capped() {
        let mut data = Data::default();
        for i in 0..(MAX_RECENT + 5) {
            add_recent(&mut data, project(&format!("/p{i}")));
        }
        assert_eq!(data.recent_projects.len(), MAX_RECENT);
        assert_eq!(data.recent_projects[0].path, format!("/p{}", MAX_RECENT + 4));
    }

    // Checks that an entry can be removed from the list (moved-folder case).
    #[test]
    fn an_entry_can_be_removed() {
        let mut data = Data::default();
        add_recent(&mut data, project("/a"));
        add_recent(&mut data, project("/b"));
        remove_recent(&mut data, "/a");
        assert_eq!(data.recent_projects, [project("/b")]);
    }

    // Checks that what is saved is read back the same after a "restart"
    // (a new load from disk), including the settings.
    #[test]
    fn saved_data_is_read_back() {
        let file = temp_file("round-trip");
        let mut data = Data::default();
        data.settings.theme = Some("dark".into());
        data.settings.language = Some("es".into());
        data.settings.pixel_grid = Some(false);
        add_recent(&mut data, project("/a"));

        save(&file, &data).unwrap();

        assert_eq!(load(&file), data);
    }

    // Checks that a missing file and a broken file both give the defaults.
    #[test]
    fn missing_or_broken_files_give_defaults() {
        let file = temp_file("broken");
        assert_eq!(load(&file), Data::default());

        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "{ not json").unwrap();
        assert_eq!(load(&file), Data::default());
    }

    // Checks that the file uses camelCase names, as the UI expects, and that
    // a file from an older version with missing fields still loads.
    #[test]
    fn uses_camel_case_and_tolerates_missing_fields() {
        let data: Data = serde_json::from_str(r#"{"settings":{"pixelGrid":false}}"#).unwrap();
        assert_eq!(data.settings.pixel_grid, Some(false));
        assert!(data.recent_projects.is_empty());
    }
}
