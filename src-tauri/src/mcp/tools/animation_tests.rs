//! Spec `animation-organization`: the tools that make and manage animations.
//! The demo project has the sprite `goblin` (4x4, frames 1 and 2, the
//! animation `walk` over frames 1 to 2; frame 1 has two pixels).

use crate::ipc::{browser, project};
use crate::mcp::testing::{call, open_demo_project, revision, rows, setup};
use serde_json::{Value, json};

fn goblin(host: &crate::mcp::testing::TestHost) -> crate::document::Sprite {
    project::read(&host.project, |s| {
        s.project.find_sprite(&crate::document::SpriteRef::new(&["Characters"], "goblin")).unwrap().clone()
    })
    .unwrap()
}

fn entries(host: &crate::mcp::testing::TestHost) -> usize {
    project::read(&host.project, |s| s.history.entries().len()).unwrap()
}

fn animations(core: &crate::mcp::Core, session: &str) -> Vec<Value> {
    call(core, session, "get_sprite", json!({ "name": "goblin" })).unwrap()["animations"].as_array().unwrap().clone()
}

// Checks the "Four frames from the base" scenario: 3 copies of frame 1 are
// tagged `run` at 6 fps, the pixels of frame 1 are in all of them, the call is
// ONE history entry with an image, and one undo gives the sprite back.
#[test]
fn create_animation_copies_the_base_in_one_entry() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = entries(&host);
    let base_rows = rows(&host, &["Characters"], "goblin", 1);

    let done = call(&core, &session, "create_animation", json!({ "sprite": "goblin", "name": "run", "frames": 3, "fps": 6 })).unwrap();

    assert_eq!((done["from"].as_u64(), done["to"].as_u64(), done["frames"].as_u64()), (Some(3), Some(5), Some(5)));
    assert_eq!(done["revision"], revision(&host));
    for frame in 3..=5 {
        assert_eq!(rows(&host, &["Characters"], "goblin", frame), base_rows);
    }
    assert!(animations(&core, &session).contains(&json!({ "name": "run", "from": 3, "to": 5, "fps": 6, "mode": "loop" })));
    assert_eq!(entries(&host), before + 1);
    browser::undo(&host.project).unwrap();
    assert_eq!(goblin(&host).frame_count(), 2);
    assert_eq!(animations(&core, &session).len(), 1);
}

// Checks `base_frame`, `mode` and `at`: the copies come from frame 2 and go
// between the frames, and the animation after them moves with its frames.
#[test]
fn create_animation_can_choose_base_mode_and_place() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "set_pixels", json!({ "name": "goblin", "frame": 2, "pixels": [{ "x": 1, "y": 1, "key": "G" }] })).unwrap();
    call(&core, &session, "add_frame", json!({ "name": "goblin" })).unwrap();
    call(&core, &session, "add_frame", json!({ "name": "goblin" })).unwrap();
    call(&core, &session, "create_tag", json!({ "sprite": "goblin", "name": "idle", "from": 4, "to": 4, "fps": 4 })).unwrap();

    let done = call(&core, &session, "create_animation", json!({ "sprite": "goblin", "name": "hit", "frames": 2, "base_frame": 2, "at": 3, "mode": "once" })).unwrap();

    assert_eq!(done["mode"], "once");
    assert_eq!(done["fps"], 8, "the default speed");
    let frame2 = rows(&host, &["Characters"], "goblin", 2);
    assert!(frame2.contains('G'));
    assert_eq!(rows(&host, &["Characters"], "goblin", 3), frame2);
    assert_eq!(rows(&host, &["Characters"], "goblin", 4), frame2);
    assert!(!rows(&host, &["Characters"], "goblin", 5).contains('G'), "the old frame 3 is blank");
    let all = animations(&core, &session);
    assert!(all.contains(&json!({ "name": "hit", "from": 3, "to": 4, "fps": 8, "mode": "once" })), "{all:?}");
    assert!(all.contains(&json!({ "name": "idle", "from": 6, "to": 6, "fps": 4, "mode": "loop" })), "{all:?}");
}

