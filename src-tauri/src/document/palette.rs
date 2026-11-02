//! The rules for the colors of a palette: keys, names, values and the limit.

use super::error::DocError;
use super::model::{Color, MAX_COLORS, Palette, TRANSPARENT};

impl Palette {
    /// Checks a color that is added to this palette, or that replaces the
    /// color with the key `own` (the one being edited). The other colors are
    /// the ones that its key and name must not repeat.
    pub fn check_color(&self, own: Option<char>, color: &Color) -> Result<(), DocError> {
        // The limit comes first: a full palette has no free key left anyway.
        if own.is_none() && self.colors.len() >= MAX_COLORS {
            return Err(DocError::new(
                "palette_full",
                format!("a palette has at most {MAX_COLORS} colors"),
            ));
        }
        if color.key == TRANSPARENT || color.key == ' ' {
            return Err(DocError::new(
                "reserved_key",
                "\".\" means transparent and the space is not a key, choose another key",
            ));
        }
        if !color.key.is_ascii_graphic() {
            return Err(DocError::new(
                "bad_key",
                "a key is one printable ASCII character",
            ));
        }
        if color.name.trim().is_empty() {
            return Err(DocError::new("bad_name", "a color needs a name"));
        }
        if !is_hex(&color.hex) {
            return Err(DocError::new(
                "bad_color",
                format!("{:?} is not a color, expected #RRGGBB", color.hex),
            ));
        }
        let others = || self.colors.iter().filter(|c| Some(c.key) != own);
        if others().any(|c| c.key == color.key) {
            return Err(DocError::new(
                "duplicate_key",
                format!("the key {:?} is already used by another color", color.key),
            ));
        }
        if others().any(|c| c.name.to_lowercase() == color.name.trim().to_lowercase()) {
            return Err(DocError::new(
                "duplicate_name",
                format!("the name {:?} is already used by another color", color.name),
            ));
        }
        Ok(())
    }
}

/// Checks a whole list of colors, as the editor sends it on Done: each color
/// is checked against the ones before it.
pub fn check_colors(colors: &[Color]) -> Result<(), DocError> {
    let mut growing = Palette {
        name: String::new(),
        colors: Vec::new(),
    };
    for color in colors {
        growing.check_color(None, color)?;
        growing.colors.push(color.clone());
    }
    Ok(())
}

fn is_hex(hex: &str) -> bool {
    hex.len() == 7 && hex.starts_with('#') && hex[1..].bytes().all(|b| b.is_ascii_hexdigit())
}
