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

/// The digits 0 to 9 in a 3x5 font: 5 rows, 3 bits each (the left bit first).
const DIGITS: [[u8; 5]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111],
    [0b010, 0b110, 0b010, 0b010, 0b111],
    [0b111, 0b001, 0b111, 0b100, 0b111],
    [0b111, 0b001, 0b111, 0b001, 0b111],
    [0b101, 0b101, 0b111, 0b001, 0b001],
    [0b111, 0b100, 0b111, 0b001, 0b111],
    [0b111, 0b100, 0b111, 0b101, 0b111],
    [0b111, 0b001, 0b001, 0b001, 0b001],
    [0b111, 0b101, 0b111, 0b101, 0b111],
    [0b111, 0b101, 0b111, 0b001, 0b111],
];

/// The height of the band above a labeled frame, in label pixels: one of
/// space, five of digit and one of space.
pub const LABEL_BAND: u32 = 7;

/// The pixels of a number: its width, its height and the (x, y) of every
/// lit pixel. Each font pixel is `lu` x `lu`, digits are one font pixel apart.
pub(super) fn draw_number(number: u32, lu: u32) -> (u32, u32, Vec<(u32, u32)>) {
    let text = number.to_string();
    let mut lit = Vec::new();
    for (at, digit) in text.bytes().enumerate() {
        let rows = DIGITS[(digit - b'0') as usize];
        let left = at as u32 * 4;
        for (y, row) in rows.iter().enumerate() {
            for x in 0..3u32 {
                if row >> (2 - x) & 1 == 1 {
                    for (dx, dy) in (0..lu).flat_map(|dx| (0..lu).map(move |dy| (dx, dy))) {
                        lit.push(((left + x) * lu + dx, y as u32 * lu + dy));
                    }
                }
            }
        }
    }
    let chars = text.len() as u32;
    ((chars * 4 - 1) * lu, 5 * lu, lit)
}

/// All `frames` (same size) in a grid of `columns`, in reading order, over
/// the background. Cells are `unit` pixels apart (one sprite pixel). With
/// `labels`, a band above each frame shows its number (from 1).
pub fn render_sheet(frames: &[Image], columns: u32, unit: u32, labels: bool, background: Background) -> Image {
    let unit = unit.max(1);
    let (fw, fh) = frames.first().map_or((0, 0), |f| (f.width, f.height));
    let columns = columns.clamp(1, frames.len().max(1) as u32);
    let rows = (frames.len() as u32).div_ceil(columns);
    let lu = (unit / 2).max(1);
    let band = if labels { LABEL_BAND * lu } else { 0 };
    let (width, height) = (columns * fw + (columns - 1) * unit, rows * (fh + band) + rows.saturating_sub(1) * unit);
    let mut sheet = Image { width, height, rgba: vec![0; (width * height * 4) as usize] };
    let ink = if background == Background::Dark { [255, 255, 255, 255] } else { [0, 0, 0, 255] };
    let mut put = |x: u32, y: u32, color: [u8; 4]| {
        if x < width && y < height {
            let at = ((y * width + x) * 4) as usize;
            sheet.rgba[at..at + 4].copy_from_slice(&color);
        }
    };
    for (index, frame) in frames.iter().enumerate() {
        let (col, row) = (index as u32 % columns, index as u32 / columns);
        let (left, top) = (col * (fw + unit), row * (fh + band + unit));
        for (at, px) in frame.rgba.chunks_exact(4).enumerate() {
            let (x, y) = (at as u32 % frame.width, at as u32 / frame.width);
            put(left + x, top + band + y, [px[0], px[1], px[2], px[3]]);
        }
        if labels {
            let (_, _, lit) = draw_number(index as u32 + 1, lu);
            for (x, y) in lit {
                put(left + x, top + lu + y, ink);
            }
        }
    }
    flatten(&sheet, background, checker_cell(unit))
}

/// The size of `render_sheet` for `count` frames of `w` x `h` (before scale)
/// at `scale`, without drawing it.
pub fn sheet_size(w: u32, h: u32, count: u32, columns: u32, scale: u32, labels: bool) -> (u32, u32) {
    let columns = columns.clamp(1, count.max(1));
    let rows = count.div_ceil(columns);
    let band = if labels { LABEL_BAND * (scale / 2).max(1) } else { 0 };
    (columns * w * scale + (columns - 1) * scale, rows * (h * scale + band) + rows.saturating_sub(1) * scale)
}

/// Frames (same size, oldest first) drawn over each other. The last one is
/// normal and opaque. Each earlier one is tinted half way to #FF4040 and
/// translucent, the oldest the faintest (alpha 90 to 190). The newest frame
/// wins where two have a pixel.
pub fn onion(frames: &[Image]) -> Image {
    let (width, height) = frames.first().map_or((0, 0), |f| (f.width, f.height));
    let mut out = Image { width, height, rgba: vec![0; (width * height * 4) as usize] };
    let earlier = frames.len().saturating_sub(1);
    for (index, frame) in frames.iter().enumerate() {
        let alpha = if index == earlier {
            255
        } else if earlier == 1 {
            190
        } else {
            (90 + index * 100 / (earlier - 1)) as u8
        };
        for (at, px) in frame.rgba.chunks_exact(4).enumerate() {
            if px[3] == 0 {
                continue;
            }
            let color = if index == earlier {
                [px[0], px[1], px[2], 255]
            } else {
                [((px[0] as u16 + 255) / 2) as u8, ((px[1] as u16 + 64) / 2) as u8, ((px[2] as u16 + 64) / 2) as u8, alpha]
            };
            out.rgba[at * 4..at * 4 + 4].copy_from_slice(&color);
        }
    }
    out
}
