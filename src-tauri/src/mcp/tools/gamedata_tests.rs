//! Spec `game-data`: frame durations, anchors and seamless tiles.

use crate::ipc::project;
use crate::mcp::testing::{call, open_demo_project, revision, setup};
use serde_json::json;

fn durations(host: &crate::mcp::testing::TestHost) -> Vec<u32> {
    project::read(&host.project, |s| {
        let t = crate::document::SpriteRef::new(&["Characters"], "goblin");
        s.project.find_sprite(&t).unwrap().frame_durations.clone()
    })
    .unwrap()
}

// Checks that `set_frame_duration` changes one frame, `get_sprite` shows the
// durations, and the person can undo it (it is one history entry).
#[test]
fn set_frame_duration_changes_one_frame_and_is_undoable() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);

    let done = call(&core, &session, "set_frame_duration", json!({ "name": "goblin", "frame": 2, "milliseconds": 250 })).unwrap();

    assert_eq!(done["frame"], 2);
    assert_eq!(done["milliseconds"], 250);
    assert_eq!(done["revision"], before + 1);
    assert_eq!(durations(&host), vec![100, 250]);
    let sprite = call(&core, &session, "get_sprite", json!({ "name": "goblin" })).unwrap();
    assert_eq!(sprite["frame_durations"], json!([100, 250]));

    call(&core, &session, "undo_last_ai_action", json!({})).unwrap();
    assert_eq!(durations(&host), vec![100, 100]);
}

// Checks the limits: 10 to 10000 ms and an existing frame. A failed call
// changes nothing.
#[test]
fn set_frame_duration_checks_its_arguments() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    for ms in [5, 9, 10001, 20000] {
        let error = call(&core, &session, "set_frame_duration", json!({ "name": "goblin", "frame": 1, "milliseconds": ms })).unwrap_err();
        assert_eq!(error["code"], "bad_argument", "{ms}");
    }
    let error = call(&core, &session, "set_frame_duration", json!({ "name": "goblin", "frame": 9, "milliseconds": 50 })).unwrap_err();
    assert_eq!(error["code"], "bad_frame");
    assert_eq!(revision(&host), before);
    // The limits themselves are valid.
    call(&core, &session, "set_frame_duration", json!({ "name": "goblin", "frame": 1, "milliseconds": 10 })).unwrap();
    call(&core, &session, "set_frame_duration", json!({ "name": "goblin", "frame": 1, "milliseconds": 10000 })).unwrap();
}

fn anchors_of(host: &crate::mcp::testing::TestHost) -> Vec<(String, u32, u32, Option<u32>)> {
    project::read(&host.project, |s| {
        let t = crate::document::SpriteRef::new(&["Characters"], "goblin");
        s.project.find_sprite(&t).unwrap().anchors.iter().map(|a| (a.name.clone(), a.x, a.y, a.frame)).collect()
    })
    .unwrap()
}

// Checks the "Set and read" scenario: `set_anchor` adds `feet`, `get_sprite`
// lists it (and the one of a frame), setting it again moves it, and undo
// removes the last change.
#[test]
fn set_anchor_is_listed_by_get_sprite_and_can_be_undone() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let done = call(&core, &session, "set_anchor", json!({ "sprite": "goblin", "name": "feet", "x": 2, "y": 3 })).unwrap();
    assert_eq!(done["anchor"], "feet");
    assert_eq!(done["revision"], revision(&host));
    call(&core, &session, "set_anchor", json!({ "sprite": "goblin", "name": "hand", "x": 0, "y": 1, "frame": 2 })).unwrap();

    let sprite = call(&core, &session, "get_sprite", json!({ "name": "goblin" })).unwrap();
    assert_eq!(sprite["anchors"], json!([
        { "name": "feet", "x": 2, "y": 3 },
        { "name": "hand", "x": 0, "y": 1, "frame": 2 },
    ]));

    call(&core, &session, "set_anchor", json!({ "sprite": "goblin", "name": "feet", "x": 0, "y": 0 })).unwrap();
    assert_eq!(anchors_of(&host)[0], ("feet".to_string(), 0, 0, None));
    call(&core, &session, "undo_last_ai_action", json!({})).unwrap();
    assert_eq!(anchors_of(&host)[0], ("feet".to_string(), 2, 3, None));
}

// Checks the errors: outside the sprite is `bad_argument`, a missing frame is
// `bad_frame`; removing what does not exist is `not_found`. Nothing changes.
#[test]
fn anchor_errors() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    for (x, y) in [(4, 0), (0, 4)] {
        let error = call(&core, &session, "set_anchor", json!({ "sprite": "goblin", "name": "a", "x": x, "y": y })).unwrap_err();
        assert_eq!(error["code"], "bad_argument");
    }
    let error = call(&core, &session, "set_anchor", json!({ "sprite": "goblin", "name": "a", "x": 0, "y": 0, "frame": 3 })).unwrap_err();
    assert_eq!(error["code"], "bad_frame");
    let error = call(&core, &session, "set_anchor", json!({ "sprite": "goblin", "name": " ", "x": 0, "y": 0 })).unwrap_err();
    assert_eq!(error["code"], "bad_argument");
    let error = call(&core, &session, "remove_anchor", json!({ "sprite": "goblin", "name": "nope" })).unwrap_err();
    assert_eq!(error["code"], "not_found");
    assert_eq!(revision(&host), before);
}

