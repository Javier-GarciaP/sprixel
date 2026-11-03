//! Task 2.1: `only_over` and `symmetry` on the three drawing tools.

use crate::mcp::testing::{call, open_demo_project, revision, rows, setup};
use serde_json::json;

const BLANK8: &str = "......../......../......../......../......../......../......../........";

// Checks the "Shape over a color" scenario: a filled rectangle with
// `only_over` R changes the red pixels inside it, and the green ones and the
// transparent ones stay.
#[test]
fn a_shape_only_over_a_color_leaves_the_other_pixels() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let pixels = json!([{ "x": 0, "y": 0, "key": "R" }, { "x": 1, "y": 0, "key": "G" }, { "x": 2, "y": 0, "key": "R" }]);
    call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": pixels })).unwrap();

    let done = call(&core, &session, "draw_shape", json!({
        "name": "slime", "shape": "rectangle", "filled": true, "key": "G",
        "x": 0, "y": 0, "width": 4, "height": 2, "only_over": "R",
    })).unwrap();

    assert_eq!(done["pixels_changed"], 2);
    assert!(rows(&host, &[], "slime", 1).starts_with("GGG....."));
    // The middle G was already G; the dots were not red, so they stay.
    assert!(rows(&host, &[], "slime", 1).ends_with("......../......../........"));
}

// Checks `only_over` on `set_pixels` and on the region form of `apply_grid`.
#[test]
fn only_over_works_for_set_pixels_and_grid_regions() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "R" }] })).unwrap();

    call(&core, &session, "set_pixels", json!({
        "name": "slime", "only_over": "R",
        "pixels": [{ "x": 0, "y": 0, "key": "G" }, { "x": 1, "y": 0, "key": "G" }],
    })).unwrap();
    assert!(rows(&host, &[], "slime", 1).starts_with("G......."));

    call(&core, &session, "apply_grid", json!({
        "name": "slime", "grid": "GG\nGG\n", "frame": 1, "x": 0, "y": 0, "only_over": "G",
    })).unwrap();
    assert!(rows(&host, &[], "slime", 1).starts_with("G......./........"), "only the G pixel was written");
}

// Checks that a key that is not in the palette is refused.
#[test]
fn only_over_needs_a_palette_key() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);

    let error = call(&core, &session, "set_pixels", json!({
        "name": "slime", "only_over": "X", "pixels": [{ "x": 0, "y": 0, "key": "R" }],
    })).unwrap_err();

    assert_eq!(error["code"], "unknown_key");
    assert_eq!(revision(&host), before);
}

// Checks the symmetry scenarios: horizontal gives 2 pixels (x 1 and x 6 on an
// 8 wide sprite), vertical gives the other row, both gives 4.
#[test]
fn symmetry_mirrors_each_pixel() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let one = |symmetry: &str| {
        call(&core, &session, "set_pixels", json!({
            "name": "slime", "symmetry": symmetry, "pixels": [{ "x": 1, "y": 0, "key": "R" }],
        })).unwrap()["pixels_set"].as_u64().unwrap()
    };

    assert_eq!(one("horizontal"), 2);
    assert_eq!(rows(&host, &[], "slime", 1).split('/').next().unwrap(), ".R....R.");
    assert_eq!(one("both"), 2, "the two top pixels exist already; the two bottom ones are new");
    assert_eq!(rows(&host, &[], "slime", 1).split('/').last().unwrap(), ".R....R.");
    call(&core, &session, "set_pixels", json!({ "name": "slime", "symmetry": "vertical",
        "pixels": [{ "x": 3, "y": 2, "key": "G" }] })).unwrap();
    let all = rows(&host, &[], "slime", 1);
    let all: Vec<&str> = all.split('/').collect();
    assert_eq!((all[2], all[5]), ("...G....", "...G...."));
}

// Checks a given axis, and that a mirrored pixel outside the sprite fails
// without changing anything.
#[test]
fn symmetry_axis_can_move_and_must_stay_inside() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "set_pixels", json!({
        "name": "slime", "symmetry": "horizontal", "axis": { "x": 2 }, "pixels": [{ "x": 1, "y": 0, "key": "R" }],
    })).unwrap();
    assert!(rows(&host, &[], "slime", 1).starts_with(".R.R...."));

    let before = revision(&host);
    let error = call(&core, &session, "set_pixels", json!({
        "name": "slime", "symmetry": "horizontal", "axis": { "x": 0.5 }, "pixels": [{ "x": 2, "y": 3, "key": "G" }],
    })).unwrap_err();
    assert_eq!(error["code"], "out_of_range");
    assert_eq!(revision(&host), before);
    let bad = call(&core, &session, "set_pixels", json!({ "name": "slime", "symmetry": "diagonal", "pixels": [] })).unwrap_err();
    assert_eq!(bad["code"], "bad_argument");
}

// Checks symmetry on `draw_shape` and on `apply_grid`.
#[test]
fn symmetry_works_for_shapes_and_grids() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "draw_shape", json!({
        "name": "slime", "shape": "line", "key": "R", "x0": 0, "y0": 0, "x1": 2, "y1": 0, "symmetry": "horizontal",
    })).unwrap();
    assert!(rows(&host, &[], "slime", 1).starts_with("RRR..RRR"));

    call(&core, &session, "apply_grid", json!({
        "name": "slime", "grid": "GG\n", "frame": 1, "x": 0, "y": 7, "symmetry": "both",
    })).unwrap();
    let all = rows(&host, &[], "slime", 1);
    let all: Vec<&str> = all.split('/').collect();
    assert_eq!((all[0], all[7]), ("GGR..RGG", "GG....GG"));
}

// Checks that the old calls (no new argument) give the same result as before.
#[test]
fn calls_without_the_new_arguments_are_unchanged() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let done = call(&core, &session, "draw_shape", json!({
        "name": "slime", "shape": "rectangle", "filled": true, "key": "R", "x": 0, "y": 0, "width": 2, "height": 2,
    })).unwrap();

    assert_eq!(done["pixels_changed"], 4);
    assert_eq!(rows(&host, &[], "slime", 1), format!("RR....../RR....../{}", &BLANK8[18..]));
}
