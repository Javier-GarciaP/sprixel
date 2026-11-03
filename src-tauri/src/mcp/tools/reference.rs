//! `import_reference`: a PNG from the folder `references/` of the project is
//! written into a frame, with the colors of the sprite palette. The file is
//! the only thing outside the document that a tool reads, so the checks on its
//! path and size come first.

use super::args;
use super::create::base_revision;
use super::draw::preview_arg;
use super::read::check_frame;
use super::write::{Plan, write};
use crate::document::{Color, MAX_COLORS, Operation, PixelChange, TRANSPARENT};
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

/// The folder of the project where reference images live.
const FOLDER: &str = "references";
const MAX_BYTES: u64 = 2 * 1024 * 1024;
const MAX_SIDE: u32 = 512;

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "import_reference",
        description: concat!(
            "Draws a reference picture into a frame. The PNG must be inside the folder `references/` of the open project (ask the person to put it there); `file` is its path inside that folder, for example `goblin.png`. At most 512 pixels on a side and 2 MB. Every pixel gets the nearest color of the sprite palette; pixels with alpha under 128 become transparent. `width` and `height` scale the picture down first (nearest neighbor; give one and the other keeps the proportion). `max_colors` lets the call add up to that many new colors (the most used ones that the palette lacks) to the palette. The picture goes at `x`, `y` (default 0, 0) and must fit in the sprite.",
            write_note!()
        ),
        schema: json!({ "type": "object", "required": ["name", "file"], "properties": {
            "name": { "type": "string" },
            "folder": { "type": "string" },
            "frame": { "type": "integer", "minimum": 1, "description": "Default 1." },
            "file": { "type": "string", "description": "Path of the PNG inside `references/`, like \"goblin.png\"." },
            "width": { "type": "integer", "minimum": 1 },
            "height": { "type": "integer", "minimum": 1 },
            "max_colors": { "type": "integer", "minimum": 0, "description": "How many new palette colors the call may add. Default 0: only the colors of the palette." },
            "x": { "type": "integer", "minimum": 0 },
            "y": { "type": "integer", "minimum": 0 },
            "preview": preview_arg(),
            "base_revision": base_revision(),
        }}),
        need: Need::Edit,
        run: import_reference,
    }]
}

fn invalid(file: &str, why: &str) -> ToolError {
    ToolError::new("invalid_path", format!("`file` {file:?} is not allowed: {why}")).with("argument", "file")
}

/// The bytes of a PNG inside `<root>/references/`, after the checks: a
/// relative path with no `..`, no symbolic link on the way, the name ends with
/// `.png`, at most 2 MB and 512 pixels on a side.
fn read_png(root: &Path, file: &str) -> Result<Vec<u8>, ToolError> {
    if file.starts_with(['/', '\\']) || file.contains(':') {
        return Err(invalid(file, "use a path inside the folder references/, never an absolute one"));
    }
    let parts: Vec<&str> = file.split(['/', '\\']).filter(|p| !p.is_empty()).collect();
    if parts.is_empty() || parts.iter().any(|p| *p == "." || *p == "..") {
        return Err(invalid(file, "`.` and `..` are not allowed"));
    }
    if !file.to_ascii_lowercase().ends_with(".png") {
        return Err(invalid(file, "only .png files can be imported"));
    }
    let mut walk = root.to_path_buf();
    for part in std::iter::once(FOLDER).chain(parts.iter().copied()) {
        walk.push(part);
        let meta = std::fs::symlink_metadata(&walk).map_err(|_| {
            ToolError::new("not_found", format!("there is no file {file:?} in the folder {FOLDER}/ of the project"))
                .with("file", file)
        })?;
        if meta.file_type().is_symlink() {
            return Err(invalid(file, "symbolic links are not followed"));
        }
    }
    let mut path = root.join(FOLDER);
    path.extend(parts.iter());
    let meta = std::fs::metadata(&path).map_err(|e| invalid(file, &e.to_string()))?;
    if !meta.is_file() {
        return Err(invalid(file, "it is not a file"));
    }
    if meta.len() > MAX_BYTES {
        return Err(ToolError::new("too_big", format!("{file:?} is {} bytes; the limit is {MAX_BYTES} (2 MB)", meta.len()))
            .with("limit", MAX_BYTES));
    }
    let bytes = std::fs::read(&path).map_err(|e| invalid(file, &e.to_string()))?;
    let bad = |e: image::ImageError| ToolError::new("bad_image", format!("{file:?} is not a valid PNG: {e}")).with("file", file);
    let (w, h) = image::ImageReader::with_format(Cursor::new(&bytes), image::ImageFormat::Png).into_dimensions().map_err(bad)?;
    if w > MAX_SIDE || h > MAX_SIDE {
        return Err(ToolError::new("too_big", format!("{file:?} is {w}x{h}; the limit is {MAX_SIDE} pixels on a side"))
            .with("limit", MAX_SIDE));
    }
    Ok(bytes)
}

