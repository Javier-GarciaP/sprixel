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
