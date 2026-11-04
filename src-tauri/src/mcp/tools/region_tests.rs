//! Tasks 3.2 and 3.3: `transform_region` and `copy_region`.

use crate::ipc::project;
use crate::mcp::testing::{TestHost, call, open_demo_project, revision, rows, setup};
use serde_json::json;

fn entries(host: &TestHost) -> usize {
    project::read(&host.project, |s| s.history.entries().len()).unwrap()
}

/// Draws a picture on frame 1 of `slime` (8x8).
fn draw(core: &crate::mcp::Core, session: &str) {
    let grid = "frame 1\nRRG.....\nR.......\n........\n........\n........\n........\n........\n.......G\n";
    call(core, session, "apply_grid", json!({ "name": "slime", "grid": grid })).unwrap();
}

/// The first `count` rows of frame 1 of `slime`.
fn top(host: &TestHost, count: usize) -> Vec<String> {
    rows(host, &[], "slime", 1).split('/').take(count).map(str::to_string).collect()
}

fn transform(core: &crate::mcp::Core, session: &str, extra: serde_json::Value) -> Result<serde_json::Value, serde_json::Value> {
    let mut args = json!({ "name": "slime" });
    args.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    call(core, session, "transform_region", args)
}

// Checks the "Flip" scenario: the rows of the region are reversed and the
// pixels outside the region stay.
#[test]
fn flip_reverses_the_region_only() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session);

    transform(&core, &session, json!({
        "operation": "flip_horizontal", "region": { "x": 0, "y": 0, "width": 4, "height": 2 },
    })).unwrap();

    assert_eq!(top(&host, 2), [".GRR....", "...R...."]);
    assert!(rows(&host, &[], "slime", 1).ends_with(".......G"), "outside the region nothing changed");
}

// Checks the whole frame is the default region, and the vertical flip.
#[test]
fn flip_vertical_works_on_the_whole_frame_by_default() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session);

    transform(&core, &session, json!({ "operation": "flip_vertical" })).unwrap();

    assert_eq!(top(&host, 1), [".......G"]);
    assert!(rows(&host, &[], "slime", 1).ends_with("R......./RRG....."));
}

// Checks the "Mirror a half" scenario: columns 4 to 7 become columns 3 to 0.
#[test]
fn mirror_copies_the_left_half_to_the_right() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session);

    transform(&core, &session, json!({ "operation": "mirror", "direction": "left_to_right" })).unwrap();

    assert_eq!(top(&host, 2), ["RRG..GRR", "R......R"]);
}

// Checks the move scenarios: without wrap pixels are lost, with wrap they
// come back, and a negative move goes left.
#[test]
fn move_can_lose_or_wrap_pixels() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session);

    transform(&core, &session, json!({ "operation": "move", "dx": 6, "dy": 0 })).unwrap();
    assert_eq!(top(&host, 2), ["......RR", "......R."], "G and the rest left the frame");

    draw(&core, &session);
    transform(&core, &session, json!({ "operation": "move", "dx": 7, "dy": 0, "wrap": true })).unwrap();
    assert_eq!(top(&host, 2), ["RG.....R", ".......R"]);
    assert!(rows(&host, &[], "slime", 1).ends_with("......G."));
}

// Checks the rotate scenarios: a square region turns clockwise, a region that
// is not square is refused with `bad_argument` and nothing changes.
#[test]
fn rotate_turns_a_square_and_refuses_a_rectangle() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session);

    transform(&core, &session, json!({
        "operation": "rotate", "degrees": 90, "region": { "x": 0, "y": 0, "width": 2, "height": 2 },
    })).unwrap();
    assert_eq!(top(&host, 2), ["RRG.....", ".R......"]);

    let before = revision(&host);
    let error = transform(&core, &session, json!({
        "operation": "rotate", "degrees": 90, "region": { "x": 0, "y": 0, "width": 3, "height": 2 },
    })).unwrap_err();
    assert_eq!(error["code"], "bad_argument");
    assert_eq!(revision(&host), before);
}

// Checks the "Region out of the sprite" scenario, a missing argument and an
// unknown operation: each one changes nothing.
#[test]
fn bad_regions_and_arguments_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session);
    let before = revision(&host);
    let code = |extra| transform(&core, &session, extra).unwrap_err()["code"].as_str().unwrap().to_string();

    assert_eq!(code(json!({ "operation": "flip_horizontal", "region": { "x": 6, "y": 0, "width": 4, "height": 1 } })), "region_out");
    assert_eq!(code(json!({ "operation": "mirror" })), "bad_argument");
    assert_eq!(code(json!({ "operation": "move", "dx": 1 })), "bad_argument");
    assert_eq!(code(json!({ "operation": "spin" })), "bad_argument");
    assert_eq!(code(json!({ "operation": "flip_vertical", "frame": 5 })), "bad_frame");
    assert_eq!(revision(&host), before);
}