/// A picture of `width` x `height` pixels (RGBA, row by row), scaled to
/// `new_width` x `new_height` with nearest neighbor.
fn scale(rgba: &[u8], width: u32, height: u32, new_width: u32, new_height: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity((new_width * new_height * 4) as usize);
    for y in 0..new_height {
        for x in 0..new_width {
            let (sx, sy) = (x * width / new_width, y * height / new_height);
            let at = ((sy * width + sx) * 4) as usize;
            out.extend_from_slice(&rgba[at..at + 4]);
        }
    }
    out
}

fn rgb_of(hex: &str) -> [u8; 3] {
    let n = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0);
    [(n >> 16) as u8, (n >> 8) as u8, n as u8]
}

fn distance(a: [u8; 3], b: [u8; 3]) -> i64 {
    a.iter().zip(&b).map(|(&p, &q)| (i64::from(p) - i64::from(q)).pow(2)).sum()
}

/// The keys for every pixel of the picture, and the colors to add to the
/// palette first. `colors` are the palette colors; with `max_new` more than 0
/// the most used opaque colors that the palette does not have are added (up to
/// `max_new`, and never past 93 colors).
fn quantize(rgba: &[u8], colors: &[Color], max_new: usize) -> Result<(Vec<char>, Vec<Color>), ToolError> {
    let mut count: HashMap<[u8; 3], usize> = HashMap::new();
    for pixel in rgba.chunks(4).filter(|p| p[3] >= 128) {
        *count.entry([pixel[0], pixel[1], pixel[2]]).or_default() += 1;
    }
    let mut all: Vec<Color> = colors.to_vec();
    let mut added = Vec::new();
    let mut missing: Vec<([u8; 3], usize)> =
        count.iter().filter(|(rgb, _)| !all.iter().any(|c| rgb_of(&c.hex) == **rgb)).map(|(rgb, n)| (*rgb, *n)).collect();
    // The most used first; the color value breaks ties, so the result is always the same.
    missing.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (rgb, _) in missing.into_iter().take(max_new) {
        let letters = ('A'..='Z').chain('a'..='z').chain('0'..='9').chain(('!'..='~').filter(|c| !c.is_ascii_alphanumeric() && *c != TRANSPARENT));
        let key = letters.into_iter().find(|k| !all.iter().any(|c| c.key == *k));
        if all.len() >= MAX_COLORS || key.is_none() {
            return Err(ToolError::new("palette_full", format!("the palette is full ({MAX_COLORS} colors); lower `max_colors`")));
        }
        let hex = format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        let color = Color { key: key.unwrap(), name: format!("ref {}", &hex[1..]), hex };
        all.push(color.clone());
        added.push(color);
    }
    if all.is_empty() && count.values().any(|n| *n > 0) {
        return Err(ToolError::new("bad_argument", "the palette has no colors; pass `max_colors` to add some").with("argument", "max_colors"));
    }
    let table: Vec<([u8; 3], char)> = all.iter().map(|c| (rgb_of(&c.hex), c.key)).collect();
    let keys = rgba
        .chunks(4)
        .map(|p| {
            if p[3] < 128 {
                return TRANSPARENT;
            }
            let rgb = [p[0], p[1], p[2]];
            table.iter().min_by_key(|(c, _)| distance(*c, rgb)).map_or(TRANSPARENT, |(_, key)| *key)
        })
        .collect();
    Ok((keys, added))
}

