use super::plan::{self, ExportFile, Options, Scope};
use super::sheet::encode_png;
use super::ExportError;
use crate::document::Project;
use crate::render::render_frame;

/// Builds the files of the "PNG files" format: one `<frame name>.png` for
/// every frame.
pub fn build_png_files(
    project: &Project,
    scope: &Scope,
    options: &Options,
) -> Result<Vec<ExportFile>, ExportError> {
    let items = plan::items(project, scope, options)?;
    if items.is_empty() {
        return Err(ExportError::new("nothing_to_export", "the scope has no sprites"));
    }
    items
        .iter()
        .map(|item| {
            let image = render_frame(item.sprite, item.palette, item.index, options.scale);
            Ok(ExportFile { name: format!("{}.png", item.name), bytes: encode_png(&image)? })
        })
        .collect()
}
