//! Region tools: `transform_region` and `copy_region`. The first part is the
//! pure math on a rectangle of pixels (`width` x `height` bytes, row by row).
//! The tools cut the rectangle out of a frame, call these functions and write
//! back only the pixels that changed, as one `SetPixels` (so history, undo,
//! replay and preview work as for every other write).

use super::args;
use super::create::base_revision;
use super::draw::{dry_run_arg, preview_arg};
use super::read::check_frame;
use super::write::{Plan, write};
use crate::document::{Operation, PixelChange, TRANSPARENT};
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};


pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "transform_region",
        description: concat!(
            "Changes a rectangle of one frame, or of many frames with `frames` (a list or a range like \"8-13\"), in one step, without rewriting pixels. `region` is {x, y, width, height} (default: the whole frame). `operation`: `flip_horizontal` (left and right swap), `flip_vertical`, `mirror` (needs `direction`: `left_to_right` copies the mirrored left half over the right half; also `right_to_left`, `top_to_bottom`, `bottom_to_top`), `move` (needs `dx` and `dy`; pixels that leave the region are lost unless `wrap` is true) and `rotate` (needs `degrees`: 90 clockwise, 180 or 270; 90 and 270 need a square region).",
            write_note!()
        ),
        schema: json!({ "type": "object", "required": ["name", "operation"], "properties": {
            "name": { "type": "string" },
            "folder": { "type": "string" },
            "frame": { "type": "integer", "minimum": 1, "description": "Default 1. Not with `frames`." },
            "frames": frames_schema("The frames to change, all with the same region and operation, as ONE step (one undo). Not with `frame`."),
            "region": region_schema("The rectangle to change. Default: the whole frame."),
            "operation": { "type": "string", "enum": ["flip_horizontal", "flip_vertical", "mirror", "move", "rotate"] },
            "direction": { "type": "string", "enum": ["left_to_right", "right_to_left", "top_to_bottom", "bottom_to_top"], "description": "For `mirror`." },
            "dx": { "type": "integer", "description": "For `move`: pixels to the right (negative: left)." },
            "dy": { "type": "integer", "description": "For `move`: pixels down (negative: up)." },
            "wrap": { "type": "boolean", "description": "For `move`: what leaves one side comes back on the other. Default false." },
            "degrees": { "type": "integer", "enum": [90, 180, 270], "description": "For `rotate`, clockwise." },
            "dry_run": dry_run_arg(),
                "preview": preview_arg(),
            "base_revision": base_revision(),
        }}),
        need: Need::Edit,
        run: transform_region,
    },
    Tool {
        name: "copy_region",
        description: concat!(
            "Copies a rectangle of a frame to a place in the same frame, another frame, many frames of the destination with `frames` (a list or a range like \"8-13\") or another sprite (it must use the same palette). `region` is the source {x, y, width, height} (default: the whole frame) of frame `frame` of sprite `name`. The copy goes to `to_sprite` (default: the same sprite), `to_frame` (default: `frame`), with its top left pixel at `to_x`, `to_y` (default: the same place as the source). With `skip_transparent` the `.` pixels of the source keep what is in the destination.",
            write_note!()
        ),
        schema: json!({ "type": "object", "required": ["name"], "properties": {
            "name": { "type": "string", "description": "The source sprite." },
            "folder": { "type": "string" },
            "frame": { "type": "integer", "minimum": 1, "description": "The source frame. Default 1." },
            "region": region_schema("The source rectangle. Default: the whole frame."),
            "to_sprite": { "type": "string", "description": "The destination sprite. Default: the source sprite." },
            "to_folder": { "type": "string" },
            "to_frame": { "type": "integer", "minimum": 1, "description": "Default: the source frame. Not with `frames`." },
            "frames": frames_schema("Paste on all these frames of the destination sprite (the source stays `frame`), as ONE step. Not with `to_frame`."),
            "to_x": { "type": "integer", "minimum": 0 },
            "to_y": { "type": "integer", "minimum": 0 },
            "skip_transparent": { "type": "boolean", "description": "A `.` of the source keeps the destination pixel. Default false." },
            "dry_run": dry_run_arg(),
                "preview": preview_arg(),
            "base_revision": base_revision(),
        }}),
        need: Need::Edit,
        run: copy_region,
    }]
}

