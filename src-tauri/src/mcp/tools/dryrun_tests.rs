//! Spec `agent-tools-fixes`, task 1.1: `dry_run` shows what a tool would do and
//! writes nothing.

use crate::ipc::project;
use crate::mcp::testing::{TestHost, call, connect_allowed, core_with, open_demo_project, revision, rows};
use crate::mcp::{Core, Ctx, Output, tools};
use serde_json::{Value, json};

const BLANK8: &str = "......../......../......../......../......../......../......../........";

fn setup() -> (std::sync::Arc<TestHost>, Core, String) {
    let (host, core) = core_with(tools::all());
    let session = connect_allowed(&core, "Claude Code");
    open_demo_project(&host);
    (host, core, session)
}

/// Draws a 2x2 block of G at (3,3) in `slime`, as the user (no AI history).
fn block(host: &TestHost, core: &Core, session: &str) {
    call(core, session, "set_pixels", json!({ "name": "slime", "pixels": [
        { "x": 3, "y": 3, "key": "G" }, { "x": 4, "y": 3, "key": "G" },
        { "x": 3, "y": 4, "key": "G" }, { "x": 4, "y": 4, "key": "G" },
    ] })).unwrap();
    project::edit(&host.project, |s| s.history.clear()).unwrap();
}

/// What a dry run must leave alone: the pixels, the revision, the history and
/// the UI events (the activity log entry is not one: the call itself is logged).
fn state(host: &TestHost) -> (String, u64, usize, usize) {
    let entries = project::read(&host.project, |s| s.history.entries().len()).unwrap();
    (rows(host, &[], "slime", 1), revision(host), entries, host.events.lock().unwrap().iter().filter(|(name, _)| name != crate::mcp::activity::ACTIVITY_EVENT).count())
}

/// Runs a tool and gives back its raw output (the image and the JSON).
fn run(core: &Core, tool: &str, arguments: Value) -> Result<Output, crate::mcp::ToolError> {
    let tool = core.tools.iter().find(|t| t.name == tool).unwrap();
    (tool.run)(&Ctx { core, client: "Claude Code", tool: tool.name }, &arguments)
}

// Checks "Outline without writing": the answer has the real count (12), says it
// was a dry run, carries an image where the outline is already drawn, and the
// pixels, revision, history and UI events are exactly the same as before.
#[test]
fn auto_outline_dry_run_shows_the_result_and_writes_nothing() {
    let (host, core, session) = setup();
    block(&host, &core, &session);
    let before = state(&host);

    let output = run(&core, "auto_outline", json!({ "name": "slime", "key": "R", "dry_run": true })).unwrap();

    let Output::Image { png, info } = output else { panic!("a dry run has an image") };
    assert_eq!(info["pixels_changed"], 12);
    assert_eq!((info["dry_run"].as_bool(), info["changed"].as_bool()), (Some(true), Some(false)));
    assert_eq!(info["would_change"], true);
    assert_eq!(info["revision"], before.1);
    let image = image::load_from_memory(&png).unwrap().to_rgba8();
    let scale = image.width() / 8;
    // The pixel (2, 3) is outline in the preview, and it is red (#FF0000).
    let p = image.get_pixel(2 * scale + scale / 2, 3 * scale + scale / 2);
    assert_eq!((p[0], p[1], p[2]), (255, 0, 0));
    assert_eq!(state(&host), before);
}

// Checks that the other four tools also write nothing and report their counts:
// shade, move (4 pixels cleared and 4 set), copy and a grid.
#[test]
fn every_dry_run_tool_leaves_the_project_as_it_was() {
    let (host, core, session) = setup();
    block(&host, &core, &session);
    let before = state(&host);
    let ask = |tool: &str, mut args: Value| {
        args["dry_run"] = json!(true);
        args["name"] = json!("slime");
        call(&core, &session, tool, args).unwrap()
    };

    let shade = ask("auto_shade", json!({ "light": "top-left", "base": "G", "keys": ["R", "G"] }));
    assert_eq!(shade["dry_run"], true);
    let flip = ask("transform_region", json!({ "operation": "move", "dx": 2, "dy": 0 }));
    assert_eq!((flip["pixels_changed"].as_u64(), flip["dry_run"].as_bool()), (Some(8), Some(true)));
    let copy = ask("copy_region", json!({ "region": { "x": 3, "y": 3, "width": 2, "height": 2 }, "to_x": 0, "to_y": 0 }));
    assert_eq!((copy["pixels_changed"].as_u64(), copy["dry_run"].as_bool()), (Some(4), Some(true)));
    let grid = ask("apply_grid", json!({ "grid": "frame 1\nRR\nRR\n", "x": 0, "y": 0 }));
    assert_eq!((grid["pixels_changed"].as_u64(), grid["dry_run"].as_bool()), (Some(4), Some(true)));

    assert_eq!(state(&host), before);
}

// Checks that a legend color of a grid is NOT added by a dry run, and that the
// same call without `dry_run` adds it (so the dry run did not spoil it).
#[test]
fn apply_grid_dry_run_does_not_add_colors() {
    let (host, core, session) = setup();
    let grid = "legend\n  Q teal #1FB5A8\nframe 1\nQ.......\n........\n........\n........\n........\n........\n........\n........\n";
    let colors = |host: &TestHost| project::read(&host.project, |s| s.project.palette("main").unwrap().colors.len()).unwrap();
    let before = (colors(&host), state(&host));

    let dry = call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": grid, "dry_run": true })).unwrap();
    assert_eq!((dry["colors_added"].as_u64(), dry["pixels_changed"].as_u64()), (Some(1), Some(1)));
    assert_eq!((colors(&host), state(&host)), before);
    assert_eq!(rows(&host, &[], "slime", 1), BLANK8);

    call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": grid })).unwrap();
    assert_eq!(colors(&host), before.0 + 1);
}

// Checks "Stale revision": a dry run with an old `base_revision` fails like a
// real call. Without `dry_run` (false) the call writes as before.
#[test]
fn dry_run_checks_the_revision_and_false_writes() {
    let (host, core, session) = setup();
    block(&host, &core, &session);
    let now = revision(&host);

    let stale = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R", "dry_run": true, "base_revision": now - 1 })).unwrap_err();
    assert_eq!(stale["code"], "stale_revision");
    let bad = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R", "dry_run": "yes" })).unwrap_err();
    assert_eq!(bad["code"], "bad_argument");

    let real = call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "R", "dry_run": false })).unwrap();
    assert_eq!((real["changed"].as_bool(), real["pixels_changed"].as_u64()), (Some(true), Some(12)));
    assert!(real.get("dry_run").is_none());
    assert_eq!(revision(&host), now + 1);
}
