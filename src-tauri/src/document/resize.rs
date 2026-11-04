//! The pixel math of changing the canvas of a sprite: where the old content
//! goes, how a frame is cropped or extended and how anchors follow.

use super::model::{Anchor, TRANSPARENT};

/// Where the old content stays when the canvas changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeAnchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl ResizeAnchor {
    pub fn parse(text: &str) -> Option<ResizeAnchor> {
        Some(match text {
            "top-left" => ResizeAnchor::TopLeft,
            "top" => ResizeAnchor::Top,
            "top-right" => ResizeAnchor::TopRight,
            "left" => ResizeAnchor::Left,
            "center" => ResizeAnchor::Center,
            "right" => ResizeAnchor::Right,
            "bottom-left" => ResizeAnchor::BottomLeft,
            "bottom" => ResizeAnchor::Bottom,
            "bottom-right" => ResizeAnchor::BottomRight,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ResizeAnchor::TopLeft => "top-left",
            ResizeAnchor::Top => "top",
            ResizeAnchor::TopRight => "top-right",
            ResizeAnchor::Left => "left",
            ResizeAnchor::Center => "center",
            ResizeAnchor::Right => "right",
            ResizeAnchor::BottomLeft => "bottom-left",
            ResizeAnchor::Bottom => "bottom",
            ResizeAnchor::BottomRight => "bottom-right",
        }
    }

    /// Where the old top left pixel lands in the new canvas. Negative when
    /// the old content is cut on that side. When the difference is odd, the
    /// smaller part goes to the left and top.
    pub fn offset(self, old: (u32, u32), new: (u32, u32)) -> (i64, i64) {
        let (dx, dy) = (i64::from(new.0) - i64::from(old.0), i64::from(new.1) - i64::from(old.1));
        let x = match self {
            ResizeAnchor::TopLeft | ResizeAnchor::Left | ResizeAnchor::BottomLeft => 0,
            ResizeAnchor::Top | ResizeAnchor::Center | ResizeAnchor::Bottom => dx / 2,
            ResizeAnchor::TopRight | ResizeAnchor::Right | ResizeAnchor::BottomRight => dx,
        };
        let y = match self {
            ResizeAnchor::TopLeft | ResizeAnchor::Top | ResizeAnchor::TopRight => 0,
            ResizeAnchor::Left | ResizeAnchor::Center | ResizeAnchor::Right => dy / 2,
            ResizeAnchor::BottomLeft | ResizeAnchor::Bottom | ResizeAnchor::BottomRight => dy,
        };
        (x, y)
    }
}

/// One frame of one layer in the new size. The pixel at `x, y` of the old
/// frame goes to `x + offset.0, y + offset.1`; what falls outside is cut and
/// the new pixels are transparent.
pub fn resize_pixels(old: &[u8], old_size: (u32, u32), new_size: (u32, u32), offset: (i64, i64)) -> Vec<u8> {
    let (old_w, old_h) = (old_size.0 as i64, old_size.1 as i64);
    let (new_w, new_h) = (new_size.0 as i64, new_size.1 as i64);
    let mut out = vec![TRANSPARENT as u8; (new_w * new_h) as usize];
    for y in 0..old_h {
        let ny = y + offset.1;
        if !(0..new_h).contains(&ny) {
            continue;
        }
        for x in 0..old_w {
            let nx = x + offset.0;
            if (0..new_w).contains(&nx) {
                out[(ny * new_w + nx) as usize] = old[(y * old_w + x) as usize];
            }
        }
    }
    out
}

/// The anchors moved by `offset`: those that stay inside the new canvas, and
/// those that end outside (they are removed). Frame numbers do not change.
pub fn move_anchors(anchors: &[Anchor], offset: (i64, i64), new_size: (u32, u32)) -> (Vec<Anchor>, Vec<Anchor>) {
    let mut kept = Vec::new();
    let mut dropped = Vec::new();
    for anchor in anchors {
        let (x, y) = (i64::from(anchor.x) + offset.0, i64::from(anchor.y) + offset.1);
        if (0..i64::from(new_size.0)).contains(&x) && (0..i64::from(new_size.1)).contains(&y) {
            kept.push(Anchor { x: x as u32, y: y as u32, ..anchor.clone() });
        } else {
            dropped.push(anchor.clone());
        }
    }
    (kept, dropped)
}