/// The schema of a `frames` argument: a list or a range as text.
fn frames_schema(description: &str) -> Value {
    json!({ "type": ["array", "string"], "items": { "type": "integer", "minimum": 1 }, "description": format!("{description} A list like [8, 9] or a range as text like \"8-13\".") })
}

/// The schema of a `region` argument.
fn region_schema(description: &str) -> Value {
    json!({ "type": "object", "description": description, "required": ["x", "y", "width", "height"], "properties": {
        "x": { "type": "integer", "minimum": 0 }, "y": { "type": "integer", "minimum": 0 },
        "width": { "type": "integer", "minimum": 1 }, "height": { "type": "integer", "minimum": 1 },
    }})
}

pub(crate) type Rect = (u32, u32, u32, u32);

/// The pixels of a rectangle of a frame.
pub(crate) fn cut(pixels: &[u8], sprite_width: u32, (x, y, w, h): Rect) -> Vec<u8> {
    (y..y + h)
        .flat_map(|row| {
            let start = (row * sprite_width + x) as usize;
            pixels[start..start + w as usize].iter().copied()
        })
        .collect()
}

/// The pixels that change when `new` replaces the rectangle.
pub(crate) fn changes(current: &[u8], sprite_width: u32, (x, y, w, _): Rect, new: &[u8]) -> Vec<PixelChange> {
    let old = {
        let h = new.len() as u32 / w;
        cut(current, sprite_width, (x, y, w, h))
    };
    new.iter()
        .zip(&old)
        .enumerate()
        .filter(|(_, (n, o))| n != o)
        .map(|(i, (n, _))| PixelChange { x: x + i as u32 % w, y: y + i as u32 / w, key: *n as char })
        .collect()
}

/// A whole number that may be negative.
fn signed(input: &Value, key: &str) -> Result<i64, ToolError> {
    match &input[key] {
        Value::Null => Err(ToolError::new("bad_argument", format!("`{key}` must be given, as a whole number")).with("argument", key)),
        value => value
            .as_i64()
            .ok_or_else(|| ToolError::new("bad_argument", format!("`{key}` must be a whole number")).with("argument", key)),
    }
}

fn transform_region(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::optional_u32(input, "frame")?;
    let operation = args::required_str(input, "operation")?.to_string();
    let wrap = args::optional_bool(input, "wrap")?.unwrap_or(false);
    if frame.is_some() && !input["frames"].is_null() {
        return Err(ToolError::new("bad_argument", "use `frame` or `frames`, not both").with("argument", "frames"));
    }
    write(ctx, input, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        let frames = match args::frame_list(input, "frames", sprite.frame_count())? {
            Some(frames) => frames,
            None => {
                let frame = frame.unwrap_or(1);
                check_frame(sprite, frame)?;
                vec![frame]
            }
        };
        let (sw, sh) = (sprite.width, sprite.height);
        let rect = args::region(input, "region", sw, sh)?.unwrap_or((0, 0, sw, sh));
        let (w, h) = (rect.2 as usize, rect.3 as usize);
        // Every frame is worked out before anything is written, so one error
        // changes nothing.
        let mut ops = Vec::new();
        let (mut count, mut changed) = (0, Vec::new());
        for &frame in &frames {
            let current = &sprite.layers[0].frames[frame as usize - 1].pixels;
            let old = cut(current, sw, rect);
            let new = match operation.as_str() {
                "flip_horizontal" => flip_horizontal(&old, w, h),
                "flip_vertical" => flip_vertical(&old, w, h),
                "mirror" => {
                    let text = args::required_str(input, "direction")?;
                    let direction = Direction::parse(text).ok_or_else(|| {
                        ToolError::new(
                            "bad_argument",
                            format!("`direction` is {text:?}; it must be left_to_right, right_to_left, top_to_bottom or bottom_to_top"),
                        )
                        .with("argument", "direction")
                    })?;
                    mirror(&old, w, h, direction)
                }
                "move" => shift(&old, w, h, signed(input, "dx")?, signed(input, "dy")?, wrap),
                "rotate" => {
                    let degrees = args::required_u32(input, "degrees")?;
                    rotate(&old, w, h, degrees).ok_or_else(|| {
                        ToolError::new(
                            "bad_argument",
                            format!("cannot rotate a {w}x{h} region by {degrees}: use 90, 180 or 270, and a square region for 90 and 270"),
                        )
                        .with("argument", "degrees")
                    })?
                }
                other => {
                    return Err(ToolError::new(
                        "bad_argument",
                        format!("`operation` is {other:?}; it must be flip_horizontal, flip_vertical, mirror, move or rotate"),
                    )
                    .with("argument", "operation"));
                }
            };
            let pixels = changes(current, sw, rect, &new);
            if !pixels.is_empty() {
                count += pixels.len();
                changed.push(frame);
                ops.push(Operation::SetPixels { sprite: target.clone(), layer: 0, frame, pixels });
            }
        }
        let shown = changed.first().copied().unwrap_or(frames[0]);
        let mut extra = json!({ "pixels_changed": count });
        if frames.len() > 1 {
            extra["frames_changed"] = json!(changed);
        }
        Ok(Plan::new(ops, extra).preview().busy(&target, shown))
    })
}