// Checks the errors: a name in use (ignoring case), a place inside another
// animation, a base frame that does not exist, a bad mode and a stale
// revision. Nothing changes after any of them.
#[test]
fn create_animation_errors_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let code = |arguments: Value| call(&core, &session, "create_animation", arguments).unwrap_err()["code"].as_str().unwrap().to_string();

    assert_eq!(code(json!({ "sprite": "goblin", "name": "WALK", "frames": 2 })), "name_exists");
    assert_eq!(code(json!({ "sprite": "goblin", "name": "x", "frames": 2, "at": 2 })), "animation_overlap");
    assert_eq!(code(json!({ "sprite": "goblin", "name": "x", "frames": 2, "base_frame": 9 })), "bad_frame");
    assert_eq!(code(json!({ "sprite": "goblin", "name": "x", "frames": 2, "mode": "spin" })), "bad_mode");
    assert_eq!(code(json!({ "sprite": "goblin", "name": "x", "frames": 0 })), "bad_argument");
    assert_eq!(code(json!({ "sprite": "goblin", "name": "x", "frames": 2, "fps": 99 })), "bad_fps");
    assert_eq!(code(json!({ "sprite": "ghost", "name": "x", "frames": 2 })), "not_found");
    assert_eq!(code(json!({ "sprite": "goblin", "name": "x", "frames": 2, "base_revision": before + 5 })), "stale_revision");
    assert_eq!(revision(&host), before);
    assert_eq!(goblin(&host).frame_count(), 2);
}

// Checks `duplicate_animation`: the copy is at the end with the same pixels,
// speed and mode, under the new name; one history entry; undo removes it. A
// name in use and an animation that does not exist are refused.
#[test]
fn duplicate_animation_copies_frames_and_settings() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = entries(&host);

    let done = call(&core, &session, "duplicate_animation", json!({ "sprite": "goblin", "name": "WALK", "new_name": "walk2" })).unwrap();

    assert_eq!((done["from"].as_u64(), done["to"].as_u64()), (Some(3), Some(4)));
    assert_eq!(rows(&host, &["Characters"], "goblin", 3), rows(&host, &["Characters"], "goblin", 1));
    assert!(animations(&core, &session).contains(&json!({ "name": "walk2", "from": 3, "to": 4, "fps": 12, "mode": "loop" })));
    assert_eq!(entries(&host), before + 1);
    let code = |name: &str, new_name: &str| {
        call(&core, &session, "duplicate_animation", json!({ "sprite": "goblin", "name": name, "new_name": new_name })).unwrap_err()["code"].as_str().unwrap().to_string()
    };
    assert_eq!(code("walk", "Walk2"), "name_exists");
    assert_eq!(code("jump", "x"), "not_found");
    browser::undo(&host.project).unwrap();
    assert_eq!(goblin(&host).frame_count(), 2);
}

// Checks `rename_animation`: only the name changes, frames and speed stay.
#[test]
fn rename_animation_changes_only_the_name() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "create_animation", json!({ "sprite": "goblin", "name": "run", "frames": 2 })).unwrap();

    call(&core, &session, "rename_animation", json!({ "sprite": "goblin", "name": "walk", "new_name": "stroll" })).unwrap();

    let all = animations(&core, &session);
    assert!(all.contains(&json!({ "name": "stroll", "from": 1, "to": 2, "fps": 12, "mode": "loop" })), "{all:?}");
    let error = call(&core, &session, "rename_animation", json!({ "sprite": "goblin", "name": "stroll", "new_name": "RUN" })).unwrap_err();
    assert_eq!(error["code"], "name_exists");
    browser::undo(&host.project).unwrap();
    assert!(animations(&core, &session).iter().any(|a| a["name"] == "walk"));
}