// Checks that a transform is ONE history entry by the client, that undo takes
// it back, that the UI is told with the pixels to replay, and that the answer
// has a preview image. A call that changes nothing is not an entry.
#[test]
fn a_transform_is_one_undoable_entry_with_replay_and_preview() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session);
    let entries_before = entries(&host);

    let done = transform(&core, &session, json!({ "operation": "flip_horizontal" })).unwrap();

    assert_eq!(done["changed"], true);
    assert_eq!(entries(&host), entries_before + 1);
    assert_eq!(done["revision"], revision(&host));
    let events = host.events.lock().unwrap();
    let replay = events.iter().rev().find(|(e, _)| e == super::write::OPS_EVENT).expect("a replay event");
    assert_eq!(replay.1["tool"], "transform_region");
    assert!(!replay.1["pixels"].as_array().unwrap().is_empty());
    drop(events);
    project::edit(&host.project, |s| s.undo().unwrap()).unwrap();
    assert_eq!(top(&host, 1), ["RRG....."]);

    let empty = transform(&core, &session, json!({ "operation": "move", "dx": 0, "dy": 0 })).unwrap();
    assert_eq!(empty["changed"], false);
    assert_eq!(entries(&host), entries_before, "undo removed one entry and the no-op added none");
}

/// Makes the 4x4 sprite `goblin` (frame 1: R top left, G bottom right) and
/// a second palette user: `Characters/goblin` has the palette `main`.
fn copy(core: &crate::mcp::Core, session: &str, extra: serde_json::Value) -> Result<serde_json::Value, serde_json::Value> {
    let mut args = json!({ "name": "slime" });
    args.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    call(core, session, "copy_region", args)
}

// Checks the "Copy to another frame" scenario: frame 2 gets the pixels at
// `to_x`, `to_y` and frame 1 stays.
#[test]
fn copy_to_another_frame_leaves_the_source() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session);
    call(&core, &session, "add_frame", json!({ "name": "slime" })).unwrap();

    let done = copy(&core, &session, json!({
        "region": { "x": 0, "y": 0, "width": 3, "height": 2 }, "to_frame": 2, "to_x": 4, "to_y": 5,
    })).unwrap();

    assert_eq!(done["pixels_changed"], 4);
    let two = rows(&host, &[], "slime", 2);
    assert!(two.ends_with("....RRG./....R.../........"), "{two}");
    assert_eq!(top(&host, 2), ["RRG.....", "R......."]);
}

// Checks the "Skip transparent" scenario, and a copy inside the same frame
// that overlaps its source.
#[test]
fn copy_can_skip_transparent_and_overlap() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session);

    copy(&core, &session, json!({ "region": { "x": 0, "y": 0, "width": 3, "height": 1 }, "to_x": 1, "to_y": 1, "skip_transparent": true })).unwrap();
    assert_eq!(top(&host, 2), ["RRG.....", "RRRG...."]);

    copy(&core, &session, json!({ "region": { "x": 0, "y": 0, "width": 3, "height": 1 }, "to_x": 1, "to_y": 0 })).unwrap();
    assert_eq!(top(&host, 1), ["RRRG...."], "the source was read before the write");
}

// Checks a copy to another sprite, and the `palette_mismatch` and
// `region_out` errors, which change nothing.
#[test]
fn copy_to_another_sprite_checks_palette_and_size() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session);

    let done = copy(&core, &session, json!({
        "region": { "x": 0, "y": 0, "width": 2, "height": 2 }, "to_sprite": "goblin", "to_folder": "Characters", "to_x": 2, "to_y": 2,
    })).unwrap();
    assert_eq!(done["sprite"], "Characters/goblin");
    assert_eq!(rows(&host, &["Characters"], "goblin", 1), "R.../..../..RR/..R.", "the transparent source pixel erased the G");

    let before = revision(&host);
    let too_big = copy(&core, &session, json!({ "to_sprite": "goblin", "to_folder": "Characters" })).unwrap_err();
    assert_eq!(too_big["code"], "region_out");
    assert_eq!(revision(&host), before);

    project::edit(&host.project, |s| {
        let palette = crate::document::Palette { name: "other".into(), colors: vec![crate::document::Color { key: 'R', name: "red".into(), hex: "#FF0000".into() }] };
        s.apply("user", vec![
            crate::document::Operation::CreatePalette { palette, index: None },
            crate::document::Operation::CreateSprite { folder: vec![], name: "alien".into(), width: 8, height: 8, palette: "other".into() },
        ]).unwrap();
    }).unwrap();
    let mismatch = copy(&core, &session, json!({ "to_sprite": "alien" })).unwrap_err();
    assert_eq!(mismatch["code"], "palette_mismatch");
    assert_eq!(copy(&core, &session, json!({ "to_sprite": "nobody" })).unwrap_err()["code"], "not_found");
}
