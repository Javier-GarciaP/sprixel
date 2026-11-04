//! Named parts: `set_part` and `remove_part`. A part is a named rectangle of a
//! sprite (`head`, `tail`) that `transform_region` and `copy_region` accept
//! as `part` instead of `region`, so an agent does not repeat the numbers.

use super::args;
use super::create::base_revision;
use super::write::{Plan, write};
use crate::document::{Operation, Part, Sprite};
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "set_part",
            description: concat!(
                "Names a rectangle of a sprite, for example `head` or `tail`, so you can use it later with `part` in `transform_region` and `copy_region` (on any frame) instead of writing the numbers again. `x` and `y` are the top left pixel and the rectangle must fit in the sprite (`region_out`). Setting the same name again (ignoring case) replaces it. A sprite has at most 64 parts. `get_sprite` lists the parts. If you change the canvas size with `resize_sprite`, set the parts again.",
                write_note!()
            ),
            schema: json!({ "type": "object", "required": ["sprite", "name", "x", "y", "width", "height"], "properties": {
                "sprite": { "type": "string", "description": "The sprite that gets the part." },
                "folder": { "type": "string" },
                "name": { "type": "string", "description": "The part name, for example `head`. 1 to 64 characters." },
                "x": { "type": "integer", "minimum": 0 },
                "y": { "type": "integer", "minimum": 0 },
                "width": { "type": "integer", "minimum": 1 },
                "height": { "type": "integer", "minimum": 1 },
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: set_part,
        },
        Tool {
            name: "remove_part",
            description: concat!("Removes a part by name. The pixels do not change. Fails with `not_found` if there is no such part.", write_note!()),
            schema: json!({ "type": "object", "required": ["sprite", "name"], "properties": {
                "sprite": { "type": "string" },
                "folder": { "type": "string" },
                "name": { "type": "string", "description": "The part name." },
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: remove_part,
        },
    ]
}

fn set_part(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.trim().to_string();
    let (x, y) = (args::required_u32(input, "x")?, args::required_u32(input, "y")?);
    let (width, height) = (args::required_u32(input, "width")?, args::required_u32(input, "height")?);
    write(ctx, input, |session| {
        let target = args::sprite_at(&session.project, input, "sprite")?;
        let part = Part { name: name.clone(), x, y, width, height };
        let sprite = session.project.find_sprite(&target).expect("just found");
        part.validate(sprite.width, sprite.height)?;
        let parts = sprite.parts.len() + usize::from(find(sprite, &name).is_none());
        let ops = vec![Operation::SetPart { sprite: target.clone(), part, index: None }];
        Ok(Plan::new(ops, json!({ "part": name, "x": x, "y": y, "width": width, "height": height, "parts": parts })).about(&target))
    })
}

fn remove_part(ctx: &Ctx, input: &Value) -> ToolResult {
    let name = args::required_str(input, "name")?.to_string();
    write(ctx, input, |session| {
        let target = args::sprite_at(&session.project, input, "sprite")?;
        let ops = vec![Operation::RemovePart { sprite: target.clone(), name: name.clone() }];
        Ok(Plan::new(ops, json!({ "removed": name })).about(&target))
    })
}

fn find<'a>(sprite: &'a Sprite, name: &str) -> Option<&'a Part> {
    sprite.parts.iter().find(|p| crate::document::same_name(&p.name, name))
}

/// The rectangle an region tool works on: the argument `part` (a name), or the
/// argument `region`, or `None` for neither. Both together are `bad_argument`;
/// an unknown name is `not_found` and lists the parts; a part that no longer
/// fits the sprite (after a canvas change) is `region_out`.
pub(super) fn rect_of(input: &Value, sprite: &Sprite, region_key: &str) -> Result<Option<(u32, u32, u32, u32)>, ToolError> {
    let Some(name) = args::optional_str(input, "part")? else {
        return args::region(input, region_key, sprite.width, sprite.height);
    };
    if !input[region_key].is_null() {
        return Err(ToolError::new("bad_argument", format!("use `part` or `{region_key}`, not both")).with("argument", "part"));
    }
    let part = find(sprite, name).ok_or_else(|| {
        let names: Vec<&str> = sprite.parts.iter().map(|p| p.name.as_str()).collect();
        ToolError::new(
            "not_found",
            format!(
                "there is no part {name:?} in {:?}; the parts are: {}. Name one with `set_part`",
                sprite.name,
                if names.is_empty() { "(none)".to_string() } else { names.join(", ") }
            ),
        )
        .with("name", name)
    })?;
    part.validate(sprite.width, sprite.height)?;
    Ok(Some((part.x, part.y, part.width, part.height)))
}
