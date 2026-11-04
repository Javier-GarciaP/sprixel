//! The tools that organize animations: `create_animation`,
//! `duplicate_animation`, `rename_animation`, `delete_animation` and
//! `move_frame`. An animation is a tag over a range of frames; these tools
//! make it from the base frame without sending pixels. The rules are the ones
//! of the editor (`document::animation`), so the codes are the same.

use super::args;
use super::create::base_revision;
use super::draw::preview_arg;
use super::write::{Plan, write};
use crate::document::{self, TagMode};
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "create_animation",
        description: concat!(
            "Makes a new animation from the base frame in ONE call, without sending pixels: it copies frame `base_frame` (default 1) `frames` times at the end of the sprite (or at `at`), and tags the copies with `name`, `fps` and `mode`. Then change only what moves in those frames (`transform_region`, `apply_grid` with a region). `sprite` is the sprite, `name` is the name of the new animation (unique in the sprite, ignoring case: `name_exists`). Animations never share frames: `at` cannot be inside another animation (`animation_overlap`). Returns the frames of the animation and an image of its first frame.",
            write_note!()
        ),
        schema: json!({ "type": "object", "required": ["sprite", "name", "frames"], "properties": {
            "sprite": { "type": "string", "description": "The sprite that gets the animation." },
            "folder": { "type": "string" },
            "name": { "type": "string", "description": "The animation name, for example `walk`." },
            "frames": { "type": "integer", "minimum": 1, "maximum": 128, "description": "How many frames the animation has." },
            "fps": { "type": "integer", "minimum": 1, "maximum": 60, "description": "Default 8." },
            "mode": { "type": "string", "enum": ["loop", "once", "pingpong"], "description": "Default `loop`." },
            "base_frame": { "type": "integer", "minimum": 1, "description": "The frame to copy. Default 1." },
            "at": { "type": "integer", "minimum": 1, "description": "Where the first new frame goes (1 to frames + 1). Default: the end." },
            "preview": preview_arg(),
            "base_revision": base_revision(),
        }}),
        need: Need::Edit,
        run: create_animation,
    }]
}

fn mode_of(input: &Value) -> Result<TagMode, ToolError> {
    let text = args::optional_str(input, "mode")?.unwrap_or("loop");
    TagMode::parse(text).ok_or_else(|| {
        ToolError::new("bad_mode", format!("{text:?} is not a repeat mode; use loop, once or pingpong")).with("argument", "mode")
    })
}

fn create_animation(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.to_string();
    let frames = args::required_u32(input, "frames")? as usize;
    let fps = args::optional_u32(input, "fps")?.unwrap_or(8);
    let mode = mode_of(input)?;
    let base = args::optional_u32(input, "base_frame")?.unwrap_or(1);
    let at = args::optional_u32(input, "at")?;
    write(ctx, input, |session| {
        let target = args::sprite_at(&session.project, input, "sprite")?;
        let ops = document::create_animation(&session.project, &target, &name, frames, fps, mode, base, at)?;
        let count = session.project.find_sprite(&target).expect("just found").frame_count();
        let from = at.unwrap_or(count as u32 + 1);
        let extra = json!({
            "animation": name, "from": from, "to": from + frames as u32 - 1, "fps": fps,
            "mode": mode.as_str(), "frames": count + frames,
        });
        Ok(Plan::new(ops, extra).preview().busy(&target, from))
    })
}
