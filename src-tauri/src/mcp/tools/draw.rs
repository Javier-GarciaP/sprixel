//! The drawing tools: `apply_grid`, `set_pixels` and `draw_shape`. They need
//! the "Edit pixels" permission. Each call is applied as one atomic action:
//! all of it, or nothing.

use super::args;
use super::create::base_revision;
use super::edit::{finish_pixels, only_over_arg, only_over_schema, symmetry_arg, symmetry_schema};
use super::read::check_frame;
use super::shapes;
use super::write::{Plan, write};
use crate::document::{Grid, GridError, GridOptions, Operation, PixelChange, Project, Sprite, SpriteRef, TRANSPARENT, same_name};
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "apply_grid",
            description: super::docs::APPLY_GRID,
            schema: json!({ "type": "object", "required": ["name", "grid"], "properties": {
                "name": { "type": "string", "description": "The sprite." },
                "folder": { "type": "string" },
                "grid": { "type": "string", "description": "The text grid (see the tool description)." },
                "frame": { "type": "integer", "minimum": 1, "description": "The frame to write when the grid has no `frame N` line (only rows)." },
                "x": { "type": "integer", "minimum": 0, "description": "With `x` and `y` the grid is written as a region whose top left pixel is (x, y). Without them it is a whole frame." },
                "y": { "type": "integer", "minimum": 0 },
                "skip_transparent": { "type": "boolean", "description": "A `.` in the grid keeps the pixel that is there, instead of erasing it." },
                "dry_run": dry_run_arg(),
                "preview": preview_arg(),
                "background": super::read::background_arg(),
                "base_revision": base_revision(),
                "only_over": only_over_schema(), "symmetry": symmetry_schema().0, "axis": symmetry_schema().1,
            }}),
            need: Need::Edit,
            run: apply_grid,
        },
        Tool {
            name: "set_pixels",
            description: concat!("Sets single pixels of one frame. Each pixel is `{x, y, key}`; x and y count from 0 at the top left. The key is one key of the palette (read them with get_palette), or `.` to erase. All pixels are checked first: if one is wrong, none is set.", write_note!()),
            schema: json!({ "type": "object", "required": ["name", "pixels"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "frame": { "type": "integer", "minimum": 1, "description": "Default 1." },
                "pixels": { "type": "array", "items": { "type": "object", "required": ["x", "y", "key"], "properties": {
                    "x": { "type": "integer", "minimum": 0 },
                    "y": { "type": "integer", "minimum": 0 },
                    "key": { "type": "string", "description": "One palette key, or `.` for transparent." },
                }}},
                "preview": preview_arg(),
                "background": super::read::background_arg(),
                "base_revision": base_revision(),
                "only_over": only_over_schema(), "symmetry": symmetry_schema().0, "axis": symmetry_schema().1,
            }}),
            need: Need::Edit,
            run: set_pixels,
        },
        Tool {
            name: "draw_shape",
            description: concat!("Draws a shape in one frame with one key of the palette (read them with get_palette; `.` erases). Shapes: `line` (x0, y0, x1, y1), `rectangle` and `ellipse` (x, y of the top left corner of the box, width, height, and `filled`), and `fill` (x, y: replaces the connected area of the same key, like a bucket). The whole shape must fit inside the sprite.", write_note!()),
            schema: json!({ "type": "object", "required": ["name", "shape", "key"], "properties": {
                "name": { "type": "string" },
                "folder": { "type": "string" },
                "frame": { "type": "integer", "minimum": 1, "description": "Default 1." },
                "shape": { "type": "string", "enum": ["line", "rectangle", "ellipse", "fill"] },
                "key": { "type": "string", "description": "One palette key, or `.` for transparent." },
                "x0": { "type": "integer", "minimum": 0 }, "y0": { "type": "integer", "minimum": 0 },
                "x1": { "type": "integer", "minimum": 0 }, "y1": { "type": "integer", "minimum": 0 },
                "x": { "type": "integer", "minimum": 0 }, "y": { "type": "integer", "minimum": 0 },
                "width": { "type": "integer", "minimum": 1 }, "height": { "type": "integer", "minimum": 1 },
                "filled": { "type": "boolean", "description": "For rectangle and ellipse. Default false (outline)." },
                "preview": preview_arg(),
                "background": super::read::background_arg(),
                "base_revision": base_revision(),
                "only_over": only_over_schema(), "symmetry": symmetry_schema().0, "axis": symmetry_schema().1,
            }}),
            need: Need::Edit,
            run: draw_shape,
        },
    ]
}