fn copy_region(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::optional_u32(input, "frame")?.unwrap_or(1);
    let to_frame_arg = args::optional_u32(input, "to_frame")?;
    if to_frame_arg.is_some() && !input["frames"].is_null() {
        return Err(ToolError::new("bad_argument", "use `to_frame` or `frames`, not both").with("argument", "frames"));
    }
    let (to_x, to_y) = (args::optional_u32(input, "to_x")?, args::optional_u32(input, "to_y")?);
    let skip = args::optional_bool(input, "skip_transparent")?.unwrap_or(false);
    let to_sprite = args::optional_str(input, "to_sprite")?;
    write(ctx, input, |session| {
        let project = &session.project;
        let source = args::sprite(project, input)?;
        let from = project.find_sprite(&source).expect("just found");
        check_frame(from, frame)?;
        let target = match to_sprite {
            Some(name) => args::sprite(project, &json!({ "name": name, "folder": input["to_folder"] }))?,
            None => source.clone(),
        };
        let to = project.find_sprite(&target).expect("just found");
        let to_frames = match args::frame_list(input, "frames", to.frame_count())? {
            Some(frames) => frames,
            None => {
                let to_frame = to_frame_arg.unwrap_or(frame);
                check_frame(to, to_frame)?;
                vec![to_frame]
            }
        };
        if !crate::document::same_name(&from.palette, &to.palette) {
            return Err(ToolError::new(
                "palette_mismatch",
                format!("{:?} uses the palette {:?} and {:?} uses {:?}; a copy needs the same palette", from.name, from.palette, to.name, to.palette),
            )
            .with("from_palette", from.palette.clone())
            .with("to_palette", to.palette.clone()));
        }
        let (x, y, w, h) = args::region(input, "region", from.width, from.height)?.unwrap_or((0, 0, from.width, from.height));
        let (dx, dy) = (to_x.unwrap_or(x), to_y.unwrap_or(y));
        if u64::from(dx) + u64::from(w) > u64::from(to.width) || u64::from(dy) + u64::from(h) > u64::from(to.height) {
            return Err(ToolError::new(
                "region_out",
                format!("the copy {w}x{h} at ({dx}, {dy}) does not fit in the {}x{} sprite {:?}", to.width, to.height, to.name),
            )
            .with("width", to.width)
            .with("height", to.height));
        }
        let copy = cut(&from.layers[0].frames[frame as usize - 1].pixels, from.width, (x, y, w, h));
        let (mut ops, mut count, mut changed) = (Vec::new(), 0, Vec::new());
        for &to_frame in &to_frames {
            let current = &to.layers[0].frames[to_frame as usize - 1].pixels;
            let old = cut(current, to.width, (dx, dy, w, h));
            let pixels = changes(current, to.width, (dx, dy, w, h), &paste(&old, &copy, skip));
            if !pixels.is_empty() {
                count += pixels.len();
                changed.push(to_frame);
                ops.push(Operation::SetPixels { sprite: target.clone(), layer: 0, frame: to_frame, pixels });
            }
        }
        let shown = changed.first().copied().unwrap_or(to_frames[0]);
        let mut extra = json!({ "pixels_changed": count });
        if to_frames.len() > 1 {
            extra["frames_changed"] = json!(changed);
        }
        Ok(Plan::new(ops, extra).preview().busy(&target, shown))
    })
}

