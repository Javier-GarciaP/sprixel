//! The pictures an AI asks for to see its work: a frame over a background, a
//! sheet of frames and an onion skin. They are made from `render_frame`, which
//! keeps transparent pixels transparent (the export needs that).

use super::frame::Image;

/// What is drawn behind a preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Background {
    /// A neutral gray checkerboard, like the editor. The default.
    Checker,
    Light,
    Dark,
    /// No background: transparent pixels keep alpha 0.
    Transparent,
}

/// The same two grays as the editor canvas.
const CHECKER: [[u8; 3]; 2] = [[180, 180, 180], [156, 156, 156]];
const LIGHT: [u8; 3] = [255, 255, 255];
const DARK: [u8; 3] = [0x20, 0x20, 0x20];

impl Background {
    /// The values the `background` argument accepts, for messages.
    pub const NAMES: &'static str = "checker, light, dark, transparent";

    pub fn parse(name: &str) -> Option<Background> {
        Some(match name {
            "checker" => Background::Checker,
            "light" => Background::Light,
            "dark" => Background::Dark,
            "transparent" => Background::Transparent,
            _ => return None,
        })
    }

    /// The opaque color behind the pixel (x, y), or `None` for no background.
    fn at(self, x: u32, y: u32, cell: u32) -> Option<[u8; 3]> {
        match self {
            Background::Checker => Some(CHECKER[((x / cell + y / cell) & 1) as usize]),
            Background::Light => Some(LIGHT),
            Background::Dark => Some(DARK),
            Background::Transparent => None,
        }
    }
}

/// Puts `src` over `dst` ("over" blend, straight alpha).
pub(super) fn blend(dst: [u8; 4], src: [u8; 4]) -> [u8; 4] {
    let (sa, da) = (src[3] as u32, dst[3] as u32);
    if sa == 255 || da == 0 {
        return if sa == 0 { dst } else { src };
    }
    if sa == 0 {
        return dst;
    }
    // Alpha scaled to 0..=255 * 255 to keep whole numbers.
    let behind = da * (255 - sa);
    let out_a = sa * 255 + behind;
    let mix = |s: u8, d: u8| ((s as u32 * sa * 255 + d as u32 * behind) / out_a) as u8;
    [mix(src[0], dst[0]), mix(src[1], dst[1]), mix(src[2], dst[2]), (out_a / 255) as u8]
}

/// The image over the background. `cell` is the side of a checker square, in
/// pixels. With `Transparent` the image is returned as it is.
pub fn flatten(image: &Image, background: Background, cell: u32) -> Image {
    let cell = cell.max(1);
    let mut rgba = Vec::with_capacity(image.rgba.len());
    for (at, src) in image.rgba.chunks_exact(4).enumerate() {
        let (x, y) = (at as u32 % image.width, at as u32 / image.width);
        let src = [src[0], src[1], src[2], src[3]];
        let out = match background.at(x, y, cell) {
            Some([r, g, b]) => blend([r, g, b, 255], src),
            None => src,
        };
        rgba.extend_from_slice(&out);
    }
    Image { width: image.width, height: image.height, rgba }
}

/// The side of a checker square for a preview at `scale`: half a sprite
/// pixel, so the squares are not mistaken for sprite pixels.
pub fn checker_cell(scale: u32) -> u32 {
    (scale / 2).max(1)
}
