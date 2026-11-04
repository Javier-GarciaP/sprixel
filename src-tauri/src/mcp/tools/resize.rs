//! The tool that changes the canvas size of a sprite.

use super::args;
use super::create::base_revision;
use super::draw::preview_arg;
use super::write::{Plan, write};
use crate::document::{Operation, ResizeAnchor, MAX_SIZE, move_anchors};
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "resize_sprite",
        description: concat!("Changes the canvas size of a sprite (1 to 512 per side). Every frame of every layer is cropped or extended with transparent pixels. `anchor` says where the old drawing stays: top-left, top, top-right, left, center (default), right, bottom-left, bottom or bottom-right; with `top-left` the new space is added to the right and bottom. The sprite's anchors (`feet`...) move with the drawing; those that end outside the new canvas are removed and listed in `dropped_anchors` and `warnings`. Tags, durations and the palette do not change. Cropping loses the pixels outside, but the person can undo it. The size is meant to stay: use this when the sprite really needs a bigger or smaller canvas.", write_note!()),
        schema: json!({ "type": "object", "required": ["name", "width", "height"], "properties": {
            "name": { "type": "string", "description": "The sprite." },
            "folder": { "type": "string" },
            "width": { "type": "integer", "minimum": 1, "maximum": MAX_SIZE },
            "height": { "type": "integer", "minimum": 1, "maximum": MAX_SIZE },
            "anchor": { "type": "string", "enum": ["top-left", "top", "top-right", "left", "center", "right", "bottom-left", "bottom", "bottom-right"], "description": "Where the old drawing stays. Default `center`." },
            "preview": preview_arg(),
            "background": super::read::background_arg(),
            "base_revision": base_revision(),
        }}),
        need: Need::Edit,
        run: resize_sprite,
    }]
}

fn resize_sprite(ctx: &Ctx, input: &Value) -> ToolResult {
    let (width, height) = (args::required_u32(input, "width")?, args::required_u32(input, "height")?);
    for (name, value) in [("width", width), ("height", height)] {
        if !(1..=MAX_SIZE).contains(&value) {
            return Err(ToolError::new("bad_argument", format!("`{name}` must be from 1 to {MAX_SIZE}")).with("argument", name));
        }
    }
    let anchor_name = args::optional_str(input, "anchor")?.unwrap_or("center");
    let anchor = ResizeAnchor::parse(anchor_name).ok_or_else(|| {
        ToolError::new(
            "bad_argument",
            format!("`anchor` is {anchor_name:?}; use top-left, top, top-right, left, center, right, bottom-left, bottom or bottom-right"),
        )
        .with("argument", "anchor")
    })?;
    write(ctx, input, |session| {
        let target = args::sprite(&session.project, input)?;
        let sprite = session.project.find_sprite(&target).expect("just found");
        if (sprite.width, sprite.height) == (width, height) {
            return Err(ToolError::new("bad_argument", format!("the sprite already is {width}x{height}")).with("argument", "width"));
        }
        let offset = anchor.offset((sprite.width, sprite.height), (width, height));
        let (_, dropped) = move_anchors(&sprite.anchors, offset, (width, height));
        let dropped_json: Vec<Value> = dropped.iter().map(|a| json!({ "name": a.name, "x": a.x, "y": a.y, "frame": a.frame })).collect();
        let warnings: Vec<String> = dropped
            .iter()
            .map(|a| format!("the anchor {} (at {},{}) ended outside the new canvas and was removed", a.name, a.x, a.y))
            .collect();
        let ops = vec![Operation::ResizeSprite { sprite: target.clone(), width, height, anchor }];
        let extra = json!({
            "width": width, "height": height, "anchor": anchor.as_str(),
            "old_width": sprite.width, "old_height": sprite.height,
            "dropped_anchors": dropped_json, "warnings": warnings,
        });
        Ok(Plan::new(ops, extra).preview().busy(&target, 1))
    })
}