/// The `preview` argument of the three drawing tools.
pub(super) fn preview_arg() -> Value {
    json!({ "type": "boolean", "description": "Default true: the result carries an image of the edited frame (at most 512 pixels on a side). LOOK at it and fix what is wrong. Set false to skip it." })
}

/// The `dry_run` argument of the tools that can show their result first.
pub(super) fn dry_run_arg() -> Value {
    json!({ "type": "boolean", "description": "Default false. With true nothing is written (no new revision, no history entry): the answer has the same counts and an image of what the frame WOULD look like, with `dry_run` true and `would_change`. Use it to try before you commit." })
}

/// The keys a sprite accepts: those of its palette and `.`.
pub(super) fn valid_keys(project: &Project, sprite: &Sprite) -> Vec<String> {
    let mut keys: Vec<String> = project
        .palette(&sprite.palette)
        .map(|p| p.colors.iter().map(|c| c.key.to_string()).collect())
        .unwrap_or_default();
    keys.push(TRANSPARENT.to_string());
    keys
}

/// `unknown_key` with the keys that are valid, so the AI can pick one.
fn unknown_key(key: &str, valid: &[String]) -> ToolError {
    ToolError::new(
        "unknown_key",
        format!("the key {key:?} is not in the palette; the valid keys are: {}", valid.join(" ")),
    )
    .with("key", key)
    .with("valid_keys", valid.to_vec())
}

/// One palette key from a text argument.
pub(super) fn key_of(text: &str, valid: &[String]) -> Result<char, ToolError> {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(key), None) if valid.iter().any(|k| *k == text) => Ok(key),
        (Some(_), None) => Err(unknown_key(text, valid)),
        _ => Err(ToolError::new("bad_argument", "a key is exactly one character").with("key", text)),
    }
}

/// The error of an invalid grid: the code and place of the first problem, and
/// all the problems in `errors`, so one try can fix them all.
fn grid_failure(errors: Vec<GridError>, valid: &[String]) -> ToolError {
    let first = &errors[0];
    let mut message = first.message.clone();
    if errors.len() > 1 {
        message.push_str(&format!(" (and {} more, see `errors`)", errors.len() - 1));
    }
    let mut error = ToolError::new(first.code, message);
    for (key, value) in [("line", first.line), ("row", first.row), ("column", first.column)] {
        if let Some(value) = value {
            error = error.with(key, value);
        }
    }
    if errors.iter().any(|e| e.code == "unknown_key") {
        error = error.with("valid_keys", valid.to_vec());
    }
    error.with("errors", serde_json::to_value(&errors).unwrap_or_default())
}

