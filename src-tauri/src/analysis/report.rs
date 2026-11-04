//! `analyze_sprite`: what is wrong in a sprite, as plain numbers and places.

use super::{Connectivity, Region, View, exterior};
use std::collections::BTreeMap;

/// The opaque pixels that touch no other opaque pixel (8 neighbors), as (x, y)
/// in reading order.
pub fn loose_pixels(view: View) -> Vec<(u32, u32)> {
    let mut found = Vec::new();
    for y in 0..view.height as i64 {
        for x in 0..view.width as i64 {
            let alone = !Connectivity::Eight.offsets().iter().any(|(dx, dy)| view.is_opaque(x + dx, y + dy));
            if view.is_opaque(x, y) && alone {
                found.push((x as u32, y as u32));
            }
        }
    }
    found
}

/// The pixels where the shape is open to the outside: opaque pixels whose key is
/// not `outline_key` and that have a pixel of the exterior at their side. The
/// edge of the canvas is not the outside, and holes do not need an outline.
pub fn outline_gaps(view: View, outline_key: char) -> Vec<(u32, u32)> {
    let outside = exterior(view);
    let mut found = Vec::new();
    for y in 0..view.height as i64 {
        for x in 0..view.width as i64 {
            if !view.is_opaque(x, y) || view.get(x, y) == Some(outline_key as u8) {
                continue;
            }
            let open = Connectivity::Four.offsets().iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                view.get(nx, ny).is_some() && outside[ny as usize * view.width + nx as usize]
            });
            if open {
                found.push((x as u32, y as u32));
            }
        }
    }
    found
}

/// The palette keys that some frame uses and those that no frame uses, both in
/// the order of the palette.
pub fn colors(frames: &[View], palette: &[char]) -> (Vec<char>, Vec<char>) {
    let uses = |key: char| {
        frames.iter().any(|f| (0..f.height as i64).any(|y| (0..f.width as i64).any(|x| f.get(x, y) == Some(key as u8))))
    };
    palette.iter().partition(|key| uses(**key))
}

/// How a frame differs from the next one.
#[derive(Debug, PartialEq, Eq)]
pub struct FrameChange {
    pub changed_pixels: usize,
    /// The smallest box with all the changed pixels, or `None` if none changed.
    pub bbox: Option<Region>,
    /// The bottom `ground_rows` rows are equal in both frames.
    pub ground_kept: bool,
}

/// Compares two frames of the same size. `ground_rows` is clamped to the height.
pub fn compare(from: View, to: View, ground_rows: usize) -> FrameChange {
    assert_eq!((from.width, from.height), (to.width, to.height), "frames have the same size");
    let mut changed = 0;
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    let first_ground_row = from.height - ground_rows.min(from.height);
    let mut ground_kept = true;
    for y in 0..from.height {
        for x in 0..from.width {
            if from.get(x as i64, y as i64) != to.get(x as i64, y as i64) {
                changed += 1;
                (x0, y0, x1, y1) = (x0.min(x as u32), y0.min(y as u32), x1.max(x as u32), y1.max(y as u32));
                ground_kept &= y < first_ground_row;
            }
        }
    }
    let bbox = (changed > 0).then(|| Region { x: x0, y: y0, width: x1 - x0 + 1, height: y1 - y0 + 1 });
    FrameChange { changed_pixels: changed, bbox, ground_kept }
}

/// The key of the outline of a sprite, when there is a clear one. The edge of
/// the silhouette is the opaque pixels that have a pixel of the exterior at
/// their side (the same pixels `outline_gaps` looks at). The key that most of
/// them have, counted over all frames, wins if it has MORE than half of them.
/// With no shape, a tie or no majority the answer is `None`.
pub fn detect_outline_key(frames: &[View]) -> Option<char> {
    let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
    for view in frames {
        let outside = exterior(*view);
        for y in 0..view.height as i64 {
            for x in 0..view.width as i64 {
                let open = Connectivity::Four.offsets().iter().any(|(dx, dy)| {
                    let (nx, ny) = (x + dx, y + dy);
                    view.get(nx, ny).is_some() && outside[ny as usize * view.width + nx as usize]
                });
                if view.is_opaque(x, y) && open {
                    *counts.entry(view.get(x, y).expect("inside")).or_default() += 1;
                }
            }
        }
    }
    let total: usize = counts.values().sum();
    counts.into_iter().find(|(_, n)| n * 2 > total).map(|(key, _)| key as char)
}
