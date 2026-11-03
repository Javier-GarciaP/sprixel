use super::plan::{self, ExportFile, Item, Options, Scope};
use super::ExportError;
use crate::document::{Anchor, Project, Sprite};
use crate::render::{FRAMES_PER_ROW, Image, Placement, pack_rows, render_frame};
use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use serde::Serialize;

/// The version of the JSON layout. It grows when the layout changes in a way
/// that can break a reader.
pub const JSON_VERSION: u32 = 1;

#[derive(Serialize)]
struct Json {
    meta: Meta,
    frames: Vec<JsonFrame>,
    animations: Vec<JsonAnimation>,
}

#[derive(Serialize)]
struct Meta {
    app: &'static str,
    version: u32,
    image: String,
    size: Size,
    scale: u32,
}

#[derive(Serialize)]
struct Size {
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct Pivot {
    x: u32,
    y: u32,
}

#[derive(Serialize)]
struct JsonFrame {
    name: String,
    sprite: String,
    frame: Rect,
    /// Milliseconds.
    duration: u32,
    pivot: Pivot,
    /// The anchors of this frame, by name; coordinates in pixels of the
    /// sheet from the top left corner of the frame.
    anchors: Vec<JsonAnchor>,
}

#[derive(Serialize)]
struct JsonAnchor {
    name: String,
    x: u32,
    y: u32,
}

#[derive(Serialize)]
struct JsonAnimation {
    name: String,
    sprite: String,
    /// Positions in `frames`, from 0, both ends included.
    from: usize,
    to: usize,
    mode: &'static str,
    fps: u32,
}

/// The anchors a frame has: those of the whole sprite, where a frame anchor
/// of the same name (ignoring case) takes their place. Sorted by name.
fn frame_anchors(sprite: &Sprite, index: usize, scale: u32) -> Vec<JsonAnchor> {
    let frame = index as u32 + 1;
    let mut found: Vec<&Anchor> = Vec::new();
    for anchor in sprite.anchors.iter().filter(|a| a.frame == Some(frame)) {
        found.push(anchor);
    }
    for anchor in sprite.anchors.iter().filter(|a| a.frame.is_none()) {
        if !found.iter().any(|f| f.name.to_lowercase() == anchor.name.to_lowercase()) {
            found.push(anchor);
        }
    }
    found.sort_by_key(|a| (a.name.to_lowercase(), a.name.clone()));
    found
        .into_iter()
        .map(|a| JsonAnchor { name: a.name.clone(), x: a.x * scale, y: a.y * scale })
        .collect()
}

/// Encodes RGBA pixels as a PNG.
pub fn encode_png(image: &Image) -> Result<Vec<u8>, ExportError> {
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(&image.rgba, image.width, image.height, ExtendedColorType::Rgba8)
        .map_err(|e| ExportError::new("encode", e.to_string()))?;
    Ok(bytes)
}

/// Builds the sheet: the rendered frames at their places, and the layout.
pub fn render_sheet(items: &[Item], options: &Options) -> (Image, Vec<Placement>) {
    let images: Vec<Image> = items
        .iter()
        .map(|i| render_frame(i.sprite, i.palette, i.index, options.scale))
        .collect();
    let sizes: Vec<(u32, u32)> = images.iter().map(|i| (i.width, i.height)).collect();
    let layout = pack_rows(&sizes, options.spacing * options.scale, FRAMES_PER_ROW);
    let mut sheet = Image {
        width: layout.width,
        height: layout.height,
        rgba: vec![0; layout.width as usize * layout.height as usize * 4],
    };
    for (image, at) in images.iter().zip(&layout.placements) {
        for row in 0..image.height as usize {
            let from = row * image.width as usize * 4;
            let to = ((at.y as usize + row) * sheet.width as usize + at.x as usize) * 4;
            let len = image.width as usize * 4;
            sheet.rgba[to..to + len].copy_from_slice(&image.rgba[from..from + len]);
        }
    }
    (sheet, layout.placements)
}

/// Builds the files of the "Sheet + JSON" format: `<name>.png` and
/// `<name>.json`. The same project and options give the same bytes.
pub fn build(
    project: &Project,
    scope: &Scope,
    options: &Options,
    name: &str,
) -> Result<Vec<ExportFile>, ExportError> {
    let items = plan::items(project, scope, options)?;
    if items.is_empty() {
        return Err(ExportError::new("nothing_to_export", "the scope has no sprites"));
    }
    let (sheet, placements) = render_sheet(&items, options);
    let png = encode_png(&sheet)?;

    let frames = items
        .iter()
        .zip(&placements)
        .map(|(item, at)| JsonFrame {
            name: item.name.clone(),
            sprite: item.sprite.name.clone(),
            frame: Rect { x: at.x, y: at.y, w: at.w, h: at.h },
            duration: item.duration_ms,
            pivot: Pivot { x: 0, y: 0 },
            anchors: frame_anchors(item.sprite, item.index, options.scale),
        })
        .collect();
    let mut animations = Vec::new();
    let mut first_of_sprite = 0;
    for sprite in plan::sprites_of(project, scope) {
        for tag in &sprite.tags {
            animations.push(JsonAnimation {
                name: tag.name.clone(),
                sprite: sprite.name.clone(),
                from: first_of_sprite + tag.from as usize - 1,
                to: first_of_sprite + tag.to as usize - 1,
                mode: tag.mode.as_str(),
                fps: tag.fps,
            });
        }
        first_of_sprite += sprite.frame_count();
    }
    let json = Json {
        meta: Meta {
            app: "Sprixel",
            version: JSON_VERSION,
            image: format!("{name}.png"),
            size: Size { w: sheet.width, h: sheet.height },
            scale: options.scale,
        },
        frames,
        animations,
    };
    let mut text = serde_json::to_string_pretty(&json)
        .map_err(|e| ExportError::new("encode", e.to_string()))?;
    text.push('\n');
    Ok(vec![
        ExportFile { name: format!("{name}.png"), bytes: png },
        ExportFile { name: format!("{name}.json"), bytes: text.into_bytes() },
    ])
}
