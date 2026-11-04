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
