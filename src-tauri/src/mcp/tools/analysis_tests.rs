//! Spec `agent-analysis-tools`: the tools `auto_outline`, `auto_shade` and
//! `analyze_sprite` seen from a client.

use crate::ipc::project;
use crate::mcp::Permissions;
use crate::mcp::testing::{call, open_demo_project, revision, rows, setup};
use serde_json::json;

const BLANK8: &str = "......../......../......../......../......../......../......../........";

fn entries(host: &crate::mcp::testing::TestHost) -> usize {
    project::read(&host.project, |s| s.history.entries().len()).unwrap()
}

/// Draws a 2x2 block of G at (3,3) in `slime`, as the user (not an AI action).
fn block(host: &crate::mcp::testing::TestHost, core: &crate::mcp::Core, session: &str) {
    call(core, session, "set_pixels", json!({ "name": "slime", "pixels": [
        { "x": 3, "y": 3, "key": "G" }, { "x": 4, "y": 3, "key": "G" },
        { "x": 3, "y": 4, "key": "G" }, { "x": 4, "y": 4, "key": "G" },
    ] })).unwrap();
    project::edit(&host.project, |s| s.history.clear()).unwrap();
}

// Checks "Outline a square": the 12 pixels around a 2x2 block get the key, the
// block is the same, it is ONE history entry by the client, and the answer has
// the counts and the new revision.
#[test]
fn auto_outline_draws_a_ring_in_one_history_entry() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    block(&host, &core, &session);
    let before = revision(&host);

    let done = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R" })).unwrap();

    assert_eq!(
        rows(&host, &[], "slime", 1),
        "......../......../..RRRR../..RGGR../..RGGR../..RRRR../......../........"
    );
    assert_eq!((done["pixels_changed"].as_u64(), done["clipped"].as_u64()), (Some(12), Some(0)));
    assert_eq!(done["revision"], revision(&host));
    assert_eq!(revision(&host), before + 1);
    assert_eq!(entries(&host), 1);
    let author = project::read(&host.project, |s| s.history.entries()[0].author.clone()).unwrap();
    assert_eq!(author, "Claude Code");
}

// Checks the options reach the logic: 4 neighbors leave the corners out, and
// `inner` recolors the block instead of the transparent pixels.
#[test]
fn auto_outline_options_are_used() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    block(&host, &core, &session);

    let four = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R", "connectivity": 4 })).unwrap();
    assert_eq!(four["pixels_changed"], 8);
    project::edit(&host.project, |s| s.undo().unwrap()).unwrap();

    let inner = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R", "inner": true })).unwrap();
    assert_eq!(inner["pixels_changed"], 4);
    assert_eq!(rows(&host, &[], "slime", 1), "......../......../......../...RR.../...RR.../......../......../........");
}

// Checks that a wrong call changes nothing and says how to fix it: an unknown
// key lists the valid keys, `.` and a bad connectivity are `bad_argument`, a
// missing frame is `bad_frame`, and an old base revision is `stale_revision`.
#[test]
fn auto_outline_errors_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    block(&host, &core, &session);
    let before = revision(&host);
    let shape = rows(&host, &[], "slime", 1);

    let unknown = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "Z" })).unwrap_err();
    assert_eq!(unknown["code"], "unknown_key");
    assert!(unknown["valid_keys"].as_array().unwrap().iter().any(|k| k == "R"));
    let dot = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "." })).unwrap_err();
    assert_eq!(dot["code"], "bad_argument");
    let bad = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R", "connectivity": 5 })).unwrap_err();
    assert_eq!((bad["code"].as_str(), bad["argument"].as_str()), (Some("bad_argument"), Some("connectivity")));
    let frame = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R", "frame": 2 })).unwrap_err();
    assert_eq!(frame["code"], "bad_frame");
    let stale = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R", "base_revision": before - 1 })).unwrap_err();
    assert_eq!(stale["code"], "stale_revision");

    assert_eq!(rows(&host, &[], "slime", 1), shape);
    assert_eq!(revision(&host), before);
    assert_eq!(entries(&host), 0);
}

// Checks that a frame with nothing to outline is not an error and writes
// nothing (the revision stays).
#[test]
fn auto_outline_on_an_empty_frame_changes_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);

    let done = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R" })).unwrap();

    assert_eq!((done["changed"].as_bool(), done["pixels_changed"].as_u64()), (Some(false), Some(0)));
    assert_eq!(revision(&host), before);
    assert_eq!(rows(&host, &[], "slime", 1), BLANK8);
}

// Checks that the tool needs the "Edit pixels" permission.
#[test]
fn auto_outline_needs_the_edit_permission() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    block(&host, &core, &session);
    core.set_permissions(Permissions { edit: false, ..Permissions::default() }).unwrap();

    let error = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R" })).unwrap_err();

    assert_eq!(error["code"], "permission_denied");
    assert_eq!(error["permission"], "edit");
    assert_eq!(entries(&host), 0);
}

