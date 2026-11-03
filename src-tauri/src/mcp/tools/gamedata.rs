//! The tools for game engines: frame durations, anchor points and seamless
//! tiles.

use super::args;
use super::create::base_revision;
use super::read::{check_frame, read};
use super::write::{Plan, write};
use crate::document::{Anchor, Operation};
use crate::mcp::{Ctx, Need, Output, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "set_frame_duration",
            description: concat!("Sets how long one frame lasts, in milliseconds (10 to 10000). Frames are numbered from 1. `get_sprite` lists the durations in `frame_durations`. A frame that keeps exactly 100 ms is treated as having no duration of its own: inside an animation it lasts 1000 / fps. Set any other value to fix the time of the frame.", write_note!()),
            schema: json!({ "type": "object", "required": ["name", "frame", "milliseconds"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "frame": { "type": "integer", "minimum": 1 },
                "milliseconds": { "type": "integer", "minimum": 10, "maximum": 10000 },
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
    ]
}

/// The most mismatching pairs that `check_tile` lists.
const MAX_LISTED: usize = 50;

fn set_frame_duration(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::required_u32(input, "frame")?;
    let ms = args::required_u32(input, "milliseconds")?;
    if !(10..=10000).contains(&ms) {
        return Err(ToolError::new("bad_argument", "`milliseconds` must be from 10 to 10000").with("argument", "milliseconds"));
    }
    write(ctx, input, |session| {
        let target = args::sprite(&session.project, input)?;
        let sprite = session.project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let ops = vec![Operation::SetFrameDuration { sprite: target.clone(), frame, duration_ms: ms }];
        Ok(Plan::new(ops, json!({ "frame": frame, "milliseconds": ms })).about(&target))
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
fn flat_keys(sprite: &crate::document::Sprite, frame: usize) -> Vec<u8> {
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
