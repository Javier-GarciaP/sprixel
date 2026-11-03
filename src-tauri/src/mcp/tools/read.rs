//! The tools that only read: they never change the project, the history or
//! the revision.

use super::args;
use crate::document::{Project, SpriteRef, Tag, write_grid};
use crate::ipc::project;
use crate::mcp::{Ctx, Need, Output, Tool, ToolError, ToolResult};
use crate::render::render_frame;
use serde_json::{Value, json};

/// The event that asks the UI to show a sprite in the editor.
pub const OPEN_SPRITE_EVENT: &str = "mcp://open-sprite";

/// The largest preview, in pixels per side. Bigger images cost the AI many
/// tokens and tell it nothing more.
const MAX_PREVIEW_SIDE: u32 = 1024;
const DEFAULT_SCALE: u32 = 8;

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "get_project",
            description: "Reads the open project: its name, revision, folders, palettes and how many sprites it has. Start here.",
            schema: json!({ "type": "object", "properties": {} }),
            need: Need::Nothing,
            run: get_project,
        },
        Tool {
            name: "list_sprites",
            description: "Lists the sprites with their size, frames, palette, animations and who edited them last. Give `folder` to list only one folder (and its sub-folders).",
            schema: json!({ "type": "object", "properties": {
                "folder": { "type": "string", "description": "A folder path like \"Characters/Humans\". Omit for the whole project." }
            }}),
            need: Need::Nothing,
            run: list_sprites,
        },
        Tool {
            name: "get_sprite",
            description: "Reads one sprite: size, palette name, animations and its frames as a text grid.",
            schema: json!({ "type": "object", "required": ["name"], "properties": {
                "name": { "type": "string", "description": "The sprite name, or a path like \"Characters/goblin\"." },
                "folder": { "type": "string", "description": "The folder of the sprite, if two sprites share the name." },
                "frame": { "type": "integer", "minimum": 1, "description": "Read only this frame (from 1). Omit for all frames." }
            }}),
            need: Need::Nothing,
            run: get_sprite,
        },
        Tool {
            name: "get_palette",
            description: "Reads a palette: each color with its one-character key, name and hex value. Use only these keys when drawing.",
            schema: json!({ "type": "object", "properties": {
                "palette": { "type": "string", "description": "The palette name. Omit to use the palette of `name`, or `main`." },
                "name": { "type": "string", "description": "A sprite, to read its palette." },
                "folder": { "type": "string" }
            }}),
            need: Need::Nothing,
            run: get_palette,
        },
        Tool {
            name: "render_preview",
            description: "Returns a PNG image of one frame of a sprite, so you can see your work. The image is scaled up with sharp pixels.",
            schema: json!({ "type": "object", "required": ["name"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "frame": { "type": "integer", "minimum": 1, "description": "The frame, from 1. Default 1." },
                "scale": { "type": "integer", "minimum": 1, "maximum": 64, "description": "Each pixel becomes scale x scale pixels. Default 8. The image is at most 1024 pixels on a side." }
            }}),
            need: Need::Nothing,
            run: render_preview,
        },
        Tool {
            name: "open_sprite",
            description: "Shows a sprite in the editor, so the person sees what you work on.",
            schema: json!({ "type": "object", "required": ["name"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" }
            }}),
            need: Need::Nothing,
            run: open_sprite,
        },
    ]
}

/// Looks at the open project.
fn read<R>(ctx: &Ctx, look: impl FnOnce(&crate::storage::Session) -> Result<R, ToolError>) -> Result<R, ToolError> {
    project::read(ctx.core.host.project(), look).ok_or_else(ToolError::no_project)?
}

fn tag_json(tag: &Tag) -> Value {
    json!({ "name": tag.name, "from": tag.from, "to": tag.to, "fps": tag.fps, "mode": tag.mode.as_str() })
}

fn folders_of(folder: &crate::document::Folder, path: &mut Vec<String>, out: &mut Vec<String>) {
    for child in &folder.folders {
        path.push(child.name.clone());
        out.push(args::join(path));
        folders_of(child, path, out);
        path.pop();
    }
}

fn get_project(ctx: &Ctx, _args: &Value) -> ToolResult {
    read(ctx, |session| {
        let project = &session.project;
        let mut folders = Vec::new();
        folders_of(&project.root, &mut Vec::new(), &mut folders);
        let sprites = count_sprites(&project.root);
        Ok(Output::Json(json!({
            "name": project.name,
            "revision": project.revision,
            "folders": folders,
            "sprite_count": sprites,
            "palettes": project.palettes.iter()
                .map(|p| json!({ "name": p.name, "colors": p.colors.len() }))
                .collect::<Vec<_>>(),
        })))
    })
}

fn count_sprites(folder: &crate::document::Folder) -> usize {
    folder.sprites.len() + folder.folders.iter().map(count_sprites).sum::<usize>()
}

fn sprite_summary(session: &crate::storage::Session, folder: &[String], sprite: &crate::document::Sprite) -> Value {
    let editor = session
        .history
        .last_editor(&SpriteRef { folder: folder.to_vec(), name: sprite.name.clone() });
    json!({
        "name": sprite.name,
        "folder": args::join(folder),
        "width": sprite.width,
        "height": sprite.height,
        "frames": sprite.frame_count(),
        "palette": sprite.palette,
        "animations": sprite.tags.iter().map(tag_json).collect::<Vec<_>>(),
        "last_edited_by": editor.map(|e| json!({ "author": e.author, "at_ms": e.at_ms })),
    })
}

