//! The pixels of a shape. Plain functions: they only compute coordinates, so
//! they are easy to test. A shape is a list of `(x, y)`; the tool decides the
//! key and checks the pixels fit the sprite.

use std::collections::HashSet;

/// A line from `(x0, y0)` to `(x1, y1)`, both ends included (Bresenham).
pub fn line(x0: i64, y0: i64, x1: i64, y1: i64) -> Vec<(i64, i64)> {
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let (mut x, mut y, mut error) = (x0, y0, dx + dy);
    let mut pixels = vec![(x, y)];
    while (x, y) != (x1, y1) {
        let double = 2 * error;
        if double >= dy {
            error += dy;
            x += sx;
        }
        if double <= dx {
            error += dx;
            y += sy;
        }
        pixels.push((x, y));
    }
    pixels
}

/// A rectangle with its top left pixel at `(x, y)`, `width` x `height` pixels.
pub fn rectangle(x: i64, y: i64, width: i64, height: i64, filled: bool) -> Vec<(i64, i64)> {
    let mut pixels = Vec::new();
    for py in y..y + height {
        for px in x..x + width {
            let border = px == x || px == x + width - 1 || py == y || py == y + height - 1;
            if filled || border {
                pixels.push((px, py));
            }
        }
    }
    pixels
}

/// An ellipse that fills the box with top left `(x, y)` and the given size.
/// A pixel is inside when its center is inside the ellipse. The outline is
/// the inside pixels that touch (up, down, left or right) a pixel outside.
pub fn ellipse(x: i64, y: i64, width: i64, height: i64, filled: bool) -> Vec<(i64, i64)> {
    let (rx, ry) = (width as f64 / 2.0, height as f64 / 2.0);
    let (cx, cy) = (x as f64 + rx, y as f64 + ry);
    let inside = |px: i64, py: i64| {
        let (dx, dy) = ((px as f64 + 0.5 - cx) / rx, (py as f64 + 0.5 - cy) / ry);
        dx * dx + dy * dy <= 1.0
    };
    let mut pixels = Vec::new();
    for py in y..y + height {
        for px in x..x + width {
            if !inside(px, py) {
                continue;
            }
            let edge = !inside(px - 1, py) || !inside(px + 1, py) || !inside(px, py - 1) || !inside(px, py + 1);
            if filled || edge {
                pixels.push((px, py));
            }
        }
    }
    pixels
}

/// The pixels a flood fill from `(x, y)` would change: the connected area
/// (up, down, left, right) that has the same key as the start pixel.
/// `pixels` are the keys of the frame, row by row.
pub fn flood(pixels: &[u8], width: usize, height: usize, x: usize, y: usize) -> Vec<(i64, i64)> {
    let target = pixels[y * width + x];
    let mut seen = HashSet::new();
    let mut stack = vec![(x, y)];
    let mut area = Vec::new();
    while let Some((px, py)) = stack.pop() {
        if !seen.insert((px, py)) || pixels[py * width + px] != target {
            continue;
        }
        area.push((px as i64, py as i64));
        if px > 0 {
            stack.push((px - 1, py));
        }
        if px + 1 < width {
            stack.push((px + 1, py));
        }
        if py > 0 {
            stack.push((px, py - 1));
        }
        if py + 1 < height {
            stack.push((px, py + 1));
        }
    }
    area.sort_by_key(|&(px, py)| (py, px));
    area
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Draws pixels as rows of `#` and `.` to compare them by eye.
    fn show(pixels: &[(i64, i64)], width: i64, height: i64) -> String {
        (0..height)
            .map(|y| (0..width).map(|x| if pixels.contains(&(x, y)) { '#' } else { '.' }).collect::<String>())
            .collect::<Vec<_>>()
            .join("/")
    }

    // Checks lines: horizontal, vertical, diagonal and in any direction, with
    // both ends included.
    #[test]
    fn lines_include_both_ends() {
        assert_eq!(show(&line(0, 0, 3, 0), 4, 1), "####");
        assert_eq!(show(&line(1, 0, 1, 2), 3, 3), ".#./.#./.#.");
        assert_eq!(show(&line(0, 0, 3, 3), 4, 4), "#.../.#../..#./...#");
        assert_eq!(show(&line(3, 3, 0, 0), 4, 4), "#.../.#../..#./...#");
        assert_eq!(line(2, 2, 2, 2), vec![(2, 2)]);
        assert_eq!(line(0, 0, 4, 2).len(), 5);
    }

    // Checks the rectangle: the outline and the filled version.
    #[test]
    fn rectangles_are_outlined_or_filled() {
        assert_eq!(show(&rectangle(1, 1, 3, 3, false), 5, 5), "...../.###./.#.#./.###./.....");
        assert_eq!(show(&rectangle(0, 0, 2, 2, true), 3, 3), "##./##./...");
        assert_eq!(show(&rectangle(0, 0, 1, 1, false), 1, 1), "#");
        assert_eq!(rectangle(0, 0, 4, 1, false).len(), 4);
    }

    // Checks the ellipse: a filled circle in a 5x5 box, its outline, and the
    // degenerate boxes of one pixel.
    #[test]
    fn ellipses_fill_the_box() {
        assert_eq!(show(&ellipse(0, 0, 5, 5, true), 5, 5), ".###./#####/#####/#####/.###.");
        assert_eq!(show(&ellipse(0, 0, 5, 5, false), 5, 5), ".###./#...#/#...#/#...#/.###.");
        assert_eq!(show(&ellipse(0, 0, 1, 1, true), 1, 1), "#");
        assert_eq!(show(&ellipse(0, 0, 4, 2, true), 4, 2), "####/####");
    }

    // Checks the flood fill: it stays inside the connected area and does not
    // cross a different key, and diagonal neighbors are not connected.
    #[test]
    fn flood_fills_the_connected_area() {
        // . . R
        // . R .
        // R . .
        let frame = b"..R.R.R..";
        let area = flood(frame, 3, 3, 0, 0);
        assert_eq!(area, vec![(0, 0), (1, 0), (0, 1)]);
        assert_eq!(flood(frame, 3, 3, 2, 0), vec![(2, 0)], "diagonal R pixels are separate");
        assert_eq!(flood(b"....", 2, 2, 1, 1).len(), 4);
    }
}
