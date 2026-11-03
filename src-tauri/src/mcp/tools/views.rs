//! The tools that show many frames in one image: `render_frames` (a grid) and
//! `render_onion` (frames over each other). Both only read.

use super::args;
use super::read::{MAX_PREVIEW_SIDE, background_arg, background_of, check_frame, read};
use crate::mcp::{Ctx, Need, Output, Tool, ToolError, ToolResult};
use crate::render::{checker_cell, flatten, onion, render_frame, render_sheet, sheet_size};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
        name: "render_frames",
        description: "Returns ONE PNG image with several frames of a sprite side by side in a grid, so you can judge an animation at a glance. Frames go in order, left to right and then down. With `labels` each frame has its number above it. The image is at most 1024 pixels on a side.",
        schema: json!({ "type": "object", "required": ["name"], "properties": {
            "name": { "type": "string" },
            "folder": { "type": "string" },
            "from": { "type": "integer", "minimum": 1, "description": "The first frame, from 1. Default 1." },
            "to": { "type": "integer", "minimum": 1, "description": "The last frame. Default: the last one." },
            "columns": { "type": "integer", "minimum": 1, "description": "Frames per row. Default: about a square." },
            "scale": { "type": "integer", "minimum": 1, "maximum": 64, "description": "Each pixel becomes scale x scale pixels. Default: the largest from 1 to 8 that fits." },
            "labels": { "type": "boolean", "description": "Draw the frame number above each frame. Default false." },
            "background": background_arg()
        }}),
        need: Need::Nothing,
        run: render_frames,
    },
    Tool {
        name: "render_onion",
        description: "Returns ONE PNG image with several frames drawn over each other (an onion skin), so you can see how pixels move between frames. The last frame in `frames` is drawn normally; earlier frames are tinted red and see-through (older ones fainter). The image is at most 1024 pixels on a side.",
        schema: json!({ "type": "object", "required": ["name"], "properties": {
            "name": { "type": "string" },
            "folder": { "type": "string" },
            "frames": { "type": "array", "items": { "type": "integer", "minimum": 1 }, "description": "The frames to overlay, from 1, oldest first. Default: all frames." },
            "scale": { "type": "integer", "minimum": 1, "maximum": 64, "description": "Each pixel becomes scale x scale pixels. Default: the largest from 1 to 8 that fits." },
            "background": background_arg()
        }}),
        need: Need::Nothing,
        run: render_onion,
    }]
}

fn bad_scale() -> ToolError {
    ToolError::new("bad_argument", "`scale` must be from 1 to 64").with("argument", "scale")
}

