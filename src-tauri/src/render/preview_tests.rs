use super::*;

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
