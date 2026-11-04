//! The JSON in the array form of the Aseprite sprite sheet. Godot and Unity
//! importers for Aseprite read it.

use super::plan::Item;
use super::sheet::JSON_VERSION;
use super::ExportError;
use crate::document::{Sprite, TagMode};
use crate::render::Placement;
use serde::Serialize;

#[derive(Serialize)]
struct Sheet {
    frames: Vec<Frame>,
    meta: Meta,
}

#[derive(Serialize)]
struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Serialize)]
struct Size {
    w: u32,
    h: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Frame {
    filename: String,
    frame: Rect,
    rotated: bool,
    trimmed: bool,
    sprite_source_size: Rect,
    source_size: Size,
    duration: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Meta {
    app: &'static str,
    version: String,
    image: String,
    format: &'static str,
    size: Size,
    /// Text, as Aseprite writes it.
    scale: String,
    frame_tags: Vec<Tag>,
}

#[derive(Serialize)]
struct Tag {
    name: String,
    from: usize,
    to: usize,
    direction: &'static str,
}

/// Builds the text of the JSON. `sprites` are the sprites of the export in
/// the order of the frames.
pub(super) fn json(
    items: &[Item],
    placements: &[Placement],
    sprites: &[&Sprite],
    image: String,
    sheet_size: (u32, u32),
    scale: u32,
) -> Result<String, ExportError> {
    let frames = items
        .iter()
        .zip(placements)
        .map(|(item, at)| {
            let (w, h) = (item.sprite.width * scale, item.sprite.height * scale);
            Frame {
                filename: item.name.clone(),
                frame: Rect { x: at.x, y: at.y, w: at.w, h: at.h },
                rotated: false,
                trimmed: false,
                sprite_source_size: Rect { x: 0, y: 0, w, h },
                source_size: Size { w, h },
                duration: item.duration_ms,
            }
        })
        .collect();
    // With several sprites two of them can have a tag with the same name, so
    // the name of the sprite goes first.
    let several = sprites.len() > 1;
    let mut frame_tags = Vec::new();
    let mut first_of_sprite = 0;
    for sprite in sprites {
        for tag in &sprite.tags {
            frame_tags.push(Tag {
                name: if several { format!("{}_{}", sprite.name, tag.name) } else { tag.name.clone() },
                from: first_of_sprite + tag.from as usize - 1,
                to: first_of_sprite + tag.to as usize - 1,
                direction: match tag.mode {
                    TagMode::PingPong => "pingpong",
                    TagMode::Loop | TagMode::Once => "forward",
                },
            });
        }
        first_of_sprite += sprite.frame_count();
    }
    let sheet = Sheet {
        frames,
        meta: Meta {
            app: "Sprixel",
            version: JSON_VERSION.to_string(),
            image,
            format: "RGBA8888",
            size: Size { w: sheet_size.0, h: sheet_size.1 },
            scale: scale.to_string(),
            frame_tags,
        },
    };
    let mut text = serde_json::to_string_pretty(&sheet).map_err(|e| ExportError::new("encode", e.to_string()))?;
    text.push('\n');
    Ok(text)
}