fn render_frames(ctx: &Ctx, input: &Value) -> ToolResult {
    let from = args::optional_u32(input, "from")?;
    let to = args::optional_u32(input, "to")?;
    let columns = args::optional_u32(input, "columns")?;
    let scale = args::optional_u32(input, "scale")?;
    let labels = args::optional_bool(input, "labels")?.unwrap_or(false);
    let background = background_of(input)?;
    if columns == Some(0) {
        return Err(ToolError::new("bad_argument", "`columns` must be 1 or more").with("argument", "columns"));
    }
    if scale.is_some_and(|s| !(1..=64).contains(&s)) {
        return Err(bad_scale());
    }
    let (image, info) = read(ctx, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        let count = sprite.frame_count() as u32;
        let (from, to) = (from.unwrap_or(1), to.unwrap_or(count));
        check_frame(sprite, from)?;
        check_frame(sprite, to)?;
        if from > to {
            return Err(ToolError::new("bad_argument", format!("`from` ({from}) is after `to` ({to})")).with("argument", "from"));
        }
        let shown = to - from + 1;
        let columns = columns.unwrap_or_else(|| (shown as f64).sqrt().ceil() as u32).min(shown);
        let fits = |s: u32| {
            let (w, h) = sheet_size(sprite.width, sprite.height, shown, columns, s, labels);
            w <= MAX_PREVIEW_SIDE && h <= MAX_PREVIEW_SIDE
        };
        let largest = (1..=64).rev().find(|s| fits(*s)).unwrap_or(0);
        let scale = match scale {
            Some(s) => s,
            None if largest == 0 => 1,
            None => largest.min(8),
        };
        if !fits(scale) {
            let (w, h) = sheet_size(sprite.width, sprite.height, shown, columns, scale, labels);
            return Err(ToolError::new(
                "too_big",
                format!("the image would be {w} x {h} pixels; the maximum is {MAX_PREVIEW_SIDE} on a side. {}", if largest == 0 {
                    "Even scale 1 is too big: ask for fewer frames or more columns".to_string()
                } else {
                    format!("Use scale {largest} or less")
                }),
            )
            .with("limit", MAX_PREVIEW_SIDE)
            .with("max_scale", largest));
        }
        let palette = project.palette(&sprite.palette).ok_or_else(|| {
            ToolError::new("palette_not_found", format!("the palette {:?} of this sprite does not exist", sprite.palette))
        })?;
        let frames: Vec<_> = (from..=to).map(|f| render_frame(sprite, palette, f as usize - 1, scale)).collect();
        let image = render_sheet(&frames, columns, scale, labels, background);
        let info = json!({
            "kind": "frames", "name": sprite.name, "folder": args::join(&target.folder), "from": from, "to": to,
            "columns": columns, "scale": scale, "labels": labels, "width": image.width, "height": image.height,
            "revision": project.revision,
        });
        Ok((image, info))
    })?;
    let png = crate::export::encode_png(&image).map_err(|e| ToolError::new(e.code, e.message))?;
    Ok(Output::Image { png, info })
}

fn render_onion(ctx: &Ctx, input: &Value) -> ToolResult {
    let list = match input.get("frames") {
        None | Some(Value::Null) => None,
        Some(Value::Array(items)) if !items.is_empty() => Some(
            items
                .iter()
                .map(|n| n.as_u64().and_then(|n| u32::try_from(n).ok()))
                .collect::<Option<Vec<u32>>>()
                .ok_or_else(|| ToolError::new("bad_argument", "`frames` must be a list of whole numbers").with("argument", "frames"))?,
        ),
        Some(_) => {
            return Err(ToolError::new("bad_argument", "`frames` must be a non-empty list of frame numbers").with("argument", "frames"));
        }
    };
    let scale = args::optional_u32(input, "scale")?;
    let background = background_of(input)?;
    if scale.is_some_and(|s| !(1..=64).contains(&s)) {
        return Err(bad_scale());
    }
    let (image, info) = read(ctx, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        let frames = list.unwrap_or_else(|| (1..=sprite.frame_count() as u32).collect());
        for frame in &frames {
            check_frame(sprite, *frame)?;
        }
        let side = sprite.width.max(sprite.height);
        let largest = (MAX_PREVIEW_SIDE / side).max(1);
        let scale = scale.unwrap_or(largest.min(8));
        if side * scale > MAX_PREVIEW_SIDE {
            return Err(ToolError::new(
                "too_big",
                format!("the image would be {} pixels on a side; the maximum is {MAX_PREVIEW_SIDE}. Use scale {largest} or less", side * scale),
            )
            .with("limit", MAX_PREVIEW_SIDE)
            .with("max_scale", largest));
        }
        let palette = project.palette(&sprite.palette).ok_or_else(|| {
            ToolError::new("palette_not_found", format!("the palette {:?} of this sprite does not exist", sprite.palette))
        })?;
        let images: Vec<_> = frames.iter().map(|f| render_frame(sprite, palette, *f as usize - 1, scale)).collect();
        let image = flatten(&onion(&images), background, checker_cell(scale));
        let info = json!({
            "kind": "onion", "name": sprite.name, "folder": args::join(&target.folder), "frames": frames,
            "scale": scale, "width": image.width, "height": image.height, "revision": project.revision,
        });
        Ok((image, info))
    })?;
    let png = crate::export::encode_png(&image).map_err(|e| ToolError::new(e.code, e.message))?;
    Ok(Output::Image { png, info })
}
