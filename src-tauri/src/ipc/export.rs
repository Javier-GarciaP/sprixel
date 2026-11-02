//! What the Export tab asks of the core: what an export would do, the preview,
//! the export itself and opening the output folder. The functions do not need
//! a window, so the tests call them directly.

use super::browser::CoreError;
use super::project::{self, ProjectState};
use crate::document::SpriteRef;
use crate::export::{
    self, DEFAULT_NAMING, ExportError, ExportFile, Format, Options, Scope, Summary, Written,
};
use crate::storage::DiskWriter;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScopeArg {
    Project,
    Folder { path: Vec<String> },
    Sprites { sprites: Vec<SpriteArg> },
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpriteArg {
    pub folder: Vec<String>,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatArg {
    Sheet,
    Png,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptionsArg {
    pub scale: u32,
    pub spacing: u32,
    /// `None` is the default pattern.
    pub naming: Option<String>,
}

impl ScopeArg {
    fn to_scope(&self) -> Scope {
        match self {
            ScopeArg::Project => Scope::Project,
            ScopeArg::Folder { path } => Scope::Folder(path.clone()),
            ScopeArg::Sprites { sprites } => Scope::Sprites(
                sprites
                    .iter()
                    .map(|s| SpriteRef { folder: s.folder.clone(), name: s.name.clone() })
                    .collect(),
            ),
        }
    }
}

impl FormatArg {
    fn to_format(&self) -> Format {
        match self {
            FormatArg::Sheet => Format::SheetJson,
            FormatArg::Png => Format::PngFiles,
        }
    }
}

impl OptionsArg {
    fn to_options(&self) -> Options {
        Options {
            scale: self.scale,
            spacing: self.spacing,
            naming: self.naming.clone().unwrap_or_else(|| DEFAULT_NAMING.to_string()),
        }
    }
}

impl From<ExportError> for CoreError {
    fn from(error: ExportError) -> Self {
        CoreError { code: error.code.to_string(), message: error.message }
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SummaryInfo {
    pub sprites: usize,
    pub frames: usize,
    pub files: Vec<String>,
    pub sheet_width: u32,
    pub sheet_height: u32,
}

impl From<Summary> for SummaryInfo {
    fn from(s: Summary) -> Self {
        SummaryInfo {
            sprites: s.sprites,
            frames: s.frames,
            files: s.files,
            sheet_width: s.width,
            sheet_height: s.height,
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct WrittenInfo {
    pub output: String,
    pub files: Vec<String>,
}

impl From<Written> for WrittenInfo {
    fn from(w: Written) -> Self {
        WrittenInfo { output: w.output, files: w.files }
    }
}

/// What the export would write, for the counts and the file list.
pub fn summary(
    state: &ProjectState,
    scope: &ScopeArg,
    format: &FormatArg,
    options: &OptionsArg,
) -> Result<SummaryInfo, CoreError> {
    let scope = scope.to_scope();
    project::read(state, |session| {
        let name = export::default_name(&session.project, &scope);
        export::summarize(&session.project, &scope, format.to_format(), &options.to_options(), &name)
    })
    .ok_or_else(CoreError::no_project)?
    .map(SummaryInfo::from)
    .map_err(CoreError::from)
}

/// The preview of the sheet: 8 bytes with the width and the height (little
/// endian, 4 bytes each), then the RGBA pixels.
pub fn preview(
    state: &ProjectState,
    scope: &ScopeArg,
    options: &OptionsArg,
) -> Result<Vec<u8>, CoreError> {
    let scope = scope.to_scope();
    let image = project::read(state, |session| {
        export::preview(&session.project, &scope, &options.to_options())
    })
    .ok_or_else(CoreError::no_project)??;
    let mut bytes = Vec::with_capacity(8 + image.rgba.len());
    bytes.extend_from_slice(&image.width.to_le_bytes());
    bytes.extend_from_slice(&image.height.to_le_bytes());
    bytes.extend_from_slice(&image.rgba);
    Ok(bytes)
}

/// Exports to the folder `output` of the project. The files are built while
/// the project is read, and written after, so the project is not held while
/// the disk works.
pub fn run(
    state: &ProjectState,
    scope: &ScopeArg,
    format: &FormatArg,
    options: &OptionsArg,
    output: &str,
) -> Result<WrittenInfo, CoreError> {
    let scope = scope.to_scope();
    let options = options.to_options();
    let (files, root): (Result<Vec<ExportFile>, ExportError>, PathBuf) =
        project::read(state, |session| {
            let project = &session.project;
            let files = match format.to_format() {
                Format::SheetJson => {
                    export::build(project, &scope, &options, &export::default_name(project, &scope))
                }
                Format::PngFiles => export::build_png_files(project, &scope, &options),
            };
            (files, session.root().to_path_buf())
        })
        .ok_or_else(CoreError::no_project)?;
    let written = export::write_files(&DiskWriter, &root, output, &files?)?;
    Ok(written.into())
}

/// Opens the folder `output` of the project in the file manager.
pub fn reveal(state: &ProjectState, output: &str) -> Result<(), CoreError> {
    let root = project::read(state, |session| session.root().to_path_buf())
        .ok_or_else(CoreError::no_project)?;
    let (folder, _) = export::resolve_output(&root, output)?;
    let opener = if cfg!(target_os = "windows") {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(opener)
        .arg(&folder)
        .spawn()
        .map(|_| ())
        .map_err(|e| CoreError { code: "io".to_string(), message: e.to_string() })
}

#[cfg(test)]
#[path = "export_tests.rs"]
mod tests;
