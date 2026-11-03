//! The tools that finish and check a sprite for an agent: `auto_outline`,
//! `auto_shade` (they need "Edit pixels") and `analyze_sprite` (read-only). The
//! rules are in `crate::analysis`; here the arguments are read and checked and
//! the result is written like any other pixel write.

use super::args;
use super::create::base_revision;
use super::draw::{key_of, preview_arg, valid_keys};
use super::read::check_frame;
use super::write::{Plan, write};
use crate::analysis::{self, Connectivity, OutlineOptions, View};
use crate::document::{Operation, PixelChange, Sprite, SpriteRef, TRANSPARENT};
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "auto_outline",
        description: concat!("Draws a one pixel outline with one palette key around the shape of a frame, so you do not draw it by hand. By default it draws on every transparent pixel that touches the shape (`connectivity` 8 also counts corners, 4 only the sides), holes included. `outside_only` true skips holes. `inner` true recolors the edge pixels of the shape instead of drawing around it. Outline pixels that fall outside the sprite are skipped and counted in `clipped`. Call it once, when the shape is done. Returns `pixels_changed` and an image of the frame.", write_note!()),
        schema: json!({ "type": "object", "required": ["name", "key"], "properties": {
            "name": { "type": "string" },
            "folder": { "type": "string" },
            "frame": { "type": "integer", "minimum": 1, "description": "Default 1." },
            "key": { "type": "string", "description": "The palette key of the outline (not `.`). Read the keys with get_palette." },
            "connectivity": { "type": "integer", "enum": [4, 8], "description": "Default 8." },
            "outside_only": { "type": "boolean", "description": "Do not outline holes. Default false." },
            "inner": { "type": "boolean", "description": "Outline inside the shape. Default false." },
            "preview": preview_arg(),
            "base_revision": base_revision(),
        }}),
        need: Need::Edit,
        run: auto_outline,
    }]
}

/// The pixels of a frame (from 1) of the first layer.
fn view_of(sprite: &Sprite, frame: u32) -> View<'_> {
    View::new(&sprite.layers[0].frames[frame as usize - 1].pixels, sprite.width as usize, sprite.height as usize)
}

/// The operations that set `changes` (none, if the list is empty).
fn set_ops(target: &SpriteRef, frame: u32, changes: Vec<PixelChange>) -> Vec<Operation> {
    if changes.is_empty() {
        Vec::new()
    } else {
        vec![Operation::SetPixels { sprite: target.clone(), layer: 0, frame, pixels: changes }]
    }
}

/// An outline or shading key must be a real color, not transparent.
fn opaque_key(text: &str, valid: &[String], argument: &str) -> Result<char, ToolError> {
    let key = key_of(text, valid)?;
    if key == TRANSPARENT {
        return Err(ToolError::new("bad_argument", format!("`{argument}` must be a color key, not `.`")).with("argument", argument));
    }
    Ok(key)
}

fn auto_outline(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::optional_u32(input, "frame")?.unwrap_or(1);
    let key_text = args::required_str(input, "key")?.to_string();
    let connectivity = match args::optional_u32(input, "connectivity")? {
        None | Some(8) => Connectivity::Eight,
        Some(4) => Connectivity::Four,
        Some(_) => return Err(ToolError::new("bad_argument", "`connectivity` must be 4 or 8").with("argument", "connectivity")),
    };
    let outside_only = args::optional_bool(input, "outside_only")?.unwrap_or(false);
    let inner = args::optional_bool(input, "inner")?.unwrap_or(false);
    write(ctx, input, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let key = opaque_key(&key_text, &valid_keys(project, sprite), "key")?;
        let outline = analysis::auto_outline(view_of(sprite, frame), &OutlineOptions { key, connectivity, outside_only, inner });
        let count = outline.changes.len();
        let ops = set_ops(&target, frame, outline.changes);
        Ok(Plan::new(ops, json!({ "pixels_changed": count, "clipped": outline.clipped })).preview().busy(&target, frame))
    })
}
