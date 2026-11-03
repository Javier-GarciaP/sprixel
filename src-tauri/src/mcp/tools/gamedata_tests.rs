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
