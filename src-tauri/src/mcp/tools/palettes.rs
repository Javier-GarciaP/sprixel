//! The tools about palettes: create one, point a sprite to one, and (later)
//! clean and extend them. They need the "Edit" permission.

use super::args;
use super::create::base_revision;
use super::write::{Plan, write};
use crate::document::{Color, ColorEdit, Operation, Palette, check_colors};
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "create_palette",
            description: concat!("Creates a palette. `colors` is a list of `{key?, hex, name?}`: a color without `key` gets the next free key. Palette names are unique ignoring case. Point a sprite to it with `set_sprite_palette`, or create the sprite with `own_palette: true`.", write_note!()),
            schema: json!({ "type": "object", "required": ["name"], "properties": {
                "name": { "type": "string", "description": "The new palette name." },
                "colors": { "type": "array", "description": "The colors, in order.", "items": { "type": "object", "required": ["hex"], "properties": {
                    "key": { "type": "string", "description": "One printable ASCII character, not a space and not `.`. Omit to get the next free key." },
                    "hex": { "type": "string", "description": "#RRGGBB" },
                    "name": { "type": "string", "description": "Unique inside the palette. Default: the hex without `#`." },
                }}},
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: create_palette,
        },
        Tool {
            name: "set_sprite_palette",
            description: concat!("Makes a sprite use another existing palette. No pixel changes, so every key the sprite uses must exist in that palette, or the call fails with `keys_missing`.", write_note!()),
            schema: json!({ "type": "object", "required": ["name", "palette"], "properties": {
                "name": { "type": "string", "description": "The sprite name." },
                "folder": { "type": "string" },
                "palette": { "type": "string", "description": "The name of an existing palette." },
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: set_sprite_palette,
        },
        Tool {
            name: "remove_unused_colors",
            description: concat!("Removes from one palette the colors that no sprite uses, and returns their keys (`removed`). Colors in use stay. `.` is transparent, not a color, and is never touched.", write_note!()),
            schema: json!({ "type": "object", "required": ["palette"], "properties": {
                "palette": { "type": "string", "description": "The name of an existing palette." },
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: remove_unused_colors,
        },
    ]
}

/// The colors of a `colors` argument. Keys that are not given are the next
/// free keys, which never collide with a key written later in the list.
pub fn read_colors(input: &Value) -> Result<Vec<Color>, ToolError> {
    let items = match args::object(input)?.get("colors") {
        None | Some(Value::Null) => return Ok(Vec::new()),
        Some(Value::Array(items)) => items,
        Some(_) => return Err(ToolError::new("bad_argument", "`colors` must be a list").with("argument", "colors")),
    };
    let bad = |what: &str| ToolError::new("bad_argument", format!("each color needs {what}")).with("argument", "colors");
    let mut parsed: Vec<(Option<char>, String, Option<String>)> = Vec::new();
    for item in items {
        let hex = args::required_str(item, "hex").map_err(|_| bad("a `hex`"))?.to_ascii_uppercase();
        let key = match args::optional_str(item, "key")? {
            None => None,
            Some(text) => {
                let mut chars = text.chars();
                match (chars.next(), chars.next()) {
                    (Some(key), None) => Some(key),
                    _ => return Err(ToolError::new("bad_key", "a key is one printable ASCII character").with("argument", "colors")),
                }
            }
        };
        parsed.push((key, hex, args::optional_str(item, "name")?.map(|n| n.trim().to_string())));
    }
    // The keys that were given are reserved before the others are chosen.
    let mut taken = Palette {
        name: String::new(),
        colors: parsed
            .iter()
            .filter_map(|(key, ..)| key.map(|key| Color { key, name: String::new(), hex: String::new() }))
            .collect(),
    };
    let mut colors = Vec::new();
    for (key, hex, name) in parsed {
        let key = match key {
            Some(key) => key,
            None => {
                let key = taken.next_free_key().ok_or_else(|| ToolError::new("palette_full", "a palette has at most 93 colors"))?;
                taken.colors.push(Color { key, name: String::new(), hex: String::new() });
                key
            }
        };
        let name = name.unwrap_or_else(|| hex.trim_start_matches('#').to_string());
        colors.push(Color { key, name, hex });
    }
    check_colors(&colors)?;
    Ok(colors)
}

fn create_palette(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.trim().to_string();
    let colors = read_colors(input)?;
    write(ctx, input, |_session| {
        let shown: Vec<Value> = colors
            .iter()
            .map(|c| json!({ "key": c.key.to_string(), "name": c.name, "hex": c.hex }))
            .collect();
        let palette = Palette { name: name.clone(), colors };
        let ops = vec![Operation::CreatePalette { palette, index: None }];
        Ok(Plan::new(ops, json!({ "palette": name, "colors": shown })))
    })
}

fn set_sprite_palette(ctx: &Ctx, input: &Value) -> ToolResult {
    let palette = args::required_str(input, "palette")?.to_string();
    write(ctx, input, |session| {
        let target = args::sprite(&session.project, input)?;
        let real = session
            .project
            .palette(&palette)
            .map(|p| p.name.clone())
            .ok_or_else(|| ToolError::new("not_found", format!("there is no palette {palette:?}")).with("name", palette.clone()))?;
        let ops = vec![Operation::SetSpritePalette { sprite: target.clone(), palette: real.clone() }];
        Ok(Plan::new(ops, json!({ "palette": real })).about(&target))
    })
}

fn remove_unused_colors(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "palette")?.to_string();
    write(ctx, input, |session| {
        let project = &session.project;
        let palette = project
            .palette(&name)
            .ok_or_else(|| ToolError::new("not_found", format!("there is no palette {name:?}")).with("name", name.clone()))?;
        let used = project.used_keys(&palette.name);
        let (keep, drop): (Vec<&Color>, Vec<&Color>) = palette.colors.iter().partition(|c| used.contains(&c.key));
        let edits: Vec<ColorEdit> =
            keep.iter().map(|c| ColorEdit { from: Some(c.key), color: (*c).clone() }).collect();
        let ops = project.palette_edit_operations(&palette.name, &edits, &[])?;
        let removed: Vec<String> = drop.iter().map(|c| c.key.to_string()).collect();
        Ok(Plan::new(ops, json!({ "palette": palette.name, "removed": removed })))
    })
}
