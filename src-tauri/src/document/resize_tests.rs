use super::resize::*;
use super::Anchor;

fn anchor(name: &str, x: u32, y: u32, frame: Option<u32>) -> Anchor {
    Anchor { name: name.to_string(), x, y, frame }
}

// Checks the names of the nine places, because the tool, the dialog and the
// docs all use them.
#[test]
fn the_nine_anchors_have_names() {
    for name in ["top-left", "top", "top-right", "left", "center", "right", "bottom-left", "bottom", "bottom-right"] {
        assert_eq!(ResizeAnchor::parse(name).unwrap().as_str(), name);
    }
    assert!(ResizeAnchor::parse("middle").is_none());
}

// Checks where the old top left corner lands for each anchor when the canvas
// grows from 4x4 to 8x6 (so the content sits at the corner/edge/center).
#[test]
fn the_offset_puts_the_content_at_the_anchor() {
    let at = |a: &str| ResizeAnchor::parse(a).unwrap().offset((4, 4), (8, 6));
    assert_eq!(at("top-left"), (0, 0));
    assert_eq!(at("top"), (2, 0));
    assert_eq!(at("top-right"), (4, 0));
    assert_eq!(at("left"), (0, 1));
    assert_eq!(at("center"), (2, 1));
    assert_eq!(at("right"), (4, 1));
    assert_eq!(at("bottom-left"), (0, 2));
    assert_eq!(at("bottom"), (2, 2));
    assert_eq!(at("bottom-right"), (4, 2));
}

// Checks that cropping uses negative offsets (the part left of the anchor is cut).
#[test]
fn cropping_has_negative_offsets() {
    let at = |a: &str| ResizeAnchor::parse(a).unwrap().offset((8, 8), (4, 4));
    assert_eq!(at("top-left"), (0, 0));
    assert_eq!(at("bottom-right"), (-4, -4));
    assert_eq!(at("center"), (-2, -2));
}

// Checks the odd difference: the smaller part goes to the left/top, in both
// directions (growing by 3 gives 1 before and 2 after; cropping by 3 cuts 1 before and 2 after).
#[test]
fn an_odd_difference_gives_the_smaller_part_to_the_left_and_top() {
    let center = ResizeAnchor::Center;
    assert_eq!(center.offset((4, 4), (7, 7)), (1, 1));
    assert_eq!(center.offset((7, 7), (4, 4)), (-1, -1));
}

// Checks that extending keeps the pixels at the offset and fills the rest
// with transparent pixels ("." keys).
#[test]
fn extending_adds_transparent_pixels() {
    let old = b"AB\nCD".iter().copied().filter(|b| *b != b'\n').collect::<Vec<u8>>();
    let grown = resize_pixels(&old, (2, 2), (4, 4), (1, 1));
    // Rows: "....", ".AB.", ".CD.", "....".
    assert_eq!(grown, b".....AB..CD.....".to_vec());
}

// Checks that cropping keeps only what is inside the new canvas.
#[test]
fn cropping_cuts_what_is_outside() {
    let old = b"ABCDEFGHIJKLMNOP".to_vec(); // 4x4
    let cut = resize_pixels(&old, (4, 4), (2, 2), (-1, -1));
    assert_eq!(cut, b"FG JK".iter().copied().filter(|b| *b != b' ').collect::<Vec<u8>>());
}

// Checks a resize that crops one side and extends the other (width shrinks, height grows).
#[test]
fn width_can_shrink_while_height_grows() {
    let old = b"ABCD".to_vec(); // 2x2
    let out = resize_pixels(&old, (2, 2), (1, 3), (0, 1));
    assert_eq!(out, b".A C".iter().copied().filter(|b| *b != b' ').collect::<Vec<u8>>());
}

// Checks that anchors move by the offset, and that the ones that end outside
// the new canvas are separated (frame numbers do not change).
#[test]
fn anchors_move_and_the_outside_ones_are_dropped() {
    let anchors = vec![anchor("feet", 1, 3, None), anchor("hand", 3, 0, Some(2)), anchor("head", 0, 0, None)];
    let (kept, dropped) = move_anchors(&anchors, (-1, 0), (3, 4));
    assert_eq!(kept, vec![anchor("feet", 0, 3, None), anchor("hand", 2, 0, Some(2))]);
    assert_eq!(dropped, vec![anchor("head", 0, 0, None)]);
}

// Checks that an anchor on the last column or row still fits, and one past it does not.
#[test]
fn the_edge_of_the_new_canvas_is_inside() {
    let (kept, dropped) = move_anchors(&[anchor("a", 3, 3, None), anchor("b", 4, 3, None)], (0, 0), (4, 4));
    assert_eq!(kept.len(), 1);
    assert_eq!(dropped[0].name, "b");
}
