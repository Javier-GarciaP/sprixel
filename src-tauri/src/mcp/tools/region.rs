//! Region tools: `transform_region` and `copy_region`. This part is the pure
//! math on a rectangle of pixels (`width` x `height` bytes, row by row). The
//! tools below it cut the rectangle out of a frame, call these functions and
//! write back only the pixels that changed.

use crate::document::TRANSPARENT;

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
