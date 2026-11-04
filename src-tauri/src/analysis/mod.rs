//! Pure functions that outline, shade and check a frame. They know nothing of
//! Tauri or MCP: a frame goes in (`View`), a list of pixel changes or a report
//! comes out, and the same input always gives the same output. The MCP tools in
//! `mcp/tools/analysis.rs` are thin wrappers. The exact rules are in
//! `openspec/changes/agent-analysis-tools/design.md`.

mod outline;
mod report;
mod shade;

pub use outline::{OutlineOptions, auto_outline, thin_regions};
pub use report::{FrameChange, colors, compare, loose_pixels, outline_gaps};
pub use shade::{Light, Region, ShadeOptions, auto_shade, isolated_changes};

use crate::document::TRANSPARENT;

/// The pixels of one frame: one palette key (an ASCII byte) per pixel, row by
/// row. `.` is transparent, any other key is opaque.
#[derive(Clone, Copy)]
pub struct View<'a> {
    pixels: &'a [u8],
    pub width: usize,
    pub height: usize,
}

impl<'a> View<'a> {
    pub fn new(pixels: &'a [u8], width: usize, height: usize) -> View<'a> {
        assert_eq!(pixels.len(), width * height, "a frame has width x height pixels");
        View { pixels, width, height }
    }

    /// The key at (x, y), or `None` outside the canvas.
    pub fn get(&self, x: i64, y: i64) -> Option<u8> {
        let inside = x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height;
        inside.then(|| self.pixels[y as usize * self.width + x as usize])
    }

    /// True for a pixel with a key other than `.`. Outside the canvas there are
    /// no pixels, so it is false.
    pub fn is_opaque(&self, x: i64, y: i64) -> bool {
        self.get(x, y).is_some_and(|key| key != TRANSPARENT as u8)
    }
}

/// Which pixels touch a pixel: the 4 at its sides, or those and the 4 corners.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connectivity {
    Four,
    Eight,
}

const SIDES: [(i64, i64); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];
const SIDES_AND_CORNERS: [(i64, i64); 8] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

impl Connectivity {
    pub fn offsets(self) -> &'static [(i64, i64)] {
        match self {
            Connectivity::Four => &SIDES,
            Connectivity::Eight => &SIDES_AND_CORNERS,
        }
    }
}

/// For every pixel, true when it is transparent and can be reached from a
/// transparent pixel on the border of the canvas by moving up, down, left or
/// right through transparent pixels. A transparent pixel that is not reached is
/// a hole.
pub fn exterior(view: View) -> Vec<bool> {
    let (w, h) = (view.width as i64, view.height as i64);
    let mut reached = vec![false; view.width * view.height];
    let mut stack = Vec::new();
    let mut visit = |x: i64, y: i64, stack: &mut Vec<(i64, i64)>| {
        let index = y as usize * view.width + x as usize;
        if !reached[index] && !view.is_opaque(x, y) {
            reached[index] = true;
            stack.push((x, y));
        }
    };
    for x in 0..w {
        visit(x, 0, &mut stack);
        visit(x, h - 1, &mut stack);
    }
    for y in 0..h {
        visit(0, y, &mut stack);
        visit(w - 1, y, &mut stack);
    }
    while let Some((x, y)) = stack.pop() {
        for (dx, dy) in SIDES {
            let (nx, ny) = (x + dx, y + dy);
            if view.get(nx, ny).is_some() {
                visit(nx, ny, &mut stack);
            }
        }
    }
    reached
}

#[cfg(test)]
mod support_tests;

#[cfg(test)]
mod outline_tests;

#[cfg(test)]
mod shade_tests;

#[cfg(test)]
mod report_tests;
