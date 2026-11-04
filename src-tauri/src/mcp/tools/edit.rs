//! What `set_pixels`, `draw_shape` and `apply_grid` share: the arguments
//! `only_over` and `symmetry`, and the step that turns the wanted pixels into
//! the pixels that really change.

use super::args;
use crate::document::PixelChange;
use crate::mcp::ToolError;
use serde_json::{Value, json};
use std::collections::HashSet;

/// The JSON schema of `only_over`.
pub fn only_over_schema() -> Value {
    json!({ "type": "string", "description": "One palette key. Only pixels that have this key before the call change; the other pixels stay." })
}

/// The JSON schema of `symmetry` and `axis`, to put in a tool.
pub fn symmetry_schema() -> (Value, Value) {
    (
        json!({ "type": "string", "enum": ["horizontal", "vertical", "both"], "description": "Every pixel is also drawn mirrored. `horizontal` mirrors left to right, `vertical` top to bottom. The axis is the center of the sprite unless you give `axis`." }),
        json!({ "type": "object", "description": "With `symmetry`: the mirror line. `x` is the column of the vertical line and `y` the row of the horizontal one; whole or half numbers (3.5 is between columns 3 and 4).", "properties": {
            "x": { "type": "number", "minimum": 0 }, "y": { "type": "number", "minimum": 0 },
        }}),
    )
}

/// The mirror lines of a call. Axes are stored doubled, so half pixels work.
#[derive(Debug, Clone, Copy)]
pub struct Symmetry {
    horizontal: bool,
    vertical: bool,
    axis2_x: i64,
    axis2_y: i64,
}

/// Reads `symmetry` and `axis` for a sprite of this size.
pub fn symmetry_arg(input: &Value, width: u32, height: u32) -> Result<Option<Symmetry>, ToolError> {
    let Some(mode) = args::optional_str(input, "symmetry")? else {
        return Ok(None);
    };
    let (horizontal, vertical) = match mode {
        "horizontal" => (true, false),
        "vertical" => (false, true),
        "both" => (true, true),
        other => {
            return Err(ToolError::new(
                "bad_argument",
                format!("`symmetry` is {other:?}; it must be horizontal, vertical or both"),
            )
            .with("argument", "symmetry"));
        }
    };
    let axis = |name: &str, default2: i64| -> Result<i64, ToolError> {
        match &input["axis"][name] {
            Value::Null => Ok(default2),
            value => value
                .as_f64()
                .filter(|v| *v >= 0.0 && (v * 2.0).fract() == 0.0)
                .map(|v| (v * 2.0) as i64)
                .ok_or_else(|| {
                    ToolError::new("bad_argument", format!("`axis.{name}` must be a whole or half number, zero or more"))
                        .with("argument", "axis")
                }),
        }
    };
    Ok(Some(Symmetry {
        horizontal,
        vertical,
        axis2_x: axis("x", i64::from(width) - 1)?,
        axis2_y: axis("y", i64::from(height) - 1)?,
    }))
}

/// `only_over` as a key of the palette (`valid` lists them).
pub fn only_over_arg(input: &Value, valid: &[String]) -> Result<Option<char>, ToolError> {
    match args::optional_str(input, "only_over")? {
        None => Ok(None),
        Some(text) => super::draw::key_of(text, valid).map(Some),
    }
}

fn out_of_range(x: i64, y: i64, width: u32, height: u32) -> ToolError {
    ToolError::new(
        "out_of_range",
        format!("the mirrored pixel would be at ({x}, {y}), outside the {width}x{height} sprite; move the `axis` or the shape"),
    )
    .with("x", x)
    .with("y", y)
}

/// The pixels that really change. `wanted` are `(x, y, key)` in drawing
/// order; `current` is the frame. Steps: add the mirrored pixels, keep one
/// value per place (a pixel that was asked for wins over a mirrored one),
/// drop the places whose key is not `only_over`, drop the ones that already
/// have the key.
pub fn finish_pixels(
    current: &[u8],
    width: u32,
    height: u32,
    wanted: Vec<(i64, i64, char)>,
    symmetry: Option<Symmetry>,
    only_over: Option<char>,
) -> Result<Vec<PixelChange>, ToolError> {
    let mut all = Vec::with_capacity(wanted.len());
    let asked: HashSet<(i64, i64)> = wanted.iter().map(|&(x, y, _)| (x, y)).collect();
    for &(x, y, key) in &wanted {
        all.push((x, y, key));
        let Some(s) = symmetry else { continue };
        let (mx, my) = (s.axis2_x - x, s.axis2_y - y);
        let mut mirrors = Vec::new();
        if s.horizontal {
            mirrors.push((mx, y));
        }
        if s.vertical {
            mirrors.push((x, my));
        }
        if s.horizontal && s.vertical {
            mirrors.push((mx, my));
        }
        for (px, py) in mirrors {
            if px < 0 || py < 0 || px >= i64::from(width) || py >= i64::from(height) {
                return Err(out_of_range(px, py, width, height));
            }
            if !asked.contains(&(px, py)) {
                all.push((px, py, key));
            }
        }
    }
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for (x, y, key) in all {
        let now = current[(y * i64::from(width) + x) as usize];
        if !seen.insert((x, y)) || only_over.is_some_and(|k| now != k as u8) || now == key as u8 {
            continue;
        }
        out.push(PixelChange { x: x as u32, y: y as u32, key });
    }
    Ok(out)
}
