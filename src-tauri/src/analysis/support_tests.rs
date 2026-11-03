//! Helpers shared by the tests of this module: frames written as text rows.

use super::View;
use crate::document::PixelChange;

/// A frame from text rows (all the same length). Returns the pixels and size.
pub fn frame(rows: &[&str]) -> (Vec<u8>, usize, usize) {
    let width = rows[0].len();
    assert!(rows.iter().all(|r| r.len() == width), "rows must have the same length");
    (rows.iter().flat_map(|r| r.bytes()).collect(), width, rows.len())
}

/// The view of a frame made by `frame`.
pub fn view(frame: &(Vec<u8>, usize, usize)) -> View<'_> {
    View::new(&frame.0, frame.1, frame.2)
}

/// The frame after the changes, as text rows.
pub fn after(frame: &(Vec<u8>, usize, usize), changes: &[PixelChange]) -> Vec<String> {
    let (mut pixels, width, _) = (frame.0.clone(), frame.1, frame.2);
    for c in changes {
        pixels[c.y as usize * width + c.x as usize] = c.key as u8;
    }
    pixels.chunks(width).map(|r| String::from_utf8(r.to_vec()).unwrap()).collect()
}
