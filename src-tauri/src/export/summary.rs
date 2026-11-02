//! What an export would do, without writing anything: the counts and the file
//! names for the Export tab, the preview image and the default file name.

use super::plan::{self, Format, Options, Scope};
use super::sheet::render_sheet;
use super::ExportError;
use crate::document::Project;
use crate::render::{FRAMES_PER_ROW, Image, pack_rows};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub sprites: usize,
    pub frames: usize,
    /// The names of the files the export writes.
    pub files: Vec<String>,
    /// The size of the sheet. 0 by 0 when there is nothing to export.
    pub width: u32,
    pub height: u32,
}

/// Counts what the export of `scope` would write. An empty scope is not an
/// error here: it gives zeros, and the tab shows the empty state.
pub fn summarize(
    project: &Project,
    scope: &Scope,
    format: Format,
    options: &Options,
    name: &str,
) -> Result<Summary, ExportError> {
    let items = plan::items(project, scope, options)?;
    let sizes: Vec<(u32, u32)> = items
        .iter()
        .map(|i| (i.sprite.width * options.scale, i.sprite.height * options.scale))
        .collect();
    let layout = pack_rows(&sizes, options.spacing * options.scale, FRAMES_PER_ROW);
    let files = match (format, items.is_empty()) {
        (_, true) => Vec::new(),
        (Format::SheetJson, false) => vec![format!("{name}.png"), format!("{name}.json")],
        (Format::PngFiles, false) => items.iter().map(|i| format!("{}.png", i.name)).collect(),
    };
    Ok(Summary {
        sprites: plan::sprites_of(project, scope).len(),
        frames: items.len(),
        files,
        width: layout.width,
        height: layout.height,
    })
}

/// The sheet as the "Sheet + JSON" format would draw it, for the preview.
/// Always at 1x, so the preview is cheap; only the spacing counts.
pub fn preview(project: &Project, scope: &Scope, options: &Options) -> Result<Image, ExportError> {
    let options = Options { scale: 1, ..options.clone() };
    let items = plan::items(project, scope, &options)?;
    if items.is_empty() {
        return Err(ExportError::new("nothing_to_export", "the scope has no sprites"));
    }
    Ok(render_sheet(&items, &options).0)
}

/// The base name of the files: the name of the folder, of the only sprite, or
/// of the project, in lower case with `-` instead of spaces and only letters,
/// digits, `-` and `_`.
pub fn default_name(project: &Project, scope: &Scope) -> String {
    let source = match scope {
        Scope::Folder(path) if !path.is_empty() => path.last().cloned().unwrap_or_default(),
        Scope::Sprites(sprites) if sprites.len() == 1 => sprites[0].name.clone(),
        _ => project.name.clone(),
    };
    let name: String = source
        .to_lowercase()
        .chars()
        .map(|c| if c.is_whitespace() { '-' } else { c })
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if name.is_empty() { "sprites".to_string() } else { name }
}
