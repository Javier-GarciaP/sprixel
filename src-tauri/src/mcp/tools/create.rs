//! The tools that create things: folders, sprites and frames. They need the
//! "Create sprites" permission and follow the same rules and errors as the
//! app (`name_exists`, `too_big`...).

use super::args;
use super::write::{Plan, write};
use crate::document::{DEFAULT_FRAME_MS, Operation, SpriteRef};
use crate::ipc::browser::create_sprite_operations;
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "create_folder",
            description: concat!("Creates a folder in the project. `parent` is the folder to create it in (omit for the project root). Names are unique inside a folder, ignoring case.", write_note!()),
            schema: json!({ "type": "object", "required": ["name"], "properties": {
                "name": { "type": "string", "description": "The new folder name." },
                "parent": { "type": "string", "description": "A folder path like \"Characters\". Omit for the project root." },
                "base_revision": base_revision(),
            }}),
            need: Need::Create,
            run: create_folder,
        },
        Tool {
            name: "create_sprite",
            description: concat!("Creates an empty (transparent) sprite with one frame. The size is chosen now and cannot change later: 1 to 512 on each side. Without `palette` it uses the palette `main`.", write_note!()),
            schema: json!({ "type": "object", "required": ["name", "width", "height"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string", "description": "A folder path like \"Characters\". Omit for the project root." },
                "width": { "type": "integer", "minimum": 1, "maximum": 512 },
                "height": { "type": "integer", "minimum": 1, "maximum": 512 },
                "palette": { "type": "string", "description": "The name of an existing palette. Default `main`." },
                "base_revision": base_revision(),
            }}),
            need: Need::Create,
            run: create_sprite,
        },
        Tool {
            name: "add_frame",
            description: concat!("Adds a frame to a sprite: empty, or a copy of another frame with `copy_of`. Frames are numbered from 1. By default it goes at the end.", write_note!()),
            schema: json!({ "type": "object", "required": ["name"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "copy_of": { "type": "integer", "minimum": 1, "description": "The frame to copy. Omit for an empty frame." },
                "at": { "type": "integer", "minimum": 1, "description": "The place of the new frame (1 to frames + 1). Default: the end." },
                "base_revision": base_revision(),
            }}),
            need: Need::Create,
            run: add_frame,
        },
    ]
}

/// The schema of the optional `base_revision` of every write tool.
pub fn base_revision() -> Value {
    json!({
        "type": "integer", "minimum": 0,
        "description": "The revision you read. If the project changed since, the call fails with `stale_revision` and changes nothing."
    })
}

fn create_folder(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.to_string();
    let parent = args::folder_path(input, "parent")?.unwrap_or_default();
    write(ctx, input, |_session| {
        let path = args::join(&[parent.clone(), vec![name.clone()]].concat());
        Ok(Plan::new(vec![Operation::CreateFolder { parent, name }], json!({ "path": path })))
    })
}

fn create_sprite(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.to_string();
    let folder = args::folder_path(input, "folder")?.unwrap_or_default();
    let (width, height) = (args::required_u32(input, "width")?, args::required_u32(input, "height")?);
    let palette = args::optional_str(input, "palette")?.map(str::to_string);
    write(ctx, input, |session| {
        let ops = create_sprite_operations(&session.project, &folder, &name, width, height, palette.as_deref());
        let extra = json!({
            "name": name, "folder": args::join(&folder), "width": width, "height": height,
            "palette": palette.unwrap_or_else(|| "main".to_string()),
        });
        let created = SpriteRef { folder: folder.clone(), name: name.clone() };
        Ok(Plan::new(ops, extra).about(&created))
    })
}

fn add_frame(ctx: &Ctx, input: &Value) -> ToolResult {
    let copy_of = args::optional_u32(input, "copy_of")?;
    let at = args::optional_u32(input, "at")?;
    write(ctx, input, |session| {
        let target = args::sprite(&session.project, input)?;
        let sprite = session.project.find_sprite(&target).expect("just found");
        let frames = sprite.frame_count() as u32;
        let at = at.unwrap_or(frames + 1);
        if at == 0 || at > frames + 1 {
            return Err(ToolError::new(
                "bad_frame",
                format!("`at` is {at}; the new frame can go from 1 to {}", frames + 1),
            )
            .with("frame", at)
            .with("frames", frames));
        }
        if let Some(copy) = copy_of {
            super::read::check_frame(sprite, copy)?;
        }
        let ops = vec![Operation::InsertFrame { sprite: target.clone(), at, copy_of, duration_ms: DEFAULT_FRAME_MS }];
        Ok(Plan::new(ops, json!({ "frame": at, "frames": frames + 1 })).busy(&target, at))
    })
}
