//! The operations of one user stroke: the pixels that changed, plus the new
//! colors they need in the palette. Both go in one history entry, so one Undo
//! takes back the pixels and the colors together.

use super::error::DocError;
use super::model::{Color, Project, TRANSPARENT};
use super::operation::{Operation, PixelChange, SpriteRef};

/// One pixel of a stroke. `color` is `#RRGGBB`, or `None` for transparent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorChange {
    pub x: u32,
    pub y: u32,
    pub color: Option<String>,
}

impl Project {
    /// Builds the operations for a stroke on one frame of a sprite. A color
    /// that is not in the palette of the sprite is added with the next free
    /// key; its name is its hex value. Nothing is applied here: when the
    /// palette has no room for a new color the error is `palette_full` and
    /// the project stays as it is.
    pub fn stroke_operations(
        &self,
        sprite: &SpriteRef,
        layer: usize,
        frame: u32,
        changes: &[ColorChange],
    ) -> Result<Vec<Operation>, DocError> {
        let target = self
            .find_sprite(sprite)
            .ok_or_else(|| DocError::new("not_found", format!("sprite {:?} does not exist", sprite.name)))?;
        let palette = self.palette(&target.palette).ok_or_else(|| {
            DocError::new(
                "palette_not_found",
                format!("palette {:?} does not exist", target.palette),
            )
        })?;
        // The palette as it will be after the new colors, to pick the keys.
        let mut growing = palette.clone();
        let mut ops = Vec::new();
        let mut pixels = Vec::with_capacity(changes.len());
        for change in changes {
            let key = match &change.color {
                None => TRANSPARENT,
                Some(hex) => {
                    let hex = normalize_hex(hex)?;
                    match growing.key_of_hex(&hex) {
                        Some(key) => key,
                        None => {
                            let key = growing.next_free_key().ok_or_else(|| {
                                DocError::new("palette_full", "The palette is full")
                            })?;
                            let color = Color {
                                key,
                                name: hex[1..].to_string(),
                                hex,
                            };
                            ops.push(Operation::AddColor {
                                palette: palette.name.clone(),
                                color: color.clone(),
                                index: None,
                            });
                            growing.colors.push(color);
                            key
                        }
                    }
                }
            };
            pixels.push(PixelChange {
                x: change.x,
                y: change.y,
                key,
            });
        }
        ops.push(Operation::SetPixels {
            sprite: sprite.clone(),
            layer,
            frame,
            pixels,
        });
        Ok(ops)
    }
}

// `#rrggbb` or `#RRGGBB` becomes `#RRGGBB`.
fn normalize_hex(hex: &str) -> Result<String, DocError> {
    let valid = hex.len() == 7
        && hex.starts_with('#')
        && hex[1..].bytes().all(|b| b.is_ascii_hexdigit());
    if valid {
        Ok(hex.to_ascii_uppercase())
    } else {
        Err(DocError::new(
            "bad_color",
            format!("{hex:?} is not a color, expected #RRGGBB"),
        ))
    }
}
