//! The tools that organize animations: `create_animation`,
//! `duplicate_animation`, `rename_animation`, `delete_animation` and
//! `move_frame`. An animation is a tag over a range of frames; these tools
//! make it from the base frame without sending pixels. The rules are the ones
//! of the editor (`document::animation`), so the codes are the same.

use super::args;
use super::create::base_revision;
use super::draw::preview_arg;
use super::read::check_frame;
use super::write::{Plan, write};
use crate::document::{self, Operation, TagMode};
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
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
    },
    Tool {
        name: "duplicate_animation",
        description: concat!(
            "Copies an animation: its frames are copied at the end of the sprite (or at `at`) and tagged `new_name`, with the same speed and repeat mode. Use it for a variant (a fast walk, a hurt walk) that you then change. `sprite` is the sprite, `name` the animation to copy. Errors: `not_found`, `name_exists`, `animation_overlap`.",
            write_note!()
        ),
        schema: json!({ "type": "object", "required": ["sprite", "name", "new_name"], "properties": {
            "sprite": { "type": "string" },
            "folder": { "type": "string" },
            "name": { "type": "string", "description": "The animation to copy." },
            "new_name": { "type": "string", "description": "The name of the copy (unique in the sprite, ignoring case)." },
            "at": { "type": "integer", "minimum": 1, "description": "Where the first copied frame goes. Default: the end." },
            "base_revision": base_revision(),
        }}),
        need: Need::Edit,
        run: duplicate_animation,
    },
    Tool {
        name: "rename_animation",
        description: concat!(
            "Gives an animation another name. Its frames, speed and mode do not change. Errors: `not_found`, `name_exists`.",
            write_note!()
        ),
        schema: json!({ "type": "object", "required": ["sprite", "name", "new_name"], "properties": {
            "sprite": { "type": "string" },
            "folder": { "type": "string" },
            "name": { "type": "string", "description": "The current name." },
            "new_name": { "type": "string" },
            "base_revision": base_revision(),
        }}),
        need: Need::Edit,
        run: rename_animation,
    },
    Tool {
        name: "delete_animation",
        description: concat!(
            "Removes an animation. By default only the name goes and its frames stay in the sprite (they belong to no animation). With `with_frames` true the frames are deleted too, and the animations after them follow their frames; a sprite keeps at least one frame (`last_frame`). Error: `not_found`.",
            write_note!()
        ),
        schema: json!({ "type": "object", "required": ["sprite", "name"], "properties": {
            "sprite": { "type": "string" },
            "folder": { "type": "string" },
            "name": { "type": "string" },
            "with_frames": { "type": "boolean", "description": "Also delete its frames. Default false." },
            "base_revision": base_revision(),
        }}),
        need: Need::Edit,
        run: delete_animation,
    },
    Tool {
        name: "move_frame",
        description: concat!(
            "Moves a frame to another place in the sprite: frame `from` ends at position `to` (frames are numbered from 1). Every animation keeps the same frames, so its range follows them, and anchors follow too. Use it to put a frame in the right animation. Returns an image of the moved frame.",
            write_note!()
        ),
        schema: json!({ "type": "object", "required": ["name", "from", "to"], "properties": {
            "name": { "type": "string" },
            "folder": { "type": "string" },
            "from": { "type": "integer", "minimum": 1 },
            "to": { "type": "integer", "minimum": 1 },
            "preview": preview_arg(),
            "base_revision": base_revision(),
        }}),
        need: Need::Edit,
        run: move_frame,
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

fn duplicate_animation(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.to_string();
    let new_name = args::required_str(input, "new_name")?.to_string();
    let at = args::optional_u32(input, "at")?;
    write(ctx, input, |session| {
        let target = args::sprite_at(&session.project, input, "sprite")?;
        let sprite = session.project.find_sprite(&target).expect("just found");
        let count = sprite.frame_count() as u32;
        let source = sprite.tags.iter().find(|t| crate::document::same_name(&t.name, &name)).cloned();
        let ops = document::duplicate_animation(&session.project, &target, &name, &new_name, at)?;
        let source = source.expect("duplicate_animation found it");
        let from = at.unwrap_or(count + 1);
        let to = from + (source.to - source.from);
        let extra = json!({
            "animation": new_name, "copy_of": source.name, "from": from, "to": to,
            "fps": source.fps, "mode": source.mode.as_str(), "frames": count + (to - from + 1),
        });
        Ok(Plan::new(ops, extra).preview().busy(&target, from))
    })
}

fn rename_animation(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.to_string();
    let new_name = args::required_str(input, "new_name")?.to_string();
    write(ctx, input, |session| {
        let target = args::sprite_at(&session.project, input, "sprite")?;
        let ops = document::rename_animation(&session.project, &target, &name, &new_name)?;
        Ok(Plan::new(ops, json!({ "animation": new_name, "was": name })).about(&target))
    })
}

fn delete_animation(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.to_string();
    let with_frames = args::optional_bool(input, "with_frames")?.unwrap_or(false);
    write(ctx, input, |session| {
        let target = args::sprite_at(&session.project, input, "sprite")?;
        let ops = document::delete_animation(&session.project, &target, &name, with_frames)?;
        let deleted = ops.iter().filter(|op| matches!(op, Operation::DeleteFrame { .. })).count();
        let frames = session.project.find_sprite(&target).expect("just found").frame_count() - deleted;
        Ok(Plan::new(ops, json!({ "deleted": name, "deleted_frames": deleted, "frames": frames })).about(&target))
    })
}

fn move_frame(ctx: &Ctx, input: &Value) -> ToolResult {
    let (from, to) = (args::required_u32(input, "from")?, args::required_u32(input, "to")?);
    write(ctx, input, |session| {
        let target = args::sprite(&session.project, input)?;
        let sprite = session.project.find_sprite(&target).expect("just found");
        check_frame(sprite, from)?;
        check_frame(sprite, to)?;
        let ops = if from == to {
            Vec::new()
        } else {
            vec![Operation::MoveFrame { sprite: target.clone(), from, to, tags: None, anchors: None }]
        };
        Ok(Plan::new(ops, json!({ "moved_frame": from, "to": to })).preview().busy(&target, to))
    })
}
