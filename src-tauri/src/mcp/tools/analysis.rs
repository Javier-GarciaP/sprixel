//! The tools that finish and check a sprite for an agent: `auto_outline`,
//! `auto_shade` (they need "Edit pixels") and `analyze_sprite` (read-only). The
//! rules are in `crate::analysis`; here the arguments are read and checked and
//! the result is written like any other pixel write.

use super::args;
use super::create::base_revision;
use super::draw::{dry_run_arg, key_of, preview_arg, valid_keys};
use super::read::{check_frame, read};
use super::write::{Plan, write};
use crate::analysis::{self, Connectivity, Light, OutlineOptions, Region, ShadeOptions, View};
use crate::document::{Operation, PixelChange, Sprite, SpriteRef, TRANSPARENT};
use crate::mcp::{Ctx, Need, Output, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
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
                "dry_run": dry_run_arg(),
                "preview": preview_arg(),
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: auto_outline,
        },
        Tool {
            name: "auto_shade",
            description: concat!("Shades the pixels of one key with a ramp of tones, so you do not shade by hand. Give the `light` direction, the `base` key to shade and the ramp `keys` from dark to light (it includes `base`). A pixel of `base` is measured by how many steps it is from the edge of its area, going toward the light and away from it. The pixels on the edge that faces the light get the lightest key, the next row a little less light, and so on for as many keys as are lighter than `base`; the same on the edge that faces away with the darker keys. A pixel that is as near the lit edge as the dark edge keeps `base`. Pixels of other keys never change. `region` limits which pixels change. Returns `pixels_changed`, `lit`, `shadow` and an image of the frame.", write_note!()),
            schema: json!({ "type": "object", "required": ["name", "light", "base", "keys"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "frame": { "type": "integer", "minimum": 1, "description": "Default 1." },
                "light": { "type": "string", "enum": Light::NAMES, "description": "Where the light comes from." },
                "base": { "type": "string", "description": "The key of the pixels to shade." },
                "keys": { "type": "array", "items": { "type": "string" }, "minItems": 2, "maxItems": 8, "description": "The ramp from dark to light, for example [\"D\", \"B\", \"L\"]. A text like \"DBL\" also works. It must include `base`." },
                "region": { "type": "object", "required": ["x", "y", "width", "height"], "description": "Only pixels inside this box change.", "properties": {
                    "x": { "type": "integer", "minimum": 0 }, "y": { "type": "integer", "minimum": 0 },
                    "width": { "type": "integer", "minimum": 1 }, "height": { "type": "integer", "minimum": 1 },
                }},
                "dry_run": dry_run_arg(),
                "preview": preview_arg(),
                "base_revision": base_revision(),
            }}),
            need: Need::Edit,
            run: auto_shade,
        },
        Tool {
            name: "analyze_sprite",
            description: "Checks a sprite and reports what is wrong, as JSON. `colors` has the number of colors used and the palette keys that no frame uses. For each frame, `loose_pixels` are pixels that touch no other pixel and, if you give `outline_key`, `outline_gaps` are shape pixels that are open to the outside because the outline is missing there (holes do not need an outline). Each list has `count` and `at` (a list of [x, y], at most 50). With more than one frame, `changes` compares each frame with the next: `changed_pixels`, `bbox` of the change and `ground_kept` (the bottom `ground_rows` rows are equal, so the feet did not move). It changes nothing and needs no permission.",
            schema: json!({ "type": "object", "required": ["name"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "outline_key": { "type": "string", "description": "The palette key of the outline, to look for gaps in it." },
                "ground_rows": { "type": "integer", "minimum": 1, "description": "How many bottom rows are the ground. Default 2." },
            }}),
            need: Need::Nothing,
            run: analyze_sprite,
        },
    ]
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

/// The ramp: a list of one-key texts, or one text with a key per character.
fn ramp_of(input: &Value, valid: &[String]) -> Result<Vec<char>, ToolError> {
    let bad = |why: &str| ToolError::new("bad_argument", format!("`keys` {why}")).with("argument", "keys");
    let texts: Vec<String> = match &input["keys"] {
        Value::String(text) => text.chars().map(String::from).collect(),
        Value::Array(items) => items
            .iter()
            .map(|item| item.as_str().map(str::to_string).ok_or_else(|| bad("must be a list of keys")))
            .collect::<Result<_, _>>()?,
        _ => return Err(bad("must be a list of keys, from dark to light")),
    };
    if !(2..=8).contains(&texts.len()) {
        return Err(bad("must have 2 to 8 keys"));
    }
    let keys = texts.iter().map(|text| opaque_key(text, valid, "keys")).collect::<Result<Vec<char>, _>>()?;
    let mut sorted = keys.clone();
    sorted.sort();
    sorted.dedup();
    if sorted.len() != keys.len() {
        return Err(bad("must not repeat a key"));
    }
    Ok(keys)
}