/// Adds the colors D (dark) and L (light) to the palette, and draws a filled
/// 6x6 square of G at (1,1) in `slime`, as setup (the history is cleared).
fn shaded_setup(host: &crate::mcp::testing::TestHost, core: &crate::mcp::Core, session: &str) {
    for (key, hex) in [("D", "#004400"), ("L", "#88FF88")] {
        call(core, session, "set_palette_color", json!({ "key": key, "hex": hex })).unwrap();
    }
    let square = json!({ "name": "slime", "shape": "rectangle", "key": "G", "x": 1, "y": 1, "width": 6, "height": 6, "filled": true });
    call(core, session, "draw_shape", square).unwrap();
    project::edit(&host.project, |s| s.history.clear()).unwrap();
}

// Checks "Square lit from the top left": the result is the one of the rule, the
// counts say how many pixels went lighter and darker, and it is ONE history
// entry by the client.
#[test]
fn auto_shade_shades_a_square_in_one_history_entry() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    shaded_setup(&host, &core, &session);
    let before = revision(&host);

    let done = call(&core, &session, "auto_shade", json!({ "name": "slime", "light": "top-left", "base": "G", "keys": ["D", "G", "L"] })).unwrap();

    assert_eq!(
        rows(&host, &[], "slime", 1),
        "......../.LLLLLG./.LGGGGD./.LGGGGD./.LGGGGD./.LGGGGD./.GDDDDD./........"
    );
    assert_eq!((done["pixels_changed"].as_u64(), done["lit"].as_u64(), done["shadow"].as_u64()), (Some(18), Some(9), Some(9)));
    assert_eq!(revision(&host), before + 1);
    assert_eq!(entries(&host), 1);
}

// Checks that the ramp can be a text, and that `region` limits the change to
// the box (here the left half of the square only).
#[test]
fn auto_shade_accepts_a_text_ramp_and_a_region() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    shaded_setup(&host, &core, &session);
    let input = json!({
        "name": "slime", "light": "left", "base": "G", "keys": "DGL",
        "region": { "x": 0, "y": 0, "width": 4, "height": 8 },
    });

    call(&core, &session, "auto_shade", input).unwrap();

    // Only the lit left column is inside the box; the dark right column is not.
    assert_eq!(
        rows(&host, &[], "slime", 1),
        "......../.LGGGGG./.LGGGGG./.LGGGGG./.LGGGGG./.LGGGGG./.LGGGGG./........"
    );
}

// Checks the argument errors: each says how to fix it and nothing changes.
#[test]
fn auto_shade_errors_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    shaded_setup(&host, &core, &session);
    let before = revision(&host);
    let shape = rows(&host, &[], "slime", 1);
    let ok = json!({ "name": "slime", "light": "top", "base": "G", "keys": ["D", "G", "L"] });
    let with = |key: &str, value: serde_json::Value| {
        let mut input = ok.clone();
        input[key] = value;
        input
    };
    let code = |input| call(&core, &session, "auto_shade", input).unwrap_err();

    assert_eq!(code(with("light", json!("up")))["code"], "bad_argument");
    assert!(code(with("light", json!("up")))["message"].as_str().unwrap().contains("top-left"));
    assert_eq!(code(with("base", json!("R")))["code"], "bad_argument", "base is not in the ramp");
    assert_eq!(code(with("keys", json!(["D", "G", "Z"])))["code"], "unknown_key");
    assert_eq!(code(with("keys", json!(["G"])))["code"], "bad_argument");
    assert_eq!(code(with("keys", json!(["D", "G", "D"])))["code"], "bad_argument");
    assert_eq!(code(with("base", json!(".")))["code"], "bad_argument");
    assert_eq!(code(with("frame", json!(3)))["code"], "bad_frame");
    let outside = json!({ "x": 6, "y": 0, "width": 4, "height": 1 });
    assert_eq!(code(with("region", outside))["code"], "out_of_range");
    assert_eq!(code(with("base_revision", json!(before - 1)))["code"], "stale_revision");

    assert_eq!(rows(&host, &[], "slime", 1), shape);
    assert_eq!(revision(&host), before);
    assert_eq!(entries(&host), 0);
}

// Checks that `auto_shade` needs the "Edit pixels" permission.
#[test]
fn auto_shade_needs_the_edit_permission() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    shaded_setup(&host, &core, &session);
    core.set_permissions(Permissions { edit: false, ..Permissions::default() }).unwrap();

    let error = call(&core, &session, "auto_shade", json!({ "name": "slime", "light": "top", "base": "G", "keys": "DGL" })).unwrap_err();

    assert_eq!(error["code"], "permission_denied");
}

