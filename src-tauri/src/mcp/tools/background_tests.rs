//! Spec `agent-previews`, task 1.2: the `background` option of the previews.

use crate::mcp::testing::{TestHost, connect_allowed, core_with, open_demo_project, revision};
use crate::mcp::{Core, Ctx, Output, tools};
use serde_json::{Value, json};

fn run(core: &Core, tool: &str, arguments: Value) -> Result<Output, crate::mcp::ToolError> {
    let tool = core.tools.iter().find(|t| t.name == tool).unwrap();
    (tool.run)(&Ctx { core, client: "Claude Code", tool: tool.name }, &arguments)
}

fn setup() -> (std::sync::Arc<TestHost>, Core) {
    let (host, core) = core_with(tools::all());
    connect_allowed(&core, "Claude Code");
    open_demo_project(&host);
    (host, core)
}

fn image_of(output: Output) -> image::RgbaImage {
    match output {
        Output::Image { png, .. } => image::load_from_memory(&png).unwrap().to_rgba8(),
        Output::Json(json) => panic!("no image in {json}"),
    }
}

/// The pixel (x, y) of the `slime` preview (all transparent, scale 1).
fn slime_pixel(core: &Core, background: Option<&str>, x: u32, y: u32) -> [u8; 4] {
    let mut args = json!({ "name": "slime", "scale": 1 });
    if let Some(background) = background {
        args["background"] = json!(background);
    }
    image_of(run(core, "render_preview", args).unwrap()).get_pixel(x, y).0
}

// Checks the default: an empty (transparent) pixel is drawn as a gray of the
// checkerboard, not as white and not as nothing. So a white sprite stands out.
#[test]
fn the_default_background_is_a_gray_checker() {
    let (_host, core) = setup();
    let pixel = slime_pixel(&core, None, 0, 0);
    assert_eq!(pixel[3], 255);
    assert!(pixel[0] < 255 && pixel[0] == pixel[1] && pixel[1] == pixel[2], "{pixel:?}");
    assert_eq!(pixel, slime_pixel(&core, Some("checker"), 0, 0));
}

// Checks the other options: light is white, dark is dark gray, transparent
// keeps alpha 0.
#[test]
fn light_dark_and_transparent_backgrounds() {
    let (_host, core) = setup();
    assert_eq!(slime_pixel(&core, Some("light"), 0, 0), [255, 255, 255, 255]);
    assert_eq!(slime_pixel(&core, Some("dark"), 0, 0), [0x20, 0x20, 0x20, 255]);
    assert_eq!(slime_pixel(&core, Some("transparent"), 0, 0)[3], 0);
}

// Checks that a drawn pixel is never touched by the background.
#[test]
fn drawn_pixels_keep_their_color() {
    let (_host, core) = setup();
    let image = image_of(run(&core, "render_preview", json!({ "name": "goblin", "folder": "Characters", "scale": 1 })).unwrap());
    assert_eq!(image.get_pixel(0, 0).0, [255, 0, 0, 255]);
}

// Checks that a wrong value fails with `bad_argument` in the read tool and in
// the three write tools, and that the write changes nothing.
#[test]
fn an_unknown_background_is_bad_argument_and_changes_nothing() {
    let (host, core) = setup();
    let before = revision(&host);
    let error = run(&core, "render_preview", json!({ "name": "slime", "background": "purple" })).unwrap_err();
    assert_eq!(error.code, "bad_argument");

    for (tool, arguments) in [
        ("set_pixels", json!({ "name": "slime", "background": "purple", "pixels": [{ "x": 1, "y": 1, "key": "R" }] })),
        ("draw_shape", json!({ "name": "slime", "background": "purple", "shape": "line", "key": "G", "x0": 0, "y0": 5, "x1": 3, "y1": 5 })),
        ("apply_grid", json!({ "name": "slime", "background": "purple", "grid": "frame 1\nR.......\n........\n........\n........\n........\n........\n........\n........\n" })),
    ] {
        assert_eq!(run(&core, tool, arguments).unwrap_err().code, "bad_argument", "{tool}");
    }
    assert_eq!(revision(&host), before);
}

// Checks the write tools: the image after a write is over the checker by
// default and has alpha 0 with `transparent`.
#[test]
fn write_previews_use_the_background() {
    let (_host, core) = setup();
    let pixels = |background: &str| {
        json!({ "name": "slime", "background": background, "pixels": [{ "x": 0, "y": 0, "key": "R" }] })
    };
    // The last pixel of the 8x8 sprite (scale 32) is empty.
    let checker = image_of(run(&core, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "R" }] })).unwrap());
    assert_eq!(checker.get_pixel(255, 255).0[3], 255);
    let clear = image_of(run(&core, "set_pixels", pixels("transparent")).unwrap());
    assert_eq!(clear.get_pixel(255, 255).0[3], 0);
    let light = image_of(run(&core, "set_pixels", pixels("light")).unwrap());
    assert_eq!(light.get_pixel(255, 255).0, [255, 255, 255, 255]);
}