/// The optional box `region`, inside the sprite.
fn region_of(input: &Value, sprite: &Sprite) -> Result<Option<Region>, ToolError> {
    let item = &input["region"];
    if item.is_null() {
        return Ok(None);
    }
    let number = |key: &str| {
        item[key].as_u64().and_then(|n| u32::try_from(n).ok()).ok_or_else(|| {
            ToolError::new("bad_argument", format!("`region.{key}` must be a whole number")).with("argument", "region")
        })
    };
    let region = Region { x: number("x")?, y: number("y")?, width: number("width")?, height: number("height")? };
    if region.width == 0 || region.height == 0 {
        return Err(ToolError::new("bad_argument", "`region.width` and `region.height` must be at least 1").with("argument", "region"));
    }
    if region.x as u64 + region.width as u64 > sprite.width as u64 || region.y as u64 + region.height as u64 > sprite.height as u64 {
        return Err(ToolError::new(
            "out_of_range",
            format!("the region is not inside the {}x{} sprite", sprite.width, sprite.height),
        )
        .with("width", sprite.width)
        .with("height", sprite.height));
    }
    Ok(Some(region))
}

fn auto_shade(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::optional_u32(input, "frame")?.unwrap_or(1);
    let light_text = args::required_str(input, "light")?.to_string();
    let base_text = args::required_str(input, "base")?.to_string();
    let light = Light::parse(&light_text).ok_or_else(|| {
        ToolError::new("bad_argument", format!("`light` is {light_text:?}; it must be one of: {}", Light::NAMES.join(", ")))
            .with("argument", "light")
    })?;
    write(ctx, input, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let valid = valid_keys(project, sprite);
        let base = opaque_key(&base_text, &valid, "base")?;
        let keys = ramp_of(input, &valid)?;
        if !keys.contains(&base) {
            return Err(ToolError::new("bad_argument", "`keys` must include `base`").with("argument", "keys"));
        }
        let region = region_of(input, sprite)?;
        let shade = analysis::auto_shade(view_of(sprite, frame), &ShadeOptions { light, base, keys: &keys, region });
        let (count, lit, shadow) = (shade.changes.len(), shade.lit, shade.shadow);
        let ops = set_ops(&target, frame, shade.changes);
        Ok(Plan::new(ops, json!({ "pixels_changed": count, "lit": lit, "shadow": shadow })).preview().busy(&target, frame))
    })
}

/// The most coordinates a list of the report has.
const MAX_LISTED: usize = 50;

/// `{ "count": all of them, "at": the first 50 as [x, y] }`.
fn listed(found: &[(u32, u32)]) -> Value {
    json!({ "count": found.len(), "at": found.iter().take(MAX_LISTED).map(|(x, y)| json!([x, y])).collect::<Vec<_>>() })
}

fn analyze_sprite(ctx: &Ctx, input: &Value) -> ToolResult {
    let outline_text = args::optional_str(input, "outline_key")?.map(str::to_string);
    let ground_rows = args::optional_u32(input, "ground_rows")?.unwrap_or(2);
    if ground_rows == 0 {
        return Err(ToolError::new("bad_argument", "`ground_rows` must be 1 or more").with("argument", "ground_rows"));
    }
    read(ctx, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        let valid = valid_keys(project, sprite);
        let outline_key = outline_text.as_deref().map(|text| opaque_key(text, &valid, "outline_key")).transpose()?;
        let frames: Vec<View> = (1..=sprite.frame_count() as u32).map(|n| view_of(sprite, n)).collect();
        let palette: Vec<char> = project.palette(&sprite.palette).map(|p| p.colors.iter().map(|c| c.key).collect()).unwrap_or_default();
        let (used, unused) = analysis::colors(&frames, &palette);
        let keys = |list: &[char]| list.iter().map(|k| k.to_string()).collect::<Vec<_>>();
        let reports: Vec<Value> = frames
            .iter()
            .enumerate()
            .map(|(i, view)| {
                let mut report = json!({ "frame": i + 1, "loose_pixels": listed(&analysis::loose_pixels(*view)) });
                if let Some(key) = outline_key {
                    report["outline_gaps"] = listed(&analysis::outline_gaps(*view, key));
                }
                report
            })
            .collect();
        let mut answer = json!({
            "sprite": args::join(&[target.folder.clone(), vec![target.name.clone()]].concat()),
            "revision": project.revision,
            "width": sprite.width, "height": sprite.height, "frames": frames.len(),
            "colors": { "count": used.len(), "used": keys(&used), "unused": keys(&unused) },
            "frame_reports": reports,
        });
        if frames.len() > 1 {
            answer["changes"] = frames
                .windows(2)
                .enumerate()
                .map(|(i, pair)| {
                    let change = analysis::compare(pair[0], pair[1], ground_rows as usize);
                    let bbox = change.bbox.map(|b| json!({ "x": b.x, "y": b.y, "width": b.width, "height": b.height }));
                    json!({ "from": i + 1, "to": i + 2, "changed_pixels": change.changed_pixels, "bbox": bbox, "ground_kept": change.ground_kept })
                })
                .collect();
        }
        Ok(Output::Json(answer))
    })
}