// Checks `remove_anchor`: it removes the anchor of that frame only, and the
// call is one undoable history entry.
#[test]
fn remove_anchor_removes_one_and_is_undoable() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "set_anchor", json!({ "sprite": "goblin", "name": "hand", "x": 1, "y": 1 })).unwrap();
    call(&core, &session, "set_anchor", json!({ "sprite": "goblin", "name": "hand", "x": 2, "y": 2, "frame": 1 })).unwrap();

    let done = call(&core, &session, "remove_anchor", json!({ "sprite": "goblin", "name": "HAND", "frame": 1 })).unwrap();
    assert_eq!(done["removed"], "HAND");
    assert_eq!(anchors_of(&host), vec![("hand".to_string(), 1, 1, None)]);
    call(&core, &session, "undo_last_ai_action", json!({})).unwrap();
    assert_eq!(anchors_of(&host).len(), 2);
}

// Checks `check_tile` on a seamless sprite: the blank slime repeats without
// seams, so no pair is listed. The call changes nothing.
#[test]
fn a_blank_sprite_is_seamless() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let result = call(&core, &session, "check_tile", json!({ "name": "slime" })).unwrap();
    assert_eq!(result["seamless"], true);
    assert_eq!(result["mismatch_count"], 0);
    assert_eq!(result["mismatches"], json!([]));
    assert_eq!(revision(&host), before);
}

// Checks the "Seam" scenario on the goblin (4x4): red at 0,0 and green at 3,3.
// Left column vs right column differ in rows 0 and 3; top row vs bottom row
// differ in columns 0 and 3. Each pair says the two places and keys.
#[test]
fn check_tile_lists_the_mismatching_pairs() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let result = call(&core, &session, "check_tile", json!({ "name": "goblin" })).unwrap();
    assert_eq!(result["seamless"], false);
    assert_eq!(result["mismatch_count"], 4);
    let list = result["mismatches"].as_array().unwrap();
    assert!(list.contains(&json!({ "edge": "left_right", "a": { "x": 0, "y": 0, "key": "R" }, "b": { "x": 3, "y": 0, "key": "." } })));
    assert!(list.contains(&json!({ "edge": "left_right", "a": { "x": 0, "y": 3, "key": "." }, "b": { "x": 3, "y": 3, "key": "G" } })));
    assert!(list.contains(&json!({ "edge": "top_bottom", "a": { "x": 0, "y": 0, "key": "R" }, "b": { "x": 0, "y": 3, "key": "." } })));
    assert!(list.contains(&json!({ "edge": "top_bottom", "a": { "x": 3, "y": 0, "key": "." }, "b": { "x": 3, "y": 3, "key": "G" } })));
}

// Checks that matching edges with colors are seamless: the same key on both
// sides is fine, and the frame argument picks another frame.
#[test]
fn equal_edges_are_seamless_and_the_frame_can_be_chosen() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    // Frame 2 of the goblin: red on the whole left and right columns.
    let pixels: Vec<_> = (0..4).flat_map(|y| [json!({ "x": 0, "y": y, "key": "R" }), json!({ "x": 3, "y": y, "key": "R" })]).collect();
    call(&core, &session, "set_pixels", json!({ "name": "goblin", "frame": 2, "pixels": pixels })).unwrap();
    // The top and bottom rows then match too: both have red at the corners.
    let result = call(&core, &session, "check_tile", json!({ "name": "goblin", "frame": 2 })).unwrap();
    assert_eq!(result["seamless"], true, "{result}");
    assert_eq!(result["frame"], 2);
    let error = call(&core, &session, "check_tile", json!({ "name": "goblin", "frame": 7 })).unwrap_err();
    assert_eq!(error["code"], "bad_frame");
}

// Checks that the list is capped at 50 pairs while the count is the real one.
#[test]
fn the_mismatch_list_is_capped() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "create_sprite", json!({ "name": "wall", "width": 60, "height": 60 })).unwrap();
    let pixels: Vec<_> = (0..60).map(|y| json!({ "x": 0, "y": y, "key": "R" })).collect();
    call(&core, &session, "set_pixels", json!({ "name": "wall", "pixels": pixels })).unwrap();
    let result = call(&core, &session, "check_tile", json!({ "name": "wall" })).unwrap();
    assert_eq!(result["mismatch_count"], 60);
    assert_eq!(result["mismatches"].as_array().unwrap().len(), 50);
    assert_eq!(result["truncated"], true);
}