fn list_into(session: &crate::storage::Session, folder: &crate::document::Folder, path: &mut Vec<String>, out: &mut Vec<Value>) {
    for sprite in &folder.sprites {
        out.push(sprite_summary(session, path, sprite));
    }
    for child in &folder.folders {
        path.push(child.name.clone());
        list_into(session, child, path, out);
        path.pop();
    }
}

fn list_sprites(ctx: &Ctx, input: &Value) -> ToolResult {
    let start = args::folder_path(input, "folder")?.unwrap_or_default();
    read(ctx, |session| {
        let folder = session.project.find_folder(&start).ok_or_else(|| {
            ToolError::new("not_found", format!("there is no folder {:?}", args::join(&start))).with("folder", args::join(&start))
        })?;
        let mut sprites = Vec::new();
        list_into(session, folder, &mut start.clone(), &mut sprites);
        Ok(Output::Json(json!({ "revision": session.project.revision, "sprites": sprites })))
    })
}

/// Checks a frame number (from 1) of a sprite.
pub fn check_frame(sprite: &crate::document::Sprite, frame: u32) -> Result<(), ToolError> {
    if frame == 0 || frame as usize > sprite.frame_count() {
        return Err(ToolError::new(
            "bad_frame",
            format!("frame {frame} does not exist; frames go from 1 to {}", sprite.frame_count()),
        )
        .with("frame", frame)
        .with("frames", sprite.frame_count()));
    }
    Ok(())
}

fn get_sprite(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::optional_u32(input, "frame")?;
    read(ctx, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        if let Some(frame) = frame {
            check_frame(sprite, frame)?;
        }
        let palette = project.palette(&sprite.palette).ok_or_else(|| {
            ToolError::new("palette_not_found", format!("the palette {:?} of this sprite does not exist", sprite.palette))
        })?;
        let mut summary = sprite_summary(session, &target.folder, sprite);
        summary["revision"] = json!(project.revision);
        summary["grid"] = json!(write_grid(sprite, palette, frame));
        Ok(Output::Json(summary))
    })
}

fn get_palette(ctx: &Ctx, input: &Value) -> ToolResult {
    let palette_name = args::optional_str(input, "palette")?.map(str::to_string);
    read(ctx, |session| {
        let project = &session.project;
        let name = match (palette_name, args::optional_str(input, "name")?) {
            (Some(name), _) => name,
            (None, Some(_)) => {
                let target = args::sprite(project, input)?;
                project.find_sprite(&target).expect("just found").palette.clone()
            }
            (None, None) => "main".to_string(),
        };
        let palette = project.palette(&name).ok_or_else(|| {
            let names: Vec<&str> = project.palettes.iter().map(|p| p.name.as_str()).collect();
            ToolError::new("not_found", format!("there is no palette {name:?}; the palettes are: {}", names.join(", ")))
                .with("name", name.clone())
        })?;
        let used = project.used_keys(&palette.name);
        Ok(Output::Json(json!({
            "name": palette.name,
            "revision": project.revision,
            "max_colors": crate::document::MAX_COLORS,
            "colors": palette.colors.iter().map(|c| json!({
                "key": c.key.to_string(), "name": c.name, "hex": c.hex, "used": used.contains(&c.key),
            })).collect::<Vec<_>>(),
            "transparent": ".",
        })))
    })
}

fn render_preview(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::optional_u32(input, "frame")?.unwrap_or(1);
    let scale = args::optional_u32(input, "scale")?.unwrap_or(DEFAULT_SCALE);
    if !(1..=64).contains(&scale) {
        return Err(ToolError::new("bad_argument", "`scale` must be from 1 to 64").with("argument", "scale"));
    }
    let (image, info) = read(ctx, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let side = sprite.width.max(sprite.height) * scale;
        if side > MAX_PREVIEW_SIDE {
            let largest = (MAX_PREVIEW_SIDE / sprite.width.max(sprite.height)).max(1);
            return Err(ToolError::new(
                "too_big",
                format!("the preview would be {side} pixels on a side; the maximum is {MAX_PREVIEW_SIDE}. Use scale {largest} or less"),
            )
            .with("limit", MAX_PREVIEW_SIDE)
            .with("max_scale", largest));
        }
        let palette = project.palette(&sprite.palette).ok_or_else(|| {
            ToolError::new("palette_not_found", format!("the palette {:?} of this sprite does not exist", sprite.palette))
        })?;
        let image = render_frame(sprite, palette, frame as usize - 1, scale);
        let info = json!({
            "name": sprite.name, "folder": args::join(&target.folder), "frame": frame, "scale": scale,
            "width": image.width, "height": image.height, "revision": project.revision,
        });
        Ok((image, info))
    })?;
    let png = crate::export::encode_png(&image).map_err(|e| ToolError::new(e.code, e.message))?;
    Ok(Output::Image { png, info })
}

fn open_sprite(ctx: &Ctx, input: &Value) -> ToolResult {
    let target = read(ctx, |session| args::sprite(&session.project, input))?;
    ctx.core.host.emit(OPEN_SPRITE_EVENT, json!({ "folder": target.folder, "name": target.name }));
    Ok(Output::Json(json!({ "opened": true, "name": target.name, "folder": args::join(&target.folder) })))
}
