use super::*;
use crate::document::{Color, Layer, Palette, Sprite};

fn palette() -> Palette {
    Palette {
        name: "main".to_string(),
        colors: vec![
            Color { key: 'R', name: "red".to_string(), hex: "#FF0000".to_string() },
            Color { key: 'b', name: "blue".to_string(), hex: "#0000ff".to_string() },
        ],
    }
}

// A 2x2 sprite: red, transparent / transparent, blue.
fn sprite() -> Sprite {
    let mut s = Sprite::new("dot", 2, 2, "main").unwrap();
    s.layers[0].frames[0].pixels = b"R..b".to_vec();
    s
}

const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const CLEAR: [u8; 4] = [0, 0, 0, 0];

fn rgba(pixels: &[[u8; 4]]) -> Vec<u8> {
    pixels.concat()
}

// Checks the colors of a frame at 1x: keys become their hex color and `.`
// becomes a fully transparent pixel.
#[test]
fn renders_palette_colors_and_transparency() {
    let image = render_frame(&sprite(), &palette(), 0, 1);
    assert_eq!((image.width, image.height), (2, 2));
    assert_eq!(image.rgba, rgba(&[RED, CLEAR, CLEAR, BLUE]));
}

// Checks nearest-neighbor scaling: at 2x every pixel becomes a 2x2 block, with
// no blending, so the colors stay exact.
#[test]
fn scale_repeats_each_pixel_without_blending() {
    let image = render_frame(&sprite(), &palette(), 0, 2);
    assert_eq!((image.width, image.height), (4, 4));
    let row_top = rgba(&[RED, RED, CLEAR, CLEAR]);
    let row_bottom = rgba(&[CLEAR, CLEAR, BLUE, BLUE]);
    let expected = [row_top.clone(), row_top, row_bottom.clone(), row_bottom].concat();
    assert_eq!(image.rgba, expected);
}

// Checks that a 64x64 sprite at 4x is 256x256 (the spec scenario).
#[test]
fn scale_four_of_a_64_sprite_is_256() {
    let big = Sprite::new("big", 64, 64, "main").unwrap();
    let image = render_frame(&big, &palette(), 0, 4);
    assert_eq!((image.width, image.height), (256, 256));
    assert_eq!(image.rgba.len(), 256 * 256 * 4);
}

// Checks that layers are stacked: a pixel of an upper layer covers the lower
// one, and a transparent pixel lets the lower one show.
#[test]
fn upper_layers_cover_lower_ones() {
    let mut s = sprite();
    s.layers.push(Layer {
        name: "Layer 2".to_string(),
        frames: vec![crate::document::Frame { pixels: b"b.b.".to_vec() }],
    });
    let image = render_frame(&s, &palette(), 0, 1);
    assert_eq!(image.rgba, rgba(&[BLUE, CLEAR, BLUE, BLUE]));
}

// Checks that the frame number picks the frame.
#[test]
fn renders_the_asked_frame() {
    let mut s = sprite();
    s.frame_durations.push(100);
    s.layers[0].frames.push(crate::document::Frame { pixels: b"bbbb".to_vec() });
    let image = render_frame(&s, &palette(), 1, 1);
    assert_eq!(image.rgba, rgba(&[BLUE, BLUE, BLUE, BLUE]));
}