const BLANK: u8 = TRANSPARENT as u8;

/// Left and right swapped in every row.
pub fn flip_horizontal(buf: &[u8], width: usize, height: usize) -> Vec<u8> {
    (0..height).flat_map(|y| (0..width).map(move |x| (x, y))).map(|(x, y)| buf[y * width + (width - 1 - x)]).collect()
}

/// Top and bottom swapped.
pub fn flip_vertical(buf: &[u8], width: usize, height: usize) -> Vec<u8> {
    (0..height).flat_map(|y| (0..width).map(move |x| (x, y))).map(|(x, y)| buf[(height - 1 - y) * width + x]).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

impl Direction {
    pub fn parse(text: &str) -> Option<Direction> {
        match text {
            "left_to_right" => Some(Direction::LeftToRight),
            "right_to_left" => Some(Direction::RightToLeft),
            "top_to_bottom" => Some(Direction::TopToBottom),
            "bottom_to_top" => Some(Direction::BottomToTop),
            _ => None,
        }
    }
}

/// The mirrored copy of one half replaces the other half. With an odd size
/// the middle row or column stays as it is.
pub fn mirror(buf: &[u8], width: usize, height: usize, direction: Direction) -> Vec<u8> {
    let mut out = buf.to_vec();
    for y in 0..height {
        for x in 0..width {
            let (from_x, from_y) = match direction {
                Direction::LeftToRight if x >= width.div_ceil(2) => (width - 1 - x, y),
                Direction::RightToLeft if x < width / 2 => (width - 1 - x, y),
                Direction::TopToBottom if y >= height.div_ceil(2) => (x, height - 1 - y),
                Direction::BottomToTop if y < height / 2 => (x, height - 1 - y),
                _ => continue,
            };
            out[y * width + x] = buf[from_y * width + from_x];
        }
    }
    out
}

/// The content moved by `dx` and `dy` inside the rectangle. Without `wrap`
/// what leaves is lost and the free places are transparent; with `wrap` it
/// comes back on the other side.
pub fn shift(buf: &[u8], width: usize, height: usize, dx: i64, dy: i64, wrap: bool) -> Vec<u8> {
    let (w, h) = (width as i64, height as i64);
    let mut out = vec![BLANK; buf.len()];
    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = (x - dx, y - dy);
            let (sx, sy) = if wrap { (sx.rem_euclid(w), sy.rem_euclid(h)) } else { (sx, sy) };
            if (0..w).contains(&sx) && (0..h).contains(&sy) {
                out[(y * w + x) as usize] = buf[(sy * w + sx) as usize];
            }
        }
    }
    out
}

/// Turns the rectangle clockwise by 90, 180 or 270 degrees. 90 and 270 need a
/// square (`None` otherwise, and for any other angle).
pub fn rotate(buf: &[u8], width: usize, height: usize, degrees: u32) -> Option<Vec<u8>> {
    match degrees {
        180 => Some(buf.iter().rev().copied().collect()),
        90 | 270 if width == height => {
            let n = width;
            Some(
                (0..n)
                    .flat_map(|y| (0..n).map(move |x| (x, y)))
                    .map(|(x, y)| if degrees == 90 { buf[(n - 1 - x) * n + y] } else { buf[x * n + (n - 1 - y)] })
                    .collect(),
            )
        }
        _ => None,
    }
}

