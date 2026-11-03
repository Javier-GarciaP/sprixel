use super::*;
use super::preview::{LABEL_BAND, draw_number};

fn one(rgba: [u8; 4]) -> Image {
    Image { width: 1, height: 1, rgba: rgba.to_vec() }
}

fn px(image: &Image, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * image.width + x) * 4) as usize;
    image.rgba[at..at + 4].try_into().unwrap()
}

// Checks the names the tools accept, and that anything else is refused.
#[test]
fn background_names_are_parsed() {
    assert_eq!(Background::parse("checker"), Some(Background::Checker));
    assert_eq!(Background::parse("light"), Some(Background::Light));
    assert_eq!(Background::parse("dark"), Some(Background::Dark));
    assert_eq!(Background::parse("transparent"), Some(Background::Transparent));
    assert_eq!(Background::parse("purple"), None);
}

// Checks that an opaque pixel is not changed by any background, and that a
// transparent pixel takes the color of the background (white and dark here).
#[test]
fn flat_backgrounds_fill_transparent_pixels_only() {
    let red = one([255, 0, 0, 255]);
    assert_eq!(flatten(&red, Background::Light, 1).rgba, red.rgba);
    let clear = one([0, 0, 0, 0]);
    assert_eq!(flatten(&clear, Background::Light, 1).rgba, vec![255, 255, 255, 255]);
    assert_eq!(flatten(&clear, Background::Dark, 1).rgba, vec![0x20, 0x20, 0x20, 255]);
}

// Checks that `transparent` leaves the image exactly as it is (alpha 0 stays).
#[test]
fn transparent_background_keeps_the_alpha() {
    let clear = Image { width: 2, height: 1, rgba: vec![0, 0, 0, 0, 10, 20, 30, 255] };
    assert_eq!(flatten(&clear, Background::Transparent, 1), clear);
}

// Checks the checkerboard: squares of `cell` pixels in two different grays,
// alternating in both directions. A white pixel is never equal to either gray,
// so a white sprite is visible.
#[test]
fn checker_alternates_two_grays() {
    let clear = Image { width: 4, height: 4, rgba: vec![0; 4 * 4 * 4] };
    let out = flatten(&clear, Background::Checker, 2);
    let (a, b) = (px(&out, 0, 0), px(&out, 2, 0));
    assert_ne!(a, b);
    assert_eq!(px(&out, 1, 1), a); // same square
    assert_eq!(px(&out, 0, 2), b); // next row of squares
    assert_eq!(px(&out, 2, 2), a);
    assert_eq!(a[3], 255);
    assert!(![a, b].contains(&[255, 255, 255, 255]));
}

// Checks the "over" blend of a half transparent pixel on a white background:
// the result is half way between the two colors, and fully opaque.
#[test]
fn translucent_pixels_blend_with_the_background() {
    let half = one([0, 0, 0, 128]);
    let out = flatten(&half, Background::Light, 1);
    assert_eq!(out.rgba[3], 255);
    assert!((out.rgba[0] as i32 - 127).abs() <= 1, "{:?}", out.rgba);
}

fn blank(w: u32, h: u32) -> Image {
    Image { width: w, height: h, rgba: vec![0; (w * h * 4) as usize] }
}

fn opaque_count(image: &Image) -> usize {
    image.rgba.chunks_exact(4).filter(|p| p[3] != 0).count()
}

// Checks the sheet size: 4 frames of 4x4 in 2 columns, a gap of `unit` (1)
// between cells, no label band: 4+1+4 = 9 on each side.
#[test]
fn sheet_size_follows_columns_and_gap() {
    let frames = vec![blank(4, 4); 4];
    let sheet = render_sheet(&frames, 2, 1, false, Background::Transparent);
    assert_eq!((sheet.width, sheet.height), (9, 9));
    // 3 frames in 2 columns: still 2 rows.
    let sheet = render_sheet(&frames[..3], 2, 1, false, Background::Transparent);
    assert_eq!((sheet.width, sheet.height), (9, 9));
    // Columns never exceed the number of frames.
    let sheet = render_sheet(&frames[..2], 5, 1, false, Background::Transparent);
    assert_eq!((sheet.width, sheet.height), (9, 4));
}

// Checks that each frame is placed in reading order (left to right, then down):
// frame 2 has a red pixel and lands in the second column.
#[test]
fn frames_are_placed_in_order() {
    let mut second = blank(2, 2);
    second.rgba[0..4].copy_from_slice(&[255, 0, 0, 255]);
    let frames = vec![blank(2, 2), second];
    let sheet = render_sheet(&frames, 2, 1, false, Background::Transparent);
    assert_eq!(px(&sheet, 3, 0), [255, 0, 0, 255]); // 2 wide + 1 gap
    assert_eq!(opaque_count(&sheet), 1);
}

// Checks the labels: with `labels` the sheet is taller (a band above each
// frame) and the band has ink pixels; without it there is nothing drawn.
#[test]
fn labels_add_a_band_with_the_frame_number() {
    let frames = vec![blank(4, 4); 2];
    let plain = render_sheet(&frames, 2, 1, false, Background::Transparent);
    let labeled = render_sheet(&frames, 2, 1, true, Background::Transparent);
    assert_eq!(opaque_count(&plain), 0);
    assert_eq!(labeled.width, plain.width);
    assert_eq!(labeled.height, plain.height + LABEL_BAND);
    assert!(opaque_count(&labeled) > 0);
    // Nothing is drawn below the band (the frames are empty).
    let below = Image { width: labeled.width, height: 4, rgba: labeled.rgba[(LABEL_BAND * labeled.width * 4) as usize..].to_vec() };
    assert_eq!(opaque_count(&below), 0);
}

// Checks that "1" and "2" look different, that ink is white on dark and black
// elsewhere, and that a number with two digits is wider than one with one.
#[test]
fn digits_differ_and_ink_follows_the_background() {
    let one = draw_number(1, 1);
    let two = draw_number(2, 1);
    assert_ne!(one, two);
    assert_eq!(one.1, 5);
    assert!(draw_number(12, 1).0 > one.0);
    let frames = vec![blank(8, 4)];
    let dark = render_sheet(&frames, 1, 1, true, Background::Dark);
    let light = render_sheet(&frames, 1, 1, true, Background::Light);
    assert!(dark.rgba.chunks_exact(4).any(|p| p == [255, 255, 255, 255]));
    assert!(light.rgba.chunks_exact(4).any(|p| p == [0, 0, 0, 255]));
}