// Checks the "Delete with frames" scenario and the default: without
// `with_frames` the frames stay; with it the frames go, the animations after
// them follow their frames, and one undo gives everything back.
#[test]
fn delete_animation_keeps_frames_unless_asked() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "create_animation", json!({ "sprite": "goblin", "name": "run", "frames": 2 })).unwrap();

    call(&core, &session, "delete_animation", json!({ "sprite": "goblin", "name": "run" })).unwrap();
    assert_eq!((goblin(&host).frame_count(), animations(&core, &session).len()), (4, 1));
    browser::undo(&host.project).unwrap();

    let before = entries(&host);
    let done = call(&core, &session, "delete_animation", json!({ "sprite": "goblin", "name": "walk", "with_frames": true })).unwrap();

    assert_eq!(done["deleted_frames"], 2);
    assert_eq!(goblin(&host).frame_count(), 2);
    assert!(animations(&core, &session).contains(&json!({ "name": "run", "from": 1, "to": 2, "fps": 8, "mode": "loop" })));
    assert_eq!(entries(&host), before + 1);
    browser::undo(&host.project).unwrap();
    assert_eq!(goblin(&host).frame_count(), 4);
    assert_eq!(animations(&core, &session).len(), 2);
    assert_eq!(call(&core, &session, "delete_animation", json!({ "sprite": "goblin", "name": "jump" })).unwrap_err()["code"], "not_found");
}

// Checks that the tags follow their frames when a frame moves, and that the
// call is one history entry. Frame 1 goes to the end: the old frame 1 pixels
// are now the last frame, and `walk` (frames 1 to 2 before) still holds them.
#[test]
fn move_frame_moves_a_frame_and_the_tags_follow() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "add_frame", json!({ "name": "goblin" })).unwrap();
    let first = rows(&host, &["Characters"], "goblin", 1);
    let before = entries(&host);

    let done = call(&core, &session, "move_frame", json!({ "name": "goblin", "from": 1, "to": 3 })).unwrap();

    assert_eq!(done["revision"], revision(&host));
    assert_eq!(rows(&host, &["Characters"], "goblin", 3), first);
    assert!(animations(&core, &session).contains(&json!({ "name": "walk", "from": 1, "to": 3, "fps": 12, "mode": "loop" })));
    assert_eq!(entries(&host), before + 1);
    assert_eq!(call(&core, &session, "move_frame", json!({ "name": "goblin", "from": 1, "to": 9 })).unwrap_err()["code"], "bad_frame");
    browser::undo(&host.project).unwrap();
    assert_eq!(rows(&host, &["Characters"], "goblin", 1), first);
    assert!(animations(&core, &session).contains(&json!({ "name": "walk", "from": 1, "to": 2, "fps": 12, "mode": "loop" })));
}

// ---- `frames` in the region tools ----

/// `slime` (8x8) with `count` frames; every frame has a red pixel at (0, 0).
fn slime_with_frames(core: &crate::mcp::Core, session: &str, count: usize) {
    for _ in 1..count {
        call(core, session, "add_frame", json!({ "name": "slime" })).unwrap();
    }
    for frame in 1..=count {
        call(core, session, "set_pixels", json!({ "name": "slime", "frame": frame, "pixels": [{ "x": 0, "y": 0, "key": "R" }] })).unwrap();
    }
}

fn first_cell(host: &crate::mcp::testing::TestHost, frame: usize) -> String {
    rows(host, &[], "slime", frame).split('/').next().unwrap().to_string()
}