/// `source` painted over `target` (both `width` x `height`). With
/// `skip_transparent` a `.` of the source keeps the target pixel.
pub fn paste(target: &[u8], source: &[u8], skip_transparent: bool) -> Vec<u8> {
    target.iter().zip(source).map(|(&t, &s)| if skip_transparent && s == BLANK { t } else { s }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn show(buf: &[u8], width: usize) -> String {
        buf.chunks(width).map(|r| String::from_utf8_lossy(r).into_owned()).collect::<Vec<_>>().join("/")
    }

    // Checks both flips on a rectangle that is not square.
    #[test]
    fn flips_reverse_rows_or_columns() {
        let buf = b"RG.R..G.....";
        assert_eq!(show(&flip_horizontal(buf, 4, 3), 4), "R.GR/.G../....");
        assert_eq!(show(&flip_vertical(buf, 4, 3), 4), "..../..G./RG.R");
    }

    // Checks the four mirror directions, with an odd width where the middle
    // column must stay.
    #[test]
    fn mirror_copies_one_half_over_the_other() {
        let buf = b"RG..";
        assert_eq!(show(&mirror(buf, 4, 1, Direction::LeftToRight), 4), "RGGR");
        assert_eq!(show(&mirror(b"..GR", 4, 1, Direction::RightToLeft), 4), "RGGR");
        assert_eq!(show(&mirror(b"RGB..", 5, 1, Direction::LeftToRight), 5), "RGBGR");
    }

    // Checks the vertical mirrors in a clear way (a column of 4 pixels).
    #[test]
    fn mirror_works_up_and_down() {
        assert_eq!(show(&mirror(b"RG..", 1, 4, Direction::TopToBottom), 1), "R/G/G/R");
        assert_eq!(show(&mirror(b"..GR", 1, 4, Direction::BottomToTop), 1), "R/G/G/R");
    }

    // Checks the move: without wrap the pixels that leave are lost and the free
    // places are `.`; with wrap they return on the other side; negative moves.
    #[test]
    fn shift_moves_and_wraps() {
        let buf = b"RG..";
        assert_eq!(show(&shift(buf, 4, 1, 2, 0, false), 4), "..RG");
        assert_eq!(show(&shift(buf, 4, 1, 3, 0, false), 4), "...R");
        assert_eq!(show(&shift(buf, 4, 1, 3, 0, true), 4), "G..R");
        assert_eq!(show(&shift(buf, 4, 1, -1, 0, true), 4), "G..R");
    }

    // Checks the vertical move with a column.
    #[test]
    fn shift_works_up_and_down() {
        assert_eq!(show(&shift(b"RG..", 1, 4, 0, 1, false), 1), "./R/G/.");
        assert_eq!(show(&shift(b"RG..", 1, 4, 0, -1, true), 1), "G/././R");
    }

    // Checks the rotations on a 2x2 square (clockwise), 180 on a rectangle, and
    // the refusal of a rectangle for 90 and 270.
    #[test]
    fn rotate_turns_clockwise() {
        let buf = b"RG..";
        assert_eq!(show(&rotate(buf, 2, 2, 90).unwrap(), 2), ".R/.G");
        assert_eq!(show(&rotate(buf, 2, 2, 180).unwrap(), 2), "../GR");
        assert_eq!(show(&rotate(buf, 2, 2, 270).unwrap(), 2), "G./R.");
        assert_eq!(show(&rotate(b"RG.", 3, 1, 180).unwrap(), 3), ".GR");
        assert!(rotate(b"RG.", 3, 1, 90).is_none());
        assert!(rotate(buf, 2, 2, 45).is_none());
        let mut back = buf.to_vec();
        for _ in 0..4 {
            back = rotate(&back, 2, 2, 90).unwrap();
        }
        assert_eq!(back, buf, "four quarter turns give the same picture");
    }

    // Checks the paste: `.` erases, unless `skip_transparent`.
    #[test]
    fn paste_can_skip_transparent() {
        assert_eq!(paste(b"RRRR", b"G.G.", false), b"G.G.");
        assert_eq!(paste(b"RRRR", b"G.G.", true), b"GRGR");
    }
}
