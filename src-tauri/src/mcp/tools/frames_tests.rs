//! Spec `agent-previews`, tasks 2.2 and 2.3: `render_frames` and `render_onion`.

use crate::mcp::testing::{TestHost, connect_allowed, core_with, open_demo_project, revision};
use crate::mcp::{Core, Ctx, Output, tools};
use serde_json::{Value, json};

fn run(core: &Core, tool: &str, arguments: Value) -> Result<Output, crate::mcp::ToolError> {
    let tool = core.tools.iter().find(|t| t.name == tool).unwrap();
    (tool.run)(&Ctx { core, client: "Claude Code", tool: tool.name }, &arguments)
}

/// The demo project plus a sprite `walk` of 4x4 pixels and 4 frames.
fn setup() -> (std::sync::Arc<TestHost>, Core) {
    let (host, core) = core_with(tools::all());
    connect_allowed(&core, "Claude Code");
    open_demo_project(&host);
    run(&core, "create_sprite", json!({ "name": "walk", "width": 4, "height": 4 })).unwrap();
    for _ in 0..3 {
        run(&core, "add_frame", json!({ "name": "walk" })).unwrap();
    }
    (host, core)
}

fn image_of(output: Output) -> (image::RgbaImage, Value) {
    match output {
        Output::Image { png, info } => (image::load_from_memory(&png).unwrap().to_rgba8(), info),
        Output::Json(json) => panic!("no image in {json}"),
    }
}

/// Puts one pixel of key R at (x, 0) in a frame of `walk`.
fn dot(core: &Core, frame: u32, x: u32) {
    run(core, "set_pixels", json!({ "name": "walk", "frame": frame, "preview": false, "pixels": [{ "x": x, "y": 0, "key": "R" }] })).unwrap();
}

// Checks the grid: 4 frames of 4x4 at scale 2 in 2 columns are 2*8 + 2 = 18
// pixels on each side (a gap of one sprite pixel), and the info says so.
#[test]
fn all_frames_in_a_grid() {
    let (_host, core) = setup();
    dot(&core, 2, 0);

    let (image, info) = image_of(run(&core, "render_frames", json!({ "name": "walk", "columns": 2, "scale": 2, "background": "transparent" })).unwrap());

    assert_eq!((image.width(), image.height()), (18, 18));
    assert_eq!((info["columns"].as_u64(), info["from"].as_u64(), info["to"].as_u64()), (Some(2), Some(1), Some(4)));
    // Frame 2 is the second cell of the first row: its red pixel is at x = 8 + 2.
    assert_eq!(image.get_pixel(10, 0).0, [255, 0, 0, 255]);
    assert_eq!(image.get_pixel(0, 0).0[3], 0);
}

// Checks `from` and `to`: only those frames are drawn (here frames 2 and 3,
// in one row by default columns of 2).
#[test]
fn a_range_of_frames() {
    let (_host, core) = setup();
    dot(&core, 3, 0);

    let (image, info) = image_of(run(&core, "render_frames", json!({ "name": "walk", "from": 2, "to": 3, "scale": 1, "background": "transparent" })).unwrap());

    assert_eq!((image.width(), image.height()), (9, 4));
    assert_eq!((info["from"].as_u64(), info["to"].as_u64()), (Some(2), Some(3)));
    assert_eq!(image.get_pixel(5, 0).0, [255, 0, 0, 255]); // the second cell
}

// Checks the defaults: columns about a square (4 frames -> 2) and the largest
// scale from 1 to 8 (here 8: 2*32 + 8 = 72 pixels).
#[test]
fn default_columns_and_scale() {
    let (_host, core) = setup();
    let (image, info) = image_of(run(&core, "render_frames", json!({ "name": "walk" })).unwrap());
    assert_eq!((info["columns"].as_u64(), info["scale"].as_u64()), (Some(2), Some(8)));
    assert_eq!((image.width(), image.height()), (72, 72));
}

// Checks labels: a band above each frame makes the image taller; the default
// has none.
#[test]
fn labels_make_the_image_taller() {
    let (_host, core) = setup();
    let plain = image_of(run(&core, "render_frames", json!({ "name": "walk", "scale": 2 })).unwrap()).0;
    let labeled = image_of(run(&core, "render_frames", json!({ "name": "walk", "scale": 2, "labels": true })).unwrap()).0;
    assert_eq!(labeled.width(), plain.width());
    assert!(labeled.height() > plain.height());
}

// Checks the size cap: too big says `too_big` with the scale that fits, and
// that scale really works.
#[test]
fn too_big_gives_a_scale_that_fits() {
    let (_host, core) = setup();
    let error = run(&core, "render_frames", json!({ "name": "walk", "columns": 4, "scale": 64 })).unwrap_err();
    assert_eq!(error.code, "too_big");
    let fits = error.details["max_scale"].as_u64().unwrap();
    assert!(fits >= 1 && fits < 64);
    let (image, _) = image_of(run(&core, "render_frames", json!({ "name": "walk", "columns": 4, "scale": fits })).unwrap());
    assert!(image.width() <= 1024 && image.height() <= 1024);
}

// Checks the errors: a frame that does not exist, a range upside down, a bad
// background and a bad column count. None changes the project.
#[test]
fn bad_arguments() {
    let (host, core) = setup();
    let before = revision(&host);
    let code = |arguments: Value| run(&core, "render_frames", arguments).unwrap_err().code;
    assert_eq!(code(json!({ "name": "walk", "to": 9 })), "bad_frame");
    assert_eq!(code(json!({ "name": "walk", "from": 0 })), "bad_frame");
    assert_eq!(code(json!({ "name": "walk", "from": 3, "to": 2 })), "bad_argument");
    assert_eq!(code(json!({ "name": "walk", "background": "purple" })), "bad_argument");
    assert_eq!(code(json!({ "name": "walk", "columns": 0 })), "bad_argument");
    assert_eq!(code(json!({ "name": "nope" })), "not_found");
    assert_eq!(revision(&host), before);
}