// Checks the "One call, six frames" scenario: `move` on frames "2-7" moves the
// pixel on all six frames, frames 1 and 8 stay, it is ONE history entry and
// one undo restores all six.
#[test]
fn transform_region_changes_a_range_of_frames_atomically() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    slime_with_frames(&core, &session, 8);
    let before = entries(&host);

    let done = call(&core, &session, "transform_region", json!({ "name": "slime", "frames": "2-7", "operation": "move", "dx": 2, "dy": 0 })).unwrap();

    assert_eq!(done["pixels_changed"], 12, "two pixels per frame");
    assert_eq!(done["frames_changed"], json!([2, 3, 4, 5, 6, 7]));
    for frame in [1, 8] {
        assert_eq!(first_cell(&host, frame), "R.......");
    }
    for frame in 2..=7 {
        assert_eq!(first_cell(&host, frame), "..R.....", "frame {frame}");
    }
    assert_eq!(entries(&host), before + 1);
    crate::ipc::browser::undo(&host.project).unwrap();
    assert_eq!(first_cell(&host, 4), "R.......");
}

// Checks that `frames` can be a list (any order, repeated numbers count once),
// that a single number as text works, and that it cannot be mixed with `frame`.
#[test]
fn frames_can_be_a_list_or_a_text() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    slime_with_frames(&core, &session, 4);

    call(&core, &session, "transform_region", json!({ "name": "slime", "frames": [4, 2, 4], "operation": "flip_horizontal" })).unwrap();
    assert_eq!((first_cell(&host, 1), first_cell(&host, 2), first_cell(&host, 3), first_cell(&host, 4)), ("R.......".into(), ".......R".into(), "R.......".into(), ".......R".into()));
    call(&core, &session, "transform_region", json!({ "name": "slime", "frames": "3", "operation": "flip_horizontal" })).unwrap();
    assert_eq!(first_cell(&host, 3), ".......R");
    let mixed = call(&core, &session, "transform_region", json!({ "name": "slime", "frame": 1, "frames": [2], "operation": "flip_horizontal" })).unwrap_err();
    assert_eq!(mixed["code"], "bad_argument");
}

// Checks the errors of `frames`: a frame that does not exist, a bad text, an
// empty list and a backwards range. Nothing changes, not even the frames
// before the bad one.
#[test]
fn bad_frames_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    slime_with_frames(&core, &session, 3);
    let before = revision(&host);
    let code = |frames: serde_json::Value| {
        call(&core, &session, "transform_region", json!({ "name": "slime", "frames": frames, "operation": "flip_horizontal" })).unwrap_err()["code"].as_str().unwrap().to_string()
    };

    assert_eq!(code(json!([1, 2, 9])), "bad_frame");
    assert_eq!(code(json!("2-9")), "bad_frame");
    assert_eq!(code(json!("0-2")), "bad_frame");
    assert_eq!(code(json!("3-1")), "bad_argument");
    assert_eq!(code(json!("a-b")), "bad_argument");
    assert_eq!(code(json!([])), "bad_argument");
    assert_eq!(code(json!(true)), "bad_argument");
    assert_eq!(revision(&host), before);
    assert_eq!(first_cell(&host, 1), "R.......");
}

// Checks `copy_region` with `frames`: the region of the source frame is
// pasted on every destination frame, as one history entry; `to_frame` cannot
// be used together with it.
#[test]
fn copy_region_pastes_on_a_range_of_frames() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    slime_with_frames(&core, &session, 5);
    call(&core, &session, "set_pixels", json!({ "name": "slime", "frame": 1, "pixels": [{ "x": 5, "y": 5, "key": "G" }] })).unwrap();
    let before = entries(&host);

    let done = call(&core, &session, "copy_region", json!({ "name": "slime", "frame": 1, "region": { "x": 5, "y": 5, "width": 1, "height": 1 }, "frames": "3-5" })).unwrap();

    assert_eq!(done["frames_changed"], json!([3, 4, 5]));
    for frame in 3..=5 {
        assert_eq!(rows(&host, &[], "slime", frame).split('/').nth(5).unwrap(), ".....G..", "frame {frame}");
    }
    assert!(!rows(&host, &[], "slime", 2).contains('G'));
    assert_eq!(entries(&host), before + 1);
    let both = call(&core, &session, "copy_region", json!({ "name": "slime", "frames": [2], "to_frame": 3 })).unwrap_err();
    assert_eq!(both["code"], "bad_argument");
}