/// A goblin (4x4, 2 frames) drawn for the report tests.
fn goblin_frames(core: &crate::mcp::Core, session: &str) {
    let g = |frame: u32, x: u32, y: u32, key: &str| json!({ "name": "goblin", "frame": frame, "pixels": [{ "x": x, "y": y, "key": key }] });
    // Frame 1 has (0,0) R and (3,3) G. Frame 2 gets a block, a loose pixel and a floor.
    for (frame, x, y, key) in [(2, 0, 3, "G"), (2, 1, 3, "G"), (2, 1, 2, "G"), (2, 3, 0, "G")] {
        call(core, session, "set_pixels", g(frame, x, y, key)).unwrap();
    }
}

// Checks the report of a two frame sprite: colors, loose pixels per frame, the
// comparison of the frames, and that the call changes nothing and is not logged.
#[test]
fn analyze_sprite_reports_loose_pixels_colors_and_changes() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    goblin_frames(&core, &session);
    project::edit(&host.project, |s| s.history.clear()).unwrap();
    let before = (revision(&host), core.activity().len());

    let report = call(&core, &session, "analyze_sprite", json!({ "name": "goblin" })).unwrap();

    assert_eq!(report["sprite"], "Characters/goblin");
    assert_eq!((report["width"].as_u64(), report["height"].as_u64(), report["frames"].as_u64()), (Some(4), Some(4), Some(2)));
    assert_eq!(report["colors"], json!({ "count": 2, "used": ["R", "G"], "unused": [] }));
    // Frame 1: R at (0,0) and G at (3,3) touch nothing.
    assert_eq!(report["frame_reports"][0]["loose_pixels"], json!({ "count": 2, "at": [[0, 0], [3, 3]] }));
    assert!(report["frame_reports"][0]["outline_gaps"].is_null(), "no outline_key, no gaps");
    // Frame 2: only (3,0) is alone (the R of frame 1 is gone; (0,3),(1,3),(1,2) touch).
    assert_eq!(report["frame_reports"][1]["loose_pixels"], json!({ "count": 1, "at": [[3, 0]] }));
    let change = &report["changes"][0];
    assert_eq!((change["from"].as_u64(), change["to"].as_u64()), (Some(1), Some(2)));
    assert_eq!(change["changed_pixels"], 6);
    assert_eq!(change["bbox"], json!({ "x": 0, "y": 0, "width": 4, "height": 4 }));
    assert_eq!(change["ground_kept"], false, "the bottom rows changed");
    assert_eq!((revision(&host), core.activity().len()), before);
    assert_eq!(entries(&host), 0);
}

// Checks the unused colors, the outline gaps and `ground_rows`, and that a
// sprite with one frame has no `changes`.
#[test]
fn analyze_sprite_finds_gaps_unused_colors_and_has_no_changes_for_one_frame() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    // slime: a 3x3 block of G with an R outline that misses the pixel at (1,2).
    call(&core, &session, "draw_shape", json!({ "name": "slime", "shape": "rectangle", "key": "G", "x": 2, "y": 2, "width": 3, "height": 3, "filled": true })).unwrap();
    call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R" })).unwrap();
    call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 1, "y": 3, "key": "." }] })).unwrap();

    let report = call(&core, &session, "analyze_sprite", json!({ "name": "slime", "outline_key": "R" })).unwrap();

    assert_eq!(report["frame_reports"][0]["outline_gaps"], json!({ "count": 1, "at": [[2, 3]] }));
    assert_eq!(report["colors"]["unused"], json!([]));
    assert!(report["changes"].is_null());
    let clean = call(&core, &session, "analyze_sprite", json!({ "name": "slime" })).unwrap();
    assert_eq!(clean["frame_reports"][0]["loose_pixels"]["count"], 0);
    let error = call(&core, &session, "analyze_sprite", json!({ "name": "slime", "outline_key": "Z" })).unwrap_err();
    assert_eq!(error["code"], "unknown_key");
    let none = call(&core, &session, "analyze_sprite", json!({ "name": "nobody" })).unwrap_err();
    assert_eq!(none["code"], "not_found");
}

// Checks that lists are cut at 50 coordinates while `count` stays complete.
#[test]
fn analyze_sprite_cuts_long_lists() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "create_sprite", json!({ "name": "dots", "width": 16, "height": 16 })).unwrap();
    // Every other pixel of the even rows: 8 x 8 = 64 loose pixels.
    let pixels: Vec<_> = (0..16).step_by(2).flat_map(|y| (0..16).step_by(2).map(move |x| json!({ "x": x, "y": y, "key": "R" }))).collect();
    call(&core, &session, "set_pixels", json!({ "name": "dots", "pixels": pixels })).unwrap();

    let report = call(&core, &session, "analyze_sprite", json!({ "name": "dots" })).unwrap();

    assert_eq!(report["frame_reports"][0]["loose_pixels"]["count"], 64);
    assert_eq!(report["frame_reports"][0]["loose_pixels"]["at"].as_array().unwrap().len(), 50);
}
