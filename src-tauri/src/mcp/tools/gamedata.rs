//! The tools for game engines: frame durations, anchor points and seamless
//! tiles.

use super::args;
use super::create::base_revision;
use super::read::{check_frame, read};
use super::write::{Plan, write};
use crate::document::{Anchor, DEFAULT_FRAME_MS, Operation};
use crate::mcp::{Ctx, Need, Output, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "set_frame_duration",
            description: concat!("Sets how long one frame lasts, in milliseconds (10 to 10000). Frames are numbered from 1. `get_sprite` lists the durations in `frame_durations`. `frame_durations` has `null` for a frame that follows the animation speed (inside an animation it lasts 1000 / fps) and a number for a frame with its own time. Send `milliseconds` null to give a frame back to the animation speed. Note: the file stores one number per frame and 100 ms means \"no time of its own\", so you cannot set an explicit 100 ms: it is read back as null.", write_note!()),
            schema: json!({ "type": "object", "required": ["name", "frame", "milliseconds"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "frame": { "type": "integer", "minimum": 1 },
                "milliseconds": { "type": ["integer", "null"], "minimum": 10, "maximum": 10000, "description": "Milliseconds, or null to follow the animation speed." },
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: set_frame_duration,
        },
        Tool {
            name: "set_anchor",
            description: concat!("Adds or moves an anchor: a named point of a sprite that a game uses, for example `feet`, `hand` or `muzzle`. `x` and `y` are a pixel inside the sprite (0,0 is the top left). Without `frame` the anchor is for the whole sprite; with `frame` it is only for that frame and wins over the sprite one of the same name. Setting the same name (and frame) again moves it. `get_sprite` lists the anchors.", write_note!()),
            schema: json!({ "type": "object", "required": ["sprite", "name", "x", "y"], "properties": {
                "sprite": { "type": "string", "description": "The sprite that gets the anchor." },
                "folder": { "type": "string" },
                "name": { "type": "string", "description": "The anchor name, for example `feet`. 1 to 64 characters." },
                "x": { "type": "integer", "minimum": 0 },
                "y": { "type": "integer", "minimum": 0 },
                "frame": { "type": "integer", "minimum": 1, "description": "Only this frame (from 1). Omit for the whole sprite." },
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: set_anchor,
        },
        Tool {
            name: "remove_anchor",
            description: concat!("Removes an anchor by name. Pass the same `frame` that you used to set it, or none for a sprite anchor. Fails with `not_found` if there is no such anchor.", write_note!()),
            schema: json!({ "type": "object", "required": ["sprite", "name"], "properties": {
                "sprite": { "type": "string" },
                "folder": { "type": "string" },
                "name": { "type": "string", "description": "The anchor name." },
                "frame": { "type": "integer", "minimum": 1 },
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: remove_anchor,
        },
        Tool {
            name: "check_tile",
            description: "Checks if a sprite frame repeats without visible seams (as a tile). It compares the first and the last column, and the first and the last row. Two pixels match when both are transparent or have the same palette key. Returns `seamless`, the real `mismatch_count` and the first 50 `mismatches` (each with `edge`, and the two places `a` and `b` with their keys). Only reads.",
            schema: json!({ "type": "object", "required": ["name"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "frame": { "type": "integer", "minimum": 1, "description": "The frame, from 1. Default 1." },
            }}),
            need: Need::Nothing,
            run: check_tile,
        },
        Tool {
            name: "render_tiled",
            description: "Returns a PNG image of one frame of a sprite repeated 3 times wide and 3 times high, on a gray checkerboard, so you can see the seams of a tile. The pixels are scaled up sharp. The image is at most 1024 pixels on a side. Only reads.",
            schema: json!({ "type": "object", "required": ["name"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "frame": { "type": "integer", "minimum": 1, "description": "The frame, from 1. Default 1." },
                "scale": { "type": "integer", "minimum": 1, "maximum": 64, "description": "Each pixel becomes scale x scale pixels. Default: the biggest of 8 or less that fits in 1024 pixels." },
            }}),
            need: Need::Nothing,
            run: render_tiled,
        },
    ]
}

/// The largest image, in pixels per side (the same cap as `render_preview`).
const MAX_SIDE: u32 = 1024;
const DEFAULT_SCALE: u32 = 8;
const TILES: u32 = 3;

/// The most mismatching pairs that `check_tile` lists.
const MAX_LISTED: usize = 50;

fn set_frame_duration(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::required_u32(input, "frame")?;
    // `null` means "follow the animation speed": the model stores DEFAULT_FRAME_MS for it.
    let ms = match input.get("milliseconds") {
        None => return Err(ToolError::new("bad_argument", "`milliseconds` must be given: a number from 10 to 10000, or null for the animation speed").with("argument", "milliseconds")),
        Some(Value::Null) => DEFAULT_FRAME_MS,
        Some(_) => args::required_u32(input, "milliseconds")?,
    };
    if !(10..=10000).contains(&ms) {
        return Err(ToolError::new("bad_argument", "`milliseconds` must be from 10 to 10000").with("argument", "milliseconds"));
    }
    write(ctx, input, |session| {
        let target = args::sprite(&session.project, input)?;
        let sprite = session.project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let ops = vec![Operation::SetFrameDuration { sprite: target.clone(), frame, duration_ms: ms }];
        let mut answer = json!({ "frame": frame, "milliseconds": if ms == DEFAULT_FRAME_MS { Value::Null } else { json!(ms) } });
        if ms == DEFAULT_FRAME_MS {
            answer["note"] = json!("this frame now follows the animation speed (1000 / fps); 100 ms is the same as having no time of its own");
        }
        Ok(Plan::new(ops, answer).about(&target))
    })
}

fn set_anchor(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.trim().to_string();
    let (x, y) = (args::required_u32(input, "x")?, args::required_u32(input, "y")?);
    let frame = args::optional_u32(input, "frame")?;
    write(ctx, input, |session| {
        let target = args::sprite_at(&session.project, input, "sprite")?;
        let sprite = session.project.find_sprite(&target).expect("just found");
        if name.is_empty() || name.chars().count() > 64 || name.chars().any(char::is_control) {
            return Err(ToolError::new("bad_argument", "`name` has 1 to 64 characters and no control characters").with("argument", "name"));
        }
        if x >= sprite.width || y >= sprite.height {
            return Err(ToolError::new(
                "bad_argument",
                format!("{x},{y} is outside the sprite; x goes from 0 to {} and y from 0 to {}", sprite.width - 1, sprite.height - 1),
            )
            .with("argument", "x"));
        }
        if let Some(frame) = frame {
            check_frame(sprite, frame)?;
        }
        let anchor = Anchor { name: name.clone(), x, y, frame };
        let ops = vec![Operation::SetAnchor { sprite: target.clone(), anchor, index: None }];
        Ok(Plan::new(ops, json!({ "anchor": name, "x": x, "y": y, "frame": frame })).about(&target))
    })
}

fn remove_anchor(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.to_string();
    let frame = args::optional_u32(input, "frame")?;
    write(ctx, input, |session| {
        let target = args::sprite_at(&session.project, input, "sprite")?;
        let ops = vec![Operation::RemoveAnchor { sprite: target.clone(), name: name.clone(), frame }];
        Ok(Plan::new(ops, json!({ "removed": name, "frame": frame })).about(&target))
    })
}

/// The keys of one frame with all layers stacked (the last layer on top).
pub(crate) fn flat_keys(sprite: &crate::document::Sprite, frame: usize) -> Vec<u8> {
    let mut flat = vec![crate::document::TRANSPARENT as u8; sprite.width as usize * sprite.height as usize];
    for layer in &sprite.layers {
        for (at, key) in layer.frames[frame].pixels.iter().enumerate() {
            if *key as char != crate::document::TRANSPARENT {
                flat[at] = *key;
            }
        }
    }
    flat
}

fn check_tile(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::optional_u32(input, "frame")?.unwrap_or(1);
    read(ctx, |session| {
        let target = args::sprite(&session.project, input)?;
        let sprite = session.project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let (w, h) = (sprite.width as usize, sprite.height as usize);
        let flat = flat_keys(sprite, frame as usize - 1);
        let at = |x: usize, y: usize| json!({ "x": x, "y": y, "key": (flat[y * w + x] as char).to_string() });
        let mut found = Vec::new();
        for y in 0..h {
            if flat[y * w] != flat[y * w + w - 1] {
                found.push(json!({ "edge": "left_right", "a": at(0, y), "b": at(w - 1, y) }));
            }
        }
        for x in 0..w {
            if flat[x] != flat[(h - 1) * w + x] {
                found.push(json!({ "edge": "top_bottom", "a": at(x, 0), "b": at(x, h - 1) }));
            }
        }
        let count = found.len();
        found.truncate(MAX_LISTED);
        Ok(Output::Json(json!({
            "name": sprite.name, "folder": args::join(&target.folder), "frame": frame,
            "width": w, "height": h, "revision": session.project.revision,
            "seamless": count == 0, "mismatch_count": count,
            "truncated": count > MAX_LISTED, "mismatches": found,
        })))
    })
}

fn render_tiled(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::optional_u32(input, "frame")?.unwrap_or(1);
    let wanted = args::optional_u32(input, "scale")?;
    if wanted.is_some_and(|scale| !(1..=64).contains(&scale)) {
        return Err(ToolError::new("bad_argument", "`scale` must be from 1 to 64").with("argument", "scale"));
    }
    let (image, info) = read(ctx, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let side = sprite.width.max(sprite.height) * TILES;
        // The biggest scale that keeps the image inside the cap.
        let largest = MAX_SIDE / side;
        let scale = wanted.unwrap_or_else(|| DEFAULT_SCALE.min(largest).max(1));
        if scale == 0 || side * scale > MAX_SIDE {
            return Err(ToolError::new(
                "too_big",
                format!("the image would be {} pixels on a side; the maximum is {MAX_SIDE}. Use scale {largest} or less", side * scale),
            )
            .with("limit", MAX_SIDE)
            .with("max_scale", largest));
        }
        let palette = project.palette(&sprite.palette).ok_or_else(|| {
            ToolError::new("palette_not_found", format!("the palette {:?} of this sprite does not exist", sprite.palette))
        })?;
        let tile = crate::render::render_frame(sprite, palette, frame as usize - 1, scale);
        let image = tile_over_checkerboard(&tile);
        let info = json!({
            "name": sprite.name, "folder": args::join(&target.folder), "frame": frame, "scale": scale,
            "tiles": TILES, "width": image.width, "height": image.height, "revision": project.revision,
        });
        Ok((image, info))
    })?;
    let png = crate::export::encode_png(&image).map_err(|e| ToolError::new(e.code, e.message))?;
    Ok(Output::Image { png, info })
}

/// The tile repeated `TILES` x `TILES` times, over a gray checkerboard.
fn tile_over_checkerboard(tile: &crate::render::Image) -> crate::render::Image {
    const SQUARE: u32 = 8;
    let (w, h) = (tile.width, tile.height);
    let (width, height) = (w * TILES, h * TILES);
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        for x in 0..width {
            let from = ((y % h) * w + x % w) as usize * 4;
            let pixel = &tile.rgba[from..from + 4];
            if pixel[3] == 255 {
                rgba.extend_from_slice(pixel);
            } else {
                let light = (x / SQUARE + y / SQUARE) % 2 == 0;
                let gray = if light { 204 } else { 153 };
                rgba.extend_from_slice(&[gray, gray, gray, 255]);
            }
        }
    }
    crate::render::Image { width, height, rgba }
}
