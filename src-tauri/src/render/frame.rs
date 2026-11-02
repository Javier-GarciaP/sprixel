use crate::document::{Palette, Sprite, TRANSPARENT};

/// Straight (not premultiplied) RGBA pixels, row by row.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// The RGBA of a palette key. A key the palette does not have, like `.`, is
/// fully transparent.
fn color_of(palette: &Palette, key: u8) -> [u8; 4] {
    if key as char == TRANSPARENT {
        return [0, 0, 0, 0];
    }
    let Some(color) = palette.colors.iter().find(|c| c.key == key as char) else {
        return [0, 0, 0, 0];
    };
    let channel = |at: usize| u8::from_str_radix(&color.hex[at..at + 2], 16).unwrap_or(0);
    [channel(1), channel(3), channel(5), 255]
}

/// Draws one frame (counted from 0) with all layers stacked, the last layer
/// on top, and scales it by `scale` with nearest-neighbor.
pub fn render_frame(sprite: &Sprite, palette: &Palette, frame: usize, scale: u32) -> Image {
    let (w, h, s) = (sprite.width as usize, sprite.height as usize, scale.max(1) as usize);
    let mut flat = vec![[0u8; 4]; w * h];
    for layer in &sprite.layers {
        for (at, key) in layer.frames[frame].pixels.iter().enumerate() {
            if *key as char != TRANSPARENT {
                flat[at] = color_of(palette, *key);
            }
        }
    }
    let mut rgba = Vec::with_capacity(w * s * h * s * 4);
    for y in 0..h * s {
        for x in 0..w * s {
            rgba.extend_from_slice(&flat[(y / s) * w + x / s]);
        }
    }
    Image { width: (w * s) as u32, height: (h * s) as u32, rgba }
}
