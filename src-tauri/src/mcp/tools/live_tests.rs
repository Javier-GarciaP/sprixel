//! Spec `agent-live-design`, tasks 1.1 and 1.5: the event `agent-ops` with the
//! pixels of a write in drawing order, and a frame that stays busy until the
//! editor says the replay is done (or a timer ends it).

use crate::ipc::editor::AGENT_EVENT;
use crate::mcp::playback_done;
use crate::mcp::testing::{TestHost, call, open_demo_project, setup};
use crate::mcp::tools::write::OPS_EVENT;
use serde_json::{Value, json};

fn events(host: &TestHost, name: &str) -> Vec<Value> {
    host.events.lock().unwrap().iter().filter(|(e, _)| e == name).map(|(_, v)| v.clone()).collect()
}

/// The pixels of the one `agent-ops` event, as text like `0,0,R`.
fn replay(host: &TestHost) -> Vec<String> {
    let ops = events(host, OPS_EVENT);
    assert_eq!(ops.len(), 1, "one event per write");
    ops[0]["pixels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| format!("{},{},{}", p[0], p[1], p[2].as_str().unwrap()))
        .collect()
}

// Checks the "Line" scenario: a line is listed from its start to its end, and
// a line drawn backwards starts at the first point it was given. The event also
// says who, which sprite, which frame and which tool.
#[test]
fn a_line_is_listed_from_start_to_end() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "draw_shape", json!({ "name": "slime", "shape": "line", "key": "R", "x0": 0, "y0": 0, "x1": 3, "y1": 0 })).unwrap();
    assert_eq!(replay(&host), ["0,0,R", "1,0,R", "2,0,R", "3,0,R"]);
    let event = &events(&host, OPS_EVENT)[0];
    assert_eq!(event["agent"], "Claude Code");
    assert_eq!(event["sprite"], json!({ "folder": [], "name": "slime" }));
    assert_eq!(event["frame"], 1);
    assert_eq!(event["tool"], "draw_shape");

    host.events.lock().unwrap().clear();
    call(&core, &session, "draw_shape", json!({ "name": "slime", "shape": "line", "key": "G", "x0": 3, "y0": 2, "x1": 0, "y1": 2 })).unwrap();
    assert_eq!(replay(&host), ["3,2,G", "2,2,G", "1,2,G", "0,2,G"]);
}

// Checks that a filled rectangle goes row by row, and only the pixels that
// really change are listed.
#[test]
fn a_shape_goes_row_by_row_with_only_the_changed_pixels() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "draw_shape", json!({ "name": "slime", "shape": "rectangle", "key": "R", "x": 1, "y": 1, "width": 2, "height": 2, "filled": true })).unwrap();

    assert_eq!(replay(&host), ["1,1,R", "2,1,R", "1,2,R", "2,2,R"]);
}

// Checks the "Grid" scenario: a grid is listed row by row (only the pixels that
// change), and the tool name is `apply_grid`.
#[test]
fn a_grid_is_listed_row_by_row() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": "frame 1\nRR......\n.G......\n........\n........\n........\n........\n........\n........\n" })).unwrap();

    assert_eq!(replay(&host), ["0,0,R", "1,0,R", "1,1,G"]);
    assert_eq!(events(&host, OPS_EVENT)[0]["tool"], "apply_grid");
}

// Checks that `set_pixels` keeps the order the agent gave, even if it is not
// top to bottom.
#[test]
fn set_pixels_keeps_the_given_order() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 5, "y": 5, "key": "G" }, { "x": 0, "y": 0, "key": "R" }, { "x": 2, "y": 1, "key": "R" }] })).unwrap();

    assert_eq!(replay(&host), ["5,5,G", "0,0,R", "2,1,R"]);
}

// Checks that a call that changes no pixel (add_frame) sends no event and frees
// the frame when the call ends, like before.
#[test]
fn a_call_without_pixels_has_no_replay_and_frees_the_frame() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "add_frame", json!({ "name": "goblin" })).unwrap();

    assert!(events(&host, OPS_EVENT).is_empty());
    assert_eq!(host.agents.writing(), None);
    assert!(events(&host, AGENT_EVENT).last().unwrap().is_null());
}

// Checks that after a write with pixels the frame stays busy (the person's
// strokes wait) until the editor says the replay is done. The event carries
// the number the editor must send back.
#[test]
fn the_frame_stays_busy_until_the_replay_is_done() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "R" }] })).unwrap();

    assert!(host.agents.writing().is_some(), "busy after the call");
    let id = events(&host, OPS_EVENT)[0]["id"].as_u64().unwrap();
    assert!(playback_done(&*host, id));
    assert_eq!(host.agents.writing(), None);
    assert!(events(&host, AGENT_EVENT).last().unwrap().is_null(), "the editor is told to reload");
    assert!(!playback_done(&*host, id), "a second done does nothing");
}

// Checks that a "done" for an old write does not free a newer one: two calls in
// a row, and the editor finishes the first replay while the second is waiting.
#[test]
fn an_old_done_does_not_free_a_newer_write() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let pixel = json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "R" }] });
    call(&core, &session, "set_pixels", pixel.clone()).unwrap();
    call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 1, "y": 0, "key": "R" }] })).unwrap();
    let ids: Vec<u64> = events(&host, OPS_EVENT).iter().map(|e| e["id"].as_u64().unwrap()).collect();

    assert!(!playback_done(&*host, ids[0]));
    assert!(host.agents.writing().is_some());
    assert!(playback_done(&*host, ids[1]));
}

// Checks the timeout: if the editor never answers (the window is closed), the
// core frees the frame by itself and tells the UI.
#[test]
fn the_core_frees_the_frame_after_the_timeout() {
    let (host, mut core, session) = setup();
    open_demo_project(&host);
    core.playback_timeout = std::time::Duration::from_millis(20);

    call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "R" }] })).unwrap();

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while host.agents.writing().is_some() {
        assert!(std::time::Instant::now() < deadline, "the timer never freed the frame");
        std::thread::yield_now();
    }
    assert!(events(&host, AGENT_EVENT).last().unwrap().is_null());
}
