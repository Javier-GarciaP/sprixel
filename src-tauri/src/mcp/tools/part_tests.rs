//! Spec `animation-organization`, "Named parts": `set_part`, `remove_part` and
//! `part` in `transform_region` and `copy_region`.

use crate::ipc::{browser, project};
use crate::mcp::testing::{TestHost, call, open_demo_project, revision, rows, setup};
use serde_json::{Value, json};

fn entries(host: &TestHost) -> usize {
    project::read(&host.project, |s| s.history.entries().len()).unwrap()
}

fn top_row(host: &TestHost, frame: usize) -> String {
    rows(host, &[], "slime", frame).split('/').next().unwrap().to_string()
}

/// `slime` (8x8) with 4 frames, each with a red pixel at (1, 0) and a green
/// one at (6, 0), so a part on the left half moves only the red one.
fn slime_with_frames(core: &crate::mcp::Core, session: &str) {
    for _ in 1..4 {
        call(core, session, "add_frame", json!({ "name": "slime" })).unwrap();
    }
    for frame in 1..=4 {
        let pixels = json!([{ "x": 1, "y": 0, "key": "R" }, { "x": 6, "y": 0, "key": "G" }]);
        call(core, session, "set_pixels", json!({ "name": "slime", "frame": frame, "pixels": pixels })).unwrap();
    }
}

// Checks `set_part`: it adds a part, a second call with the same name (any
// case) replaces it, `get_sprite` lists it, and each call is one history entry
// that one undo takes back.
#[test]
fn set_part_adds_and_replaces_a_part() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = entries(&host);

    let done = call(&core, &session, "set_part", json!({ "sprite": "slime", "name": "head", "x": 0, "y": 0, "width": 4, "height": 4 })).unwrap();
    assert_eq!((done["parts"].as_u64(), done["revision"].clone()), (Some(1), json!(revision(&host))));
    let again = call(&core, &session, "set_part", json!({ "sprite": "slime", "name": "HEAD", "x": 1, "y": 1, "width": 3, "height": 3 })).unwrap();
    assert_eq!(again["parts"], 1, "a replace does not add");

    let sprite = call(&core, &session, "get_sprite", json!({ "name": "slime" })).unwrap();
    assert_eq!(sprite["parts"], json!([{ "name": "HEAD", "x": 1, "y": 1, "width": 3, "height": 3 }]));
    assert_eq!(entries(&host), before + 2);
    browser::undo(&host.project).unwrap();
    let sprite = call(&core, &session, "get_sprite", json!({ "name": "slime" })).unwrap();
    assert_eq!(sprite["parts"][0]["name"], "head");
}

// Checks the errors of `set_part` and `remove_part`: outside the sprite, no
// size, a bad name, a sprite or part that does not exist. Nothing changes.
#[test]
fn part_errors_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let code = |tool: &str, arguments: Value| call(&core, &session, tool, arguments).unwrap_err()["code"].as_str().unwrap().to_string();

    assert_eq!(code("set_part", json!({ "sprite": "slime", "name": "a", "x": 6, "y": 0, "width": 4, "height": 2 })), "region_out");
    assert_eq!(code("set_part", json!({ "sprite": "slime", "name": "a", "x": 0, "y": 0, "width": 0, "height": 2 })), "bad_part");
    assert_eq!(code("set_part", json!({ "sprite": "slime", "name": " ", "x": 0, "y": 0, "width": 1, "height": 1 })), "bad_part");
    assert_eq!(code("set_part", json!({ "sprite": "ghost", "name": "a", "x": 0, "y": 0, "width": 1, "height": 1 })), "not_found");
    assert_eq!(code("set_part", json!({ "sprite": "slime", "name": "a", "x": 0, "y": 0, "width": 1 })), "bad_argument");
    assert_eq!(code("remove_part", json!({ "sprite": "slime", "name": "nope" })), "not_found");
    assert_eq!(revision(&host), before);
}

// Checks the "Move the head on many frames" scenario: `transform_region` with
// `part` "head", `dy` 3 and `frames` "2-4" moves the pixels inside the part on
// those frames only, as one history entry; the pixels outside the part stay.
#[test]
fn transform_region_uses_a_part_on_many_frames() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    slime_with_frames(&core, &session);
    call(&core, &session, "set_part", json!({ "sprite": "slime", "name": "head", "x": 0, "y": 0, "width": 4, "height": 4 })).unwrap();
    let before = entries(&host);

    let done = call(&core, &session, "transform_region", json!({ "name": "slime", "part": "Head", "frames": "2-4", "operation": "move", "dx": 0, "dy": 3 })).unwrap();

    assert_eq!(done["frames_changed"], json!([2, 3, 4]));
    assert_eq!(top_row(&host, 1), ".R....G.", "frame 1 is not in the range");
    for frame in 2..=4 {
        assert_eq!(top_row(&host, frame), "......G.", "frame {frame}: the green pixel is outside the part");
        assert_eq!(rows(&host, &[], "slime", frame).split('/').nth(3).unwrap(), ".R......");
    }
    assert_eq!(entries(&host), before + 1);
    browser::undo(&host.project).unwrap();
    assert_eq!(top_row(&host, 3), ".R....G.");
}

// Checks `copy_region` with `part`: the part of frame 1 is copied to frame 2
// at the same place.
#[test]
fn copy_region_uses_a_part_as_the_source() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "add_frame", json!({ "name": "slime" })).unwrap();
    call(&core, &session, "set_pixels", json!({ "name": "slime", "frame": 1, "pixels": [{ "x": 5, "y": 5, "key": "G" }] })).unwrap();
    call(&core, &session, "set_part", json!({ "sprite": "slime", "name": "tail", "x": 4, "y": 4, "width": 3, "height": 3 })).unwrap();

    call(&core, &session, "copy_region", json!({ "name": "slime", "part": "tail", "to_frame": 2 })).unwrap();

    assert_eq!(rows(&host, &[], "slime", 2).split('/').nth(5).unwrap(), ".....G..");
}

// Checks the errors of `part` in the region tools: an unknown name lists the
// parts, `part` with `region` is refused, and a part that no longer fits after
// the canvas was made smaller gives `region_out`. Removing a part works and the
// pixels stay.
#[test]
fn bad_parts_in_the_region_tools_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "set_part", json!({ "sprite": "slime", "name": "head", "x": 4, "y": 4, "width": 4, "height": 4 })).unwrap();
    let before = revision(&host);
    let flip = |extra: Value| {
        let mut arguments = json!({ "name": "slime", "operation": "flip_horizontal" });
        arguments.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        call(&core, &session, "transform_region", arguments).unwrap_err()
    };

    let unknown = flip(json!({ "part": "tail" }));
    assert_eq!(unknown["code"], "not_found");
    assert!(unknown["message"].as_str().unwrap().contains("head"), "{unknown}");
    assert_eq!(flip(json!({ "part": "head", "region": { "x": 0, "y": 0, "width": 2, "height": 2 } }))["code"], "bad_argument");
    assert_eq!(revision(&host), before);

    call(&core, &session, "resize_sprite", json!({ "name": "slime", "width": 5, "height": 5 })).unwrap();
    assert_eq!(flip(json!({ "part": "head" }))["code"], "region_out");

    call(&core, &session, "remove_part", json!({ "sprite": "slime", "name": "HEAD" })).unwrap();
    assert_eq!(call(&core, &session, "get_sprite", json!({ "name": "slime" })).unwrap()["parts"], json!([]));
    assert_eq!(flip(json!({ "part": "head" }))["code"], "not_found");
}