fn import_reference(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::optional_u32(input, "frame")?.unwrap_or(1);
    let file = args::required_str(input, "file")?.to_string();
    let (want_w, want_h) = (args::optional_u32(input, "width")?, args::optional_u32(input, "height")?);
    let max_new = args::optional_u32(input, "max_colors")?.unwrap_or(0) as usize;
    let (x, y) = (args::optional_u32(input, "x")?.unwrap_or(0), args::optional_u32(input, "y")?.unwrap_or(0));
    if want_w == Some(0) || want_h == Some(0) {
        return Err(ToolError::new("bad_argument", "`width` and `height` must be at least 1"));
    }
    write(ctx, input, |session| {
        let project = &session.project;
        let target = args::sprite(project, input)?;
        let sprite = project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let palette = project.palette(&sprite.palette).ok_or_else(|| {
            ToolError::new("palette_not_found", format!("the palette {:?} of this sprite does not exist", sprite.palette))
        })?;
        let bytes = read_png(session.root(), &file)?;
        let decoded = image::ImageReader::with_format(Cursor::new(&bytes), image::ImageFormat::Png)
            .decode()
            .map_err(|e| ToolError::new("bad_image", format!("{file:?} is not a valid PNG: {e}")).with("file", file.clone()))?
            .to_rgba8();
        let (iw, ih) = decoded.dimensions();
        let (w, h) = match (want_w, want_h) {
            (None, None) => (iw, ih),
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => (w, (u64::from(ih) * u64::from(w) / u64::from(iw)).max(1) as u32),
            (None, Some(h)) => ((u64::from(iw) * u64::from(h) / u64::from(ih)).max(1) as u32, h),
        };
        if u64::from(x) + u64::from(w) > u64::from(sprite.width) || u64::from(y) + u64::from(h) > u64::from(sprite.height) {
            return Err(ToolError::new(
                "region_out",
                format!("the picture is {w}x{h} at ({x}, {y}) and does not fit in the {}x{} sprite; use `width` and `height` to scale it down", sprite.width, sprite.height),
            )
            .with("width", sprite.width)
            .with("height", sprite.height));
        }
        let scaled = if (w, h) == (iw, ih) { decoded.into_raw() } else { scale(decoded.as_raw(), iw, ih, w, h) };
        let (keys, added) = quantize(&scaled, &palette.colors, max_new)?;
        let current = &sprite.layers[0].frames[frame as usize - 1].pixels;
        let pixels: Vec<PixelChange> = keys
            .iter()
            .enumerate()
            .map(|(i, key)| PixelChange { x: x + i as u32 % w, y: y + i as u32 / w, key: *key })
            .filter(|p| current[(p.y * sprite.width + p.x) as usize] != p.key as u8)
            .collect();
        let (colors_added, pixel_count) = (added.len(), pixels.len());
        let mut ops: Vec<Operation> =
            added.into_iter().map(|color| Operation::AddColor { palette: palette.name.clone(), color, index: None }).collect();
        if !pixels.is_empty() {
            ops.push(Operation::SetPixels { sprite: target.clone(), layer: 0, frame, pixels });
        }
        Ok(Plan::new(ops, json!({ "pixels_changed": pixel_count, "colors_added": colors_added, "width": w, "height": h }))
            .preview()
            .busy(&target, frame))
    })
}
