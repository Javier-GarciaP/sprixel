//! Spec `agent-live-design`, task 2.3: the pixel write tools return an image of
//! the edited frame.

use crate::mcp::testing::{TestHost, core_with, connect_allowed, open_demo_project, revision};
use crate::mcp::{Core, Ctx, Output, tools};
use serde_json::{Value, json};

/// Runs a tool and gives back its raw output (the image and the JSON).
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

/// The image and the info of a result that must have both.
fn image_of(output: Output) -> (image::RgbaImage, Value) {
    match output {
        Output::Image { png, info } => (image::load_from_memory(&png).unwrap().to_rgba8(), info),
        Output::Json(json) => panic!("no image in {json}"),
    }
}

// Checks the "Preview after a write" scenario for `set_pixels`: the result has
// a PNG of the edited frame (the new pixel is in it) and, next to it, the
// usual answer with the new revision.
#[test]
fn set_pixels_returns_the_edited_frame() {
    let (host, core) = setup();

    let output = run(&core, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "R" }] })).unwrap();

    let (image, info) = image_of(output);
    // An 8x8 sprite is shown at scale 32 (256 pixels), and the top left pixel is red.
    assert_eq!((image.width(), image.height()), (256, 256));
    assert_eq!(image.get_pixel(0, 0).0, [255, 0, 0, 255]);
    assert_eq!(info["revision"], revision(&host));
    assert_eq!(info["pixels_set"], 1);
    assert_eq!((info["name"].as_str(), info["frame"].as_u64()), (Some("slime"), Some(1)));
}

// Checks that `apply_grid` and `draw_shape` also return the image, of the frame
// that was written (frame 2 of the goblin here).
#[test]
fn apply_grid_and_draw_shape_return_the_edited_frame() {
    let (_host, core) = setup();

    let grid = run(&core, "apply_grid", json!({ "name": "goblin", "frame": 2, "grid": "GGGG\n....\n....\n....\n" })).unwrap();
    let (image, info) = image_of(grid);
    assert_eq!(info["frame"], 2);
    assert_eq!(image.get_pixel(0, 0).0, [0, 255, 0, 255]);

    let shape = run(&core, "draw_shape", json!({ "name": "goblin", "frame": 2, "shape": "line", "key": "R", "x0": 0, "y0": 3, "x1": 3, "y1": 3 })).unwrap();
    let (image, info) = image_of(shape);
    assert_eq!(info["frame"], 2);
    let last_row = image.height() - 1;
    assert_eq!(image.get_pixel(0, last_row).0, [255, 0, 0, 255]);
}

// Checks the "Preview off" scenario: with `preview` false the result is plain
// JSON, the write still happened and the answer is the same.
#[test]
fn preview_false_returns_no_image() {
    let (host, core) = setup();
    let before = revision(&host);

    for (tool, arguments) in [
        ("set_pixels", json!({ "name": "slime", "preview": false, "pixels": [{ "x": 1, "y": 1, "key": "R" }] })),
        ("draw_shape", json!({ "name": "slime", "preview": false, "shape": "line", "key": "G", "x0": 0, "y0": 5, "x1": 3, "y1": 5 })),
        ("apply_grid", json!({ "name": "slime", "preview": false, "grid": "frame 1\nR.......\n........\n........\n........\n........\n........\n........\n........\n" })),
    ] {
        match run(&core, tool, arguments).unwrap() {
            Output::Json(answer) => assert!(answer["revision"].as_u64().unwrap() > before, "{tool}"),
            Output::Image { .. } => panic!("{tool} returned an image"),
        }
    }
}

// Checks the size cap: the image is at most 512 pixels on a side for any
// sprite size (a small sprite is scaled up, a big one is not), and a
// sprite of 512 is shown at its own size.
#[test]
fn the_image_is_never_bigger_than_512() {
    let (_host, core) = setup();
    for (side, expected) in [(1, 32), (16, 512), (40, 480), (100, 500), (300, 300), (512, 512)] {
        let name = format!("s{side}");
        run(&core, "create_sprite", json!({ "name": name, "width": side, "height": side })).unwrap();
        let output = run(&core, "set_pixels", json!({ "name": name, "pixels": [{ "x": 0, "y": 0, "key": "R" }] })).unwrap();
        let (image, _) = image_of(output);
        assert_eq!(image.width(), expected, "a sprite of {side}");
        assert!(image.width() <= 512 && image.height() <= 512);
    }
}

// Checks that a failed write has no image (it is an error as before) and
// changes nothing.
#[test]
fn a_failed_write_has_no_image() {
    let (host, core) = setup();
    let before = revision(&host);

    let error = run(&core, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "Z" }] })).unwrap_err();

    assert_eq!(error.code, "unknown_key");
    assert_eq!(revision(&host), before);
}

// Checks the MCP shape of the result: the image comes first, then the text with
// the JSON answer, so a client that shows images shows this one, and a client
// that reads the text still gets the revision.
#[test]
fn the_mcp_result_has_the_image_and_the_text() {
    let (host, core) = core_with(tools::all());
    let session = connect_allowed(&core, "Claude Code");
    open_demo_project(&host);

    let reply = crate::mcp::testing::rpc(
        &core,
        Some(&session),
        "tools/call",
        json!({ "name": "set_pixels", "arguments": { "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "R" }] } }),
    );

    let content = reply["result"]["content"].as_array().unwrap();
    assert_eq!(content[0]["type"], "image");
    assert_eq!(content[0]["mimeType"], "image/png");
    let text: Value = serde_json::from_str(content[1]["text"].as_str().unwrap()).unwrap();
    assert_eq!(text["revision"], revision(&host));
}

// Checks that the activity log still knows the action changed the project, so
// the Agents tab keeps offering Undo for a write that came with an image.
#[test]
fn the_activity_log_still_offers_undo() {
    let (host, core) = core_with(tools::all());
    let session = connect_allowed(&core, "Claude Code");
    open_demo_project(&host);
    crate::mcp::testing::call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "R" }] })).unwrap();
    crate::mcp::testing::replay_done(&host);

    let rows = core.activity();

    assert_eq!(rows[0].entry.tool, "set_pixels");
    assert!(rows[0].can_undo);
    assert_eq!(rows[0].entry.sprite.as_deref(), Some("slime"));
}
