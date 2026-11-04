//! Task 1.3: the checks of `analyze_sprite` on plain frames.

use super::support_tests::{frame, view};
use super::{FrameChange, Region, colors, compare, loose_pixels, outline_gaps};

// Checks that a pixel with no opaque neighbor is loose, a block is not, and
// two pixels that touch only at a corner are not loose (8 neighbors).
#[test]
fn loose_pixels_touch_nothing() {
    let f = frame(&["BB.....", "BB..X.B", ".......", ".Y....B", "......."]);
    // X and Y are alone. The B at (6,1) and (6,3) are alone too (not touching).
    assert_eq!(loose_pixels(view(&f)), [(4, 1), (6, 1), (1, 3), (6, 3)]);
    let diagonal = frame(&["B.", ".B"]);
    assert!(loose_pixels(view(&diagonal)).is_empty());
}

// Checks that an outline with one missing pixel reports the body pixel that is
// open to the outside there, and nothing else.
#[test]
fn a_missing_outline_pixel_is_a_gap() {
    let f = frame(&[".......", ".KKKKK.", ".KBBBK.", "..BBBK.", ".KBBBK.", ".KKKKK.", "......."]);
    assert_eq!(outline_gaps(view(&f), 'K'), [(2, 3)]);
}

// Checks that a hole inside the shape needs no outline, so it is not a gap.
#[test]
fn a_hole_is_not_a_gap() {
    let f = frame(&["KKKKKKK", "KBBBBBK", "KB...BK", "KB...BK", "KB...BK", "KBBBBBK", "KKKKKKK"]);
    assert!(outline_gaps(view(&f), 'K').is_empty());
}

// Checks that the edge of the canvas is not "the outside": a shape that fills
// the sprite has no gaps.
#[test]
fn the_canvas_edge_is_not_a_gap() {
    let f = frame(&["BB", "BB"]);
    assert!(outline_gaps(view(&f), 'K').is_empty());
}

// Checks the colors: used keys and unused keys, both in palette order.
#[test]
fn colors_are_used_or_unused_in_palette_order() {
    let a = frame(&["K.", "B."]);
    let b = frame(&["..", "K."]);
    let (used, unused) = colors(&[view(&a), view(&b)], &['K', 'B', 'D', 'W']);
    assert_eq!((used, unused), (vec!['K', 'B'], vec!['D', 'W']));
}

fn pair(a: &[&str], b: &[&str], ground: usize) -> FrameChange {
    compare(view(&frame(a)), view(&frame(b)), ground)
}

// Checks one changed pixel above the ground: the count, the box and that the
// ground is kept.
#[test]
fn a_small_change_keeps_the_ground() {
    let change = pair(&["....", "....", "..B.", "BBBB"], &["....", ".B..", "..B.", "BBBB"], 2);
    assert_eq!(change.changed_pixels, 1);
    assert_eq!(change.bbox, Some(Region { x: 1, y: 1, width: 1, height: 1 }));
    assert!(change.ground_kept);
}

// Checks that a change in the bottom rows means the ground moved.
#[test]
fn a_change_in_the_bottom_rows_moves_the_ground() {
    let change = pair(&["....", "....", "..B.", "BBBB"], &["....", "....", "..B.", "BBB."], 2);
    assert!(!change.ground_kept);
    assert_eq!(change.bbox, Some(Region { x: 3, y: 3, width: 1, height: 1 }));
    // With one ground row the pixel at row 2 would not matter, this one does.
    let same_row = pair(&["....", "....", "..B.", "BBBB"], &["....", "....", ".B..", "BBBB"], 1);
    assert!(same_row.ground_kept);
}

// Checks the box of several changes, and no change at all.
#[test]
fn the_box_holds_all_the_changes() {
    let change = pair(&["B...", "....", "....", "...."], &["....", "....", "..B.", "...."], 2);
    assert_eq!(change.changed_pixels, 2);
    assert_eq!(change.bbox, Some(Region { x: 0, y: 0, width: 3, height: 3 }));
    let none = pair(&["B.", ".."], &["B.", ".."], 2);
    assert_eq!((none.changed_pixels, none.bbox, none.ground_kept), (0, None, true));
}

// Checks that asking for more ground rows than the sprite has uses them all.
#[test]
fn ground_rows_are_clamped_to_the_height() {
    let change = pair(&["B.", ".."], &[".B", ".."], 9);
    assert!(!change.ground_kept);
}

// Task 3.1: `detect_outline_key` picks the key that most of the silhouette edge
// has, and nothing when it cannot decide.

#[test]
fn the_outline_key_is_the_most_common_edge_key() {
    use super::support_tests::{frame, view};
    // A G block with a K ring: all 12 edge pixels are K.
    let ringed = frame(&["......", ".KKKK.", ".KGGK.", ".KGGK.", ".KKKK.", "......"]);
    assert_eq!(super::detect_outline_key(&[view(&ringed)]), Some('K'));
    // Two keys in equal numbers: no decision.
    let tie = frame(&["......", ".KKGG.", ".KKGG.", "......"]);
    assert_eq!(super::detect_outline_key(&[view(&tie)]), None);
    // Nothing drawn: no decision.
    let empty = frame(&["...", "..."]);
    assert_eq!(super::detect_outline_key(&[view(&empty)]), None);
    // Just over half is enough; exactly half is not: 3 K and 3 G edge pixels.
    let half = frame(&["....", ".KKK", ".GGG", "...."]);
    assert_eq!(super::detect_outline_key(&[view(&half)]), None);
}

// Task 3.2: `lowest_row` is the row where the sprite stands. A lifted foot
// does not change it while the other foot stays down.
#[test]
fn the_lowest_row_ignores_a_lifted_foot() {
    use super::support_tests::{frame, view};
    let both = frame(&["..B..B..", "..B..B..", "..B..B.."]);
    let lifted = frame(&["..B..B..", "..B..B..", "..B....."]);
    assert_eq!(super::lowest_row(view(&both)), Some(2));
    assert_eq!(super::lowest_row(view(&lifted)), Some(2));
    assert_eq!(super::lowest_row(view(&frame(&["...", "..."]))), None);
}
