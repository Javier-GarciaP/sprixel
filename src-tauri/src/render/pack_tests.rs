use super::*;

// Checks the spec example: 14 frames of 8x8 with spacing 1 make a sheet of
// 71x17 pixels (8 frames in the first row, 6 in the second).
#[test]
fn fourteen_frames_of_8x8_with_spacing_1_make_71x17() {
    let sizes = vec![(8, 8); 14];
    let layout = pack_rows(&sizes, 1, FRAMES_PER_ROW);
    assert_eq!((layout.width, layout.height), (71, 17));
    assert_eq!(layout.placements.len(), 14);
    assert_eq!(layout.placements[0], Placement { x: 0, y: 0, w: 8, h: 8 });
    assert_eq!(layout.placements[7], Placement { x: 63, y: 0, w: 8, h: 8 });
    assert_eq!(layout.placements[8], Placement { x: 0, y: 9, w: 8, h: 8 });
}

// Checks that no two frames overlap, whatever the sizes and spacing.
#[test]
fn frames_never_overlap() {
    let sizes: Vec<(u32, u32)> = (0..20).map(|i| (4 + i % 5, 3 + i % 7)).collect();
    for spacing in [0, 1, 16] {
        let layout = pack_rows(&sizes, spacing, 6);
        for (i, a) in layout.placements.iter().enumerate() {
            assert!(a.x + a.w <= layout.width && a.y + a.h <= layout.height);
            for b in &layout.placements[i + 1..] {
                let apart = a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
                assert!(apart, "{a:?} overlaps {b:?}");
            }
        }
    }
}

// Checks that fewer frames than one row make one row, with no extra spacing.
#[test]
fn a_short_list_is_one_row() {
    let layout = pack_rows(&[(8, 8), (8, 8), (8, 8)], 2, 8);
    assert_eq!((layout.width, layout.height), (8 * 3 + 2 * 2, 8));
}

// Checks determinism: the same input gives the same layout.
#[test]
fn same_input_same_layout() {
    let sizes = vec![(16, 16); 9];
    assert_eq!(pack_rows(&sizes, 1, 4), pack_rows(&sizes, 1, 4));
}

// Checks the empty case: no frames make an empty layout.
#[test]
fn no_frames_no_layout() {
    let layout = pack_rows(&[], 1, 8);
    assert_eq!((layout.width, layout.height, layout.placements.len()), (0, 0, 0));
}

// Checks that a row with frames of different heights is as tall as its
// tallest frame.
#[test]
fn row_height_is_the_tallest_frame() {
    let layout = pack_rows(&[(4, 4), (4, 10)], 0, 2);
    assert_eq!(layout.height, 10);
}

// Checks the sheet width rule: at most `FRAMES_PER_ROW` frames in a row, so a
// long animation wraps instead of making a very wide image.
#[test]
fn rows_wrap_after_the_frames_per_row_limit() {
    let layout = pack_rows(&vec![(4, 4); FRAMES_PER_ROW + 1], 0, FRAMES_PER_ROW);
    assert_eq!(layout.width, 4 * FRAMES_PER_ROW as u32);
    assert_eq!(layout.height, 8);
    assert_eq!(layout.placements[FRAMES_PER_ROW], Placement { x: 0, y: 4, w: 4, h: 4 });
}
