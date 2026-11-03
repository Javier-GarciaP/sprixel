//! Task 1.1: `auto_outline` on plain frames. Each test draws the frame and the
//! expected result as text, so the rule can be read in the test.

use super::support_tests::{after, frame, view};
use super::{Connectivity, OutlineOptions, auto_outline};

fn options(connectivity: Connectivity) -> OutlineOptions {
    OutlineOptions { key: 'K', connectivity, outside_only: false, inner: false }
}

fn block() -> (Vec<u8>, usize, usize) {
    frame(&["........", "........", "........", "...BB...", "...BB...", "........", "........", "........"])
}

// Checks the 8 neighbor outline: a 2x2 block gets a ring of 12 pixels, with
// the corners, and the block itself is not changed.
#[test]
fn eight_neighbors_draw_a_closed_ring() {
    let f = block();
    let out = auto_outline(view(&f), &options(Connectivity::Eight));
    assert_eq!(
        after(&f, &out.changes),
        ["........", "........", "..KKKK..", "..KBBK..", "..KBBK..", "..KKKK..", "........", "........"]
    );
    assert_eq!(out.changes.len(), 12);
    assert_eq!(out.clipped, 0);
}

// Checks the 4 neighbor outline: the four corners of the ring are not drawn.
#[test]
fn four_neighbors_leave_the_corners_empty() {
    let f = block();
    let out = auto_outline(view(&f), &options(Connectivity::Four));
    assert_eq!(
        after(&f, &out.changes),
        ["........", "........", "...KK...", "..KBBK..", "..KBBK..", "...KK...", "........", "........"]
    );
    assert_eq!(out.changes.len(), 8);
}

// Checks that the changes come row by row, left to right, so the replay in
// the editor is always the same.
#[test]
fn changes_are_in_reading_order() {
    let out = auto_outline(view(&block()), &options(Connectivity::Four));
    let order: Vec<(u32, u32)> = out.changes.iter().map(|c| (c.y, c.x)).collect();
    let mut sorted = order.clone();
    sorted.sort();
    assert_eq!(order, sorted);
    assert_eq!(order[0], (2, 3));
}

// A square ring of B with a hollow 3x3 inside, in a 7x7 frame.
fn ring() -> (Vec<u8>, usize, usize) {
    frame(&[".......", ".BBBBB.", ".B...B.", ".B...B.", ".B...B.", ".BBBBB.", "......."])
}

// Checks that by default the hole is outlined too (it touches the shape). The
// middle pixel of the 3x3 hole touches no shape pixel, so it stays empty.
#[test]
fn a_hole_is_outlined_by_default() {
    let f = ring();
    let out = auto_outline(view(&f), &options(Connectivity::Eight));
    assert_eq!(
        after(&f, &out.changes),
        ["KKKKKKK", "KBBBBBK", "KBKKKBK", "KBK.KBK", "KBKKKBK", "KBBBBBK", "KKKKKKK"]
    );
}

// Checks `outside_only`: the hole is left as it is; only the outside is drawn.
#[test]
fn outside_only_skips_the_holes() {
    let f = ring();
    let out = auto_outline(view(&f), &OutlineOptions { outside_only: true, ..options(Connectivity::Eight) });
    assert_eq!(
        after(&f, &out.changes),
        ["KKKKKKK", "KBBBBBK", "KB...BK", "KB...BK", "KB...BK", "KBBBBBK", "KKKKKKK"]
    );
}

// Checks that a hole that is only open through a diagonal still counts as
// closed: the outside is reached by up, down, left and right moves only.
#[test]
fn a_diagonal_gap_does_not_open_a_hole() {
    let f = frame(&[".......", "..BBB..", ".B...B.", ".B...B.", ".B...B.", "..BBB..", "......."]);
    let out = auto_outline(view(&f), &OutlineOptions { outside_only: true, ..options(Connectivity::Eight) });
    // The hole (rows 2 to 4, columns 2 to 4) is not drawn.
    let rows = after(&f, &out.changes);
    assert_eq!(rows[3], "KB...BK");
}

// Checks the inner outline: the edge pixels of the shape change, the
// transparent pixels do not, and the inside of the shape stays.
#[test]
fn inner_recolors_the_edge_of_the_shape() {
    let f = frame(&[".......", ".BBBBB.", ".BBBBB.", ".BBBBB.", ".BBBBB.", ".BBBBB.", "......."]);
    let out = auto_outline(view(&f), &OutlineOptions { inner: true, ..options(Connectivity::Eight) });
    assert_eq!(
        after(&f, &out.changes),
        [".......", ".KKKKK.", ".KBBBK.", ".KBBBK.", ".KBBBK.", ".KKKKK.", "......."]
    );
    assert_eq!(out.clipped, 0);
}

// Checks that the edge of the canvas is not an edge of the shape for the inner
// outline: a block that touches the left border is not outlined there.
#[test]
fn inner_ignores_the_canvas_border() {
    let f = frame(&["....", "BBB.", "BBB.", "BBB.", "...."]);
    let out = auto_outline(view(&f), &OutlineOptions { inner: true, ..options(Connectivity::Eight) });
    assert_eq!(after(&f, &out.changes), ["....", "KKK.", "BBK.", "KKK.", "...."]);
}

// Checks that with `inner` and `outside_only` a hole does not make an edge.
#[test]
fn inner_with_outside_only_ignores_the_holes() {
    let f = ring();
    let both = OutlineOptions { inner: true, outside_only: true, ..options(Connectivity::Four) };
    let out = auto_outline(view(&f), &both);
    // The ring is one pixel thick: its inner side touches the hole (ignored),
    // its outer side touches the outside, so all the ring pixels are edges.
    assert_eq!(after(&f, &out.changes), [".......", ".KKKKK.", ".K...K.", ".K...K.", ".K...K.", ".KKKKK.", "......."]);
}

// Checks that outline pixels outside the canvas are skipped and counted, once
// each.
#[test]
fn pixels_outside_the_canvas_are_clipped_and_counted() {
    let f = frame(&["....", "BB..", "BB..", "...."]);
    let out = auto_outline(view(&f), &options(Connectivity::Eight));
    assert_eq!(after(&f, &out.changes), ["KKK.", "BBK.", "BBK.", "KKK."]);
    // The four pixels of the column at x = -1 (y from 0 to 3).
    assert_eq!(out.clipped, 4);
}

// Checks that pixels that already have the outline key are not in the write,
// and that a frame with no shape gives no change.
#[test]
fn only_real_changes_are_listed() {
    let f = frame(&["....", ".KB.", "....", "...."]);
    let out = auto_outline(view(&f), &OutlineOptions { inner: true, ..options(Connectivity::Four) });
    // K stays K; the B is an edge pixel and changes.
    assert_eq!(out.changes.len(), 1);
    let empty = frame(&["....", "....", "....", "...."]);
    let out = auto_outline(view(&empty), &options(Connectivity::Eight));
    assert!(out.changes.is_empty());
    assert_eq!(out.clipped, 0);
}