/// A grid that has only rows can come with `frame`: the line `frame N` is put
/// before the first row (the first line that is not `palette:`, `legend`, a
/// comment or a legend line, which always has a space; a row never has one).
fn with_frame_line(text: &str, frame: u32) -> String {
    let has_frame = text.lines().any(|l| {
        let l = l.trim();
        l.strip_prefix("frame").is_some_and(|rest| rest.starts_with(char::is_whitespace))
    });
    if has_frame {
        return text.to_string();
    }
    let mut out = String::new();
    let mut put = false;
    for line in text.lines() {
        let trimmed = line.trim();
        let header = trimmed.is_empty()
            || trimmed.starts_with("palette:")
            || trimmed == "legend"
            || trimmed.contains(char::is_whitespace);
        if !put && !header {
            out.push_str(&format!("frame {frame}\n"));
            put = true;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// A `palette:` line is never ignored: it must name the palette of the sprite.
fn check_palette_line(project: &Project, target: &SpriteRef, sprite: &Sprite, text: &str) -> Result<(), ToolError> {
    let Some(named) = Grid::parse(text).0.palette else { return Ok(()) };
    if same_name(&named, &sprite.palette) {
        return Ok(());
    }
    if project.palette(&named).is_none() {
        let names: Vec<String> = project.palettes.iter().map(|p| p.name.clone()).collect();
        return Err(ToolError::new(
            "palette_not_found",
            format!(
                "the grid says `palette: {named}` but there is no such palette; the palettes are: {}. Create it with create_palette, or remove the `palette:` line",
                names.join(", ")
            ),
        )
        .with("palette", named)
        .with("palettes", names));
    }
    Err(ToolError::new(
        "palette_mismatch",
        format!(
            "the grid says `palette: {named}` but sprite {:?} uses the palette {:?}. Remove the `palette:` line to use {:?}, or call set_sprite_palette to make the sprite use {named:?}, or create the sprite with `own_palette: true`",
            target.name, sprite.palette, sprite.palette
        ),
    )
    .with("sprite_palette", sprite.palette.clone())
    .with("grid_palette", named))
}

fn apply_grid(ctx: &Ctx, input: &Value) -> ToolResult {
    let text = args::required_str(input, "grid")?.to_string();
    let frame = args::optional_u32(input, "frame")?;
    let (x, y) = (args::optional_u32(input, "x")?, args::optional_u32(input, "y")?);
    // A region (`x` or `y`) keeps what is under a `.` unless the call says
    // `skip_transparent: false`; a whole frame erases, as always.
    let region_form = x.is_some() || y.is_some();
    let explicit_skip = args::optional_bool(input, "skip_transparent")?;
    let skip_transparent = explicit_skip.unwrap_or(region_form);
    write(ctx, input, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        if let Some(frame) = frame {
            check_frame(sprite, frame)?;
        }
        let text = frame.map_or(text.clone(), |frame| with_frame_line(&text, frame));
        check_palette_line(project, &target, sprite, &text)?;
        let region = match (x, y) {
            (None, None) => None,
            (x, y) => Some((x.unwrap_or(0), y.unwrap_or(0))),
        };
        let plan = project
            .grid_plan(&target, &text, &GridOptions { region, skip_transparent })
            .map_err(|errors| grid_failure(errors, &valid_keys(project, sprite)))?;
        let (ops, remap) = (plan.ops, plan.remap);
        let (symmetry, only_over) = (
            symmetry_arg(input, sprite.width, sprite.height)?,
            only_over_arg(input, &valid_keys(project, sprite))?,
        );
        let ops = if symmetry.is_none() && only_over.is_none() {
            ops
        } else {
            let mut changed = Vec::new();
            for op in ops {
                match op {
                    Operation::SetPixels { sprite: s, layer, frame, pixels } => {
                        let current = &sprite.layers[0].frames[frame as usize - 1].pixels;
                        let wanted = pixels.iter().map(|p| (i64::from(p.x), i64::from(p.y), p.key)).collect();
                        let pixels = finish_pixels(current, sprite.width, sprite.height, wanted, symmetry, only_over)?;
                        if !pixels.is_empty() {
                            changed.push(Operation::SetPixels { sprite: s, layer, frame, pixels });
                        }
                    }
                    other => changed.push(other),
                }
            }
            changed
        };
        let (mut pixels, mut colors, mut busy) = (0, 0, None);
        for op in &ops {
            match op {
                Operation::SetPixels { pixels: set, frame, .. } => {
                    pixels += set.len();
                    busy.get_or_insert(*frame);
                }
                Operation::AddColor { .. } => colors += 1,
                _ => {}
            }
        }
        let mut extra = json!({ "pixels_changed": pixels, "colors_added": colors });
        if !remap.keys.is_empty() {
            // {"P": "Z"}: the legend key P conflicted, the pixels use Z.
            let moved: serde_json::Map<String, Value> =
                remap.keys.iter().map(|(from, to)| (from.to_string(), to.to_string().into())).collect();
            extra["remapped"] = moved.into();
            extra["warnings"] = remap.warnings.into();
        }
        let erased = super::write::count_erased(project, &ops);
        if region_form && explicit_skip == Some(false) && erased > 0 {
            let warning = format!("{erased} opaque pixel(s) were erased by a `.` of the region grid (`skip_transparent` is false). Send `skip_transparent` true to keep what is under a `.`");
            match extra["warnings"].as_array_mut() {
                Some(list) => list.push(warning.into()),
                None => extra["warnings"] = json!([warning]),
            }
        }
        let plan = Plan::new(ops, extra).preview();
        Ok(match busy {
            Some(frame) => plan.busy(&target, frame),
            None => plan,
        })
    })
}

/// Reads `x` and `y` of an object of the list of pixels.
fn coordinate(item: &Value, key: &str, index: usize) -> Result<u32, ToolError> {
    item[key]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| {
            ToolError::new("bad_argument", format!("pixel {index}: `{key}` must be a whole number, zero or more"))
                .with("index", index)
        })
}

fn set_pixels(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::optional_u32(input, "frame")?.unwrap_or(1);
    let list = input["pixels"]
        .as_array()
        .ok_or_else(|| ToolError::new("bad_argument", "`pixels` must be a list of {x, y, key}").with("argument", "pixels"))?;
    write(ctx, input, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let valid = valid_keys(project, sprite);
        let (symmetry, only_over) = (symmetry_arg(input, sprite.width, sprite.height)?, only_over_arg(input, &valid)?);
        let mut wanted = Vec::with_capacity(list.len());
        for (index, item) in list.iter().enumerate() {
            let (x, y) = (coordinate(item, "x", index)?, coordinate(item, "y", index)?);
            let text = item["key"].as_str().unwrap_or("");
            let key = key_of(text, &valid).map_err(|e| e.with("index", index).with("x", x).with("y", y))?;
            if x >= sprite.width || y >= sprite.height {
                return Err(out_of_range(index, x as i64, y as i64, sprite));
            }
            wanted.push((i64::from(x), i64::from(y), key));
        }
        let current = &sprite.layers[0].frames[frame as usize - 1].pixels;
        let pixels = finish_pixels(current, sprite.width, sprite.height, wanted, symmetry, only_over)?;
        let count = pixels.len();
        let ops = if pixels.is_empty() {
            Vec::new()
        } else {
            vec![Operation::SetPixels { sprite: target.clone(), layer: 0, frame, pixels }]
        };
        Ok(Plan::new(ops, json!({ "pixels_set": count })).preview().busy(&target, frame))
    })
}

