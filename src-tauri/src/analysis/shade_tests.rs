//! Task 1.2: `auto_shade` on plain frames, with the result written as text.

use super::support_tests::{after, frame, view};
use super::{Light, Region, ShadeOptions, auto_shade};

fn options<'a>(light: Light, keys: &'a [char], base: char) -> ShadeOptions<'a> {
    ShadeOptions { light, base, keys, region: None }
}

fn square() -> (Vec<u8>, usize, usize) {
    frame(&[
        "........", ".BBBBBB.", ".BBBBBB.", ".BBBBBB.", ".BBBBBB.", ".BBBBBB.", ".BBBBBB.", "........",
    ])
}

// Checks the rule on a 6x6 square lit from the top left, ramp D B L: the top
// row and the left column are light, the bottom row and the right column are
// dark, and the two corners that face sideways to the light (equal distance on
// both sides) stay neutral.
#[test]
fn a_square_lit_from_the_top_left() {
    let f = square();
    let shade = auto_shade(view(&f), &options(Light::TopLeft, &['D', 'B', 'L'], 'B'));
    assert_eq!(
        after(&f, &shade.changes),
        ["........", ".LLLLLB.", ".LBBBBD.", ".LBBBBD.", ".LBBBBD.", ".LBBBBD.", ".BDDDDD.", "........"]
    );
    assert_eq!((shade.lit, shade.shadow), (9, 9));
}

// Checks that a longer ramp gives one pixel per step: the lightest key at the
// edge that faces the light, the next one a pixel inside, and the same on the
// dark side.
#[test]
fn a_longer_ramp_has_one_pixel_per_step() {
    let f = frame(&["bbbbbbbb"]);
    let keys = ['d', 'D', 'b', 'L', 'l'];
    let shade = auto_shade(view(&f), &options(Light::Left, &keys, 'b'));
    assert_eq!(after(&f, &shade.changes), ["lLbbbbDd"]);
}

// Checks that the light can come from a straight side: from the top only the
// top row is light and the bottom row dark, from the right the columns are.
#[test]
fn the_light_can_come_from_a_side() {
    let f = frame(&["BBBB", "BBBB", "BBBB", "BBBB"]);
    let ramp = ['D', 'B', 'L'];
    let top = auto_shade(view(&f), &options(Light::Top, &ramp, 'B'));
    assert_eq!(after(&f, &top.changes), ["LLLL", "BBBB", "BBBB", "DDDD"]);
    let right = auto_shade(view(&f), &options(Light::Right, &ramp, 'B'));
    assert_eq!(after(&f, &right.changes), ["DBBL", "DBBL", "DBBL", "DBBL"]);
}

// Checks a diagonal light from the bottom right: the mirror of the top left.
#[test]
fn the_light_can_come_from_a_corner() {
    let f = frame(&["BBBB", "BBBB", "BBBB", "BBBB"]);
    let shade = auto_shade(view(&f), &options(Light::BottomRight, &['D', 'B', 'L'], 'B'));
    assert_eq!(after(&f, &shade.changes), ["DDDB", "DBBL", "DBBL", "BLLL"]);
}

// Checks that other keys are never changed and stop the measure: the outline
// K around the body counts as the edge of the body.
#[test]
fn other_keys_are_kept_and_make_an_edge() {
    let f = frame(&["KKKKK", "KBBBK", "KBBBK", "KBBBK", "KKKKK"]);
    let shade = auto_shade(view(&f), &options(Light::TopLeft, &['D', 'B', 'L'], 'B'));
    assert_eq!(after(&f, &shade.changes), ["KKKKK", "KLLBK", "KLBDK", "KBDDK", "KKKKK"]);
}

// Checks `region`: only the pixels inside the box change, and they get the
// color they would get without the box (the box does not make a new edge).
#[test]
fn a_region_limits_the_pixels_but_not_the_measure() {
    let f = frame(&["bbbbbbbb"]);
    let ramp = ['D', 'b', 'L'];
    let region = Some(Region { x: 3, y: 0, width: 5, height: 1 });
    let shade = auto_shade(view(&f), &ShadeOptions { region, ..options(Light::Left, &ramp, 'b') });
    assert_eq!(after(&f, &shade.changes), ["bbbbbbbD"]);
}

// Checks that a ramp without the base key, or a frame without base pixels,
// changes nothing.
#[test]
fn nothing_to_shade_changes_nothing() {
    let f = square();
    assert!(auto_shade(view(&f), &options(Light::Top, &['D', 'L'], 'B')).changes.is_empty());
    assert!(auto_shade(view(&f), &options(Light::Top, &['D', 'X', 'L'], 'X')).changes.is_empty());
}

// Checks the names of the light directions.
#[test]
fn light_names_are_parsed() {
    assert_eq!(Light::parse("bottom-left"), Some(Light::BottomLeft));
    assert_eq!(Light::parse("up"), None);
    for name in Light::NAMES {
        assert!(Light::parse(name).is_some(), "{name}");
    }
}
