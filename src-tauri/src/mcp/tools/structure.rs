//! The tools that change the structure of a sprite or a palette:
//! `delete_frame`, `create_tag`, `set_palette_color` and `delete_sprite`. They
//! use the same rules and error codes as the app.

use super::args;
use super::create::base_revision;
use super::read::check_frame;
use super::write::{Plan, write};
use crate::document::{Color, ColorEdit, Operation, Tag, TagMode};
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "delete_frame",
            description: concat!("Deletes one frame of a sprite (frames are numbered from 1). A sprite keeps at least one frame. Animations (tags) follow the frames.", write_note!()),
            schema: json!({ "type": "object", "required": ["name", "frame"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "frame": { "type": "integer", "minimum": 1 },
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: delete_frame,
        },
        Tool {
            name: "create_tag",
            description: concat!("Creates an animation (a tag): a named range of frames with a speed. `sprite` is the sprite, `name` is the name of the new animation. Frames are numbered from 1 and both ends are included.", write_note!()),
            schema: json!({ "type": "object", "required": ["sprite", "name", "from", "to", "fps"], "properties": {
                "sprite": { "type": "string", "description": "The sprite that gets the animation." },
                "folder": { "type": "string" },
                "name": { "type": "string", "description": "The animation name, for example `walk`." },
                "from": { "type": "integer", "minimum": 1 },
                "to": { "type": "integer", "minimum": 1 },
                "fps": { "type": "integer", "minimum": 1, "maximum": 60 },
                "mode": { "type": "string", "enum": ["loop", "once", "pingpong"], "description": "Default `loop`." },
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: create_tag,
        },
        Tool {
            name: "set_palette_color",
            description: concat!("Adds a color to a palette, or changes the name or the hex value of the color that has this `key`. A key is one printable character that is not a space and not `.`. Without `key` a new color gets the next free key. A palette has at most 93 colors. Pixels keep their key, so changing a color recolors every pixel that uses it.", write_note!()),
            schema: json!({ "type": "object", "required": ["hex"], "properties": {
                "palette": { "type": "string", "description": "The palette name. Default: the palette of `sprite`, or `main`." },
                "sprite": { "type": "string", "description": "Use the palette of this sprite." },
                "folder": { "type": "string" },
                "key": { "type": "string", "description": "One character. If the palette has it, that color is changed; if not, it is added with this key." },
                "name": { "type": "string", "description": "The color name, unique in the palette. Default: the hex without `#`." },
                "hex": { "type": "string", "description": "`#RRGGBB`." },
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: set_palette_color,
        },
        Tool {
            name: "delete_sprite",
            description: concat!("Moves a sprite to the trash. The person can undo it. It needs the permission \"Delete sprites\", which is off by default.", write_note!()),
            schema: json!({ "type": "object", "required": ["name"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "base_revision": base_revision(),
            }}),
            need: Need::Delete,
            run: delete_sprite,
        },
    ]
}

fn delete_frame(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::required_u32(input, "frame")?;
    write(ctx, input, |session| {
        let target = args::sprite(&session.project, input)?;
        let sprite = session.project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let frames = sprite.frame_count() - 1;
        let ops = vec![Operation::DeleteFrame { sprite: target.clone(), frame }];
        Ok(Plan::new(ops, json!({ "deleted_frame": frame, "frames": frames })).busy(&target, frame))
    })
}

fn create_tag(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.to_string();
    let (from, to, fps) = (
        args::required_u32(input, "from")?,
        args::required_u32(input, "to")?,
        args::required_u32(input, "fps")?,
    );
    let mode_text = args::optional_str(input, "mode")?.unwrap_or("loop");
    let mode = TagMode::parse(mode_text).ok_or_else(|| {
        ToolError::new("bad_mode", format!("{mode_text:?} is not a repeat mode; use loop, once or pingpong"))
            .with("argument", "mode")
    })?;
    write(ctx, input, |session| {
        let target = args::sprite_at(&session.project, input, "sprite")?;
        let tag = Tag { name: name.clone(), from, to, fps, mode };
        let ops = vec![Operation::AddTag { sprite: target.clone(), tag, index: None }];
        let extra = json!({ "animation": name, "from": from, "to": to, "fps": fps, "mode": mode.as_str() });
        Ok(Plan::new(ops, extra).about(&target))
    })
}

fn set_palette_color(ctx: &Ctx, input: &Value) -> ToolResult {
    let hex = args::required_str(input, "hex")?.to_ascii_uppercase();
    let key_text = args::optional_str(input, "key")?;
    let key = match key_text {
        None => None,
        Some(text) => {
            let mut chars = text.chars();
            match (chars.next(), chars.next()) {
                (Some(key), None) => Some(key),
                _ => return Err(ToolError::new("bad_key", "a key is one printable ASCII character").with("argument", "key")),
            }
        }
    };
    let color_name = args::optional_str(input, "name")?.map(|n| n.trim().to_string());
    let palette_arg = args::optional_str(input, "palette")?.map(str::to_string);
    write(ctx, input, |session| {
        let project = &session.project;
        let palette_name = match (palette_arg, args::optional_str(input, "sprite")?) {
            (Some(name), _) => name,
            (None, Some(_)) => {
                let target = args::sprite_at(project, input, "sprite")?;
                project.find_sprite(&target).expect("just found").palette.clone()
            }
            (None, None) => "main".to_string(),
        };
        let palette = project.palette(&palette_name).ok_or_else(|| {
            ToolError::new("not_found", format!("there is no palette {palette_name:?}")).with("name", palette_name.clone())
        })?;
        // The editor session of the app: every color stays, one is added or changed.
        let mut edits: Vec<ColorEdit> = palette
            .colors
            .iter()
            .map(|color| ColorEdit { from: Some(color.key), color: color.clone() })
            .collect();
        let changed = key.and_then(|key| edits.iter().position(|e| e.color.key == key));
        let added = changed.is_none();
        match changed {
            Some(at) => {
                let old = &edits[at].color;
                edits[at].color = Color {
                    key: old.key,
                    name: color_name.clone().unwrap_or_else(|| old.name.clone()),
                    hex: hex.clone(),
                };
            }
            None => {
                let key = key.or_else(|| palette.next_free_key()).ok_or_else(|| {
                    ToolError::new("palette_full", "a palette has at most 93 colors")
                })?;
                let name = color_name.clone().unwrap_or_else(|| hex.trim_start_matches('#').to_string());
                edits.push(ColorEdit { from: None, color: Color { key, name, hex: hex.clone() } });
            }
        }
        let result = edits[changed.unwrap_or(edits.len() - 1)].color.clone();
        let ops = project.palette_edit_operations(&palette.name, &edits, &[])?;
        let extra = json!({
            "palette": palette.name, "action": if added { "added" } else { "changed" },
            "key": result.key.to_string(), "name": result.name, "hex": result.hex,
        });
        Ok(Plan::new(ops, extra))
    })
}

fn delete_sprite(ctx: &Ctx, input: &Value) -> ToolResult {
    write(ctx, input, |session| {
        let target = args::sprite(&session.project, input)?;
        let ops = vec![Operation::DeleteSprite { sprite: target.clone() }];
        Ok(Plan::new(ops, json!({ "deleted": target.name, "folder": args::join(&target.folder) })).about(&target))
    })
}