fn out_of_range(index: usize, x: i64, y: i64, sprite: &Sprite) -> ToolError {
    ToolError::new(
        "out_of_range",
        format!(
            "pixel {index} at ({x}, {y}) is outside the {}x{} sprite; x goes 0 to {}, y goes 0 to {}",
            sprite.width, sprite.height, sprite.width - 1, sprite.height - 1
        ),
    )
    .with("index", index)
    .with("x", x)
    .with("y", y)
    .with("width", sprite.width)
    .with("height", sprite.height)
}

fn draw_shape(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::optional_u32(input, "frame")?.unwrap_or(1);
    let shape = args::required_str(input, "shape")?.to_string();
    let key_text = args::required_str(input, "key")?.to_string();
    let filled = args::optional_bool(input, "filled")?.unwrap_or(false);
    let number = |key: &str| args::required_u32(input, key).map(i64::from);
    write(ctx, input, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let valid = valid_keys(project, sprite);
        let key = key_of(&key_text, &valid)?;
        let (symmetry, only_over) = (symmetry_arg(input, sprite.width, sprite.height)?, only_over_arg(input, &valid)?);
        let (width, height) = (sprite.width as i64, sprite.height as i64);
        let area = match shape.as_str() {
            "line" => {
                let (x0, y0, x1, y1) = (number("x0")?, number("y0")?, number("x1")?, number("y1")?);
                inside(&[(x0, y0), (x1, y1)], sprite)?;
                shapes::line(x0, y0, x1, y1)
            }
            "rectangle" | "ellipse" => {
                let (x, y, w, h) = (number("x")?, number("y")?, number("width")?, number("height")?);
                if w == 0 || h == 0 {
                    return Err(ToolError::new("bad_argument", "`width` and `height` must be at least 1"));
                }
                inside(&[(x, y), (x + w - 1, y + h - 1)], sprite)?;
                if shape == "rectangle" { shapes::rectangle(x, y, w, h, filled) } else { shapes::ellipse(x, y, w, h, filled) }
            }
            "fill" => {
                let (x, y) = (number("x")?, number("y")?);
                inside(&[(x, y)], sprite)?;
                let pixels = &sprite.layers[0].frames[frame as usize - 1].pixels;
                shapes::flood(pixels, width as usize, height as usize, x as usize, y as usize)
            }
            other => {
                return Err(ToolError::new(
                    "bad_argument",
                    format!("`shape` is {other:?}; it must be line, rectangle, ellipse or fill"),
                )
                .with("argument", "shape"));
            }
        };
        let current = &sprite.layers[0].frames[frame as usize - 1].pixels;
        let wanted = area.iter().map(|&(x, y)| (x, y, key)).collect();
        let pixels = finish_pixels(current, sprite.width, sprite.height, wanted, symmetry, only_over)?;
        let count = pixels.len();
        let ops = if pixels.is_empty() {
            Vec::new()
        } else {
            vec![Operation::SetPixels { sprite: target.clone(), layer: 0, frame, pixels }]
        };
        Ok(Plan::new(ops, json!({ "pixels_changed": count })).preview().busy(&target, frame))
    })
}

/// The corners of a shape must be inside the sprite.
fn inside(points: &[(i64, i64)], sprite: &Sprite) -> Result<(), ToolError> {
    for (index, &(x, y)) in points.iter().enumerate() {
        if x < 0 || y < 0 || x >= sprite.width as i64 || y >= sprite.height as i64 {
            return Err(out_of_range(index, x, y, sprite));
        }
    }
    Ok(())
}
