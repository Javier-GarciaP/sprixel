//! `auto_outline`: a one pixel outline around the shape of a frame.

use super::{Connectivity, View, exterior};
use crate::document::PixelChange;
use std::collections::HashSet;

pub struct OutlineOptions {
    /// The palette key of the outline.
    pub key: char,
    pub connectivity: Connectivity,
    /// Do not outline holes (transparent areas the outside cannot reach).
    pub outside_only: bool,
    /// Recolor the edge pixels of the shape instead of drawing around it.
    pub inner: bool,
}

pub struct Outline {
    /// The pixels to set, in reading order. Only pixels that really change.
    pub changes: Vec<PixelChange>,
    /// Outline pixels that fall outside the canvas (not drawn).
    pub clipped: usize,
}

pub fn auto_outline(view: View, options: &OutlineOptions) -> Outline {
    let outside = exterior(view);
    // A transparent pixel counts as "outside the shape" when it is in the
    // exterior, or always when holes are not ignored.
    let open = |x: i64, y: i64| {
        view.get(x, y).is_some()
            && !view.is_opaque(x, y)
            && (!options.outside_only || outside[y as usize * view.width + x as usize])
    };
    let key = options.key as u8;
    let mut changes = Vec::new();
    for y in 0..view.height as i64 {
        for x in 0..view.width as i64 {
            let here = view.get(x, y).expect("inside the canvas");
            let wanted = if options.inner {
                view.is_opaque(x, y) && options.connectivity.offsets().iter().any(|(dx, dy)| open(x + dx, y + dy))
            } else {
                open(x, y) && options.connectivity.offsets().iter().any(|(dx, dy)| view.is_opaque(x + dx, y + dy))
            };
            if wanted && here != key {
                changes.push(PixelChange { x: x as u32, y: y as u32, key: options.key });
            }
        }
    }
    Outline { changes, clipped: if options.inner { 0 } else { clipped(view, options.connectivity) } }
}

/// The places outside the canvas that touch an opaque pixel, each counted once.
fn clipped(view: View, connectivity: Connectivity) -> usize {
    let mut places = HashSet::new();
    for y in 0..view.height as i64 {
        for x in 0..view.width as i64 {
            if !view.is_opaque(x, y) {
                continue;
            }
            for (dx, dy) in connectivity.offsets() {
                if view.get(x + dx, y + dy).is_none() {
                    places.insert((x + dx, y + dy));
                }
            }
        }
    }
    places.len()
}
