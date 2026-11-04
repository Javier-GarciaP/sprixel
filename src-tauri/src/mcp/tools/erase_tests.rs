//! Spec `agent-tools-fixes`, task 4.1: a region grid keeps what is under a `.`,
//! and every write says how many opaque pixels it erased.

use crate::mcp::testing::{call, open_demo_project, rows, setup};
use serde_json::json;

/// `slime` with a 2x2 block of G at (3,3).
fn with_block(host: &crate::mcp::testing::TestHost, core: &crate::mcp::Core, session: &str) {
    open_demo_project(host);
    call(core, session, "apply_grid", json!({ "name": "slime", "frame": 1, "x": 3, "y": 3, "grid": "GG\nGG\n" })).unwrap();
}

// Checks the new default: a region grid whose `.` lies over the block leaves the
// block as it is, and `erased` is not in the answer (nothing was erased).
#[test]
fn a_region_grid_keeps_the_pixels_under_a_dot() {
    let (host, core, session) = setup();
    with_block(&host, &core, &session);

    let done = call(&core, &session, "apply_grid", json!({ "name": "slime", "frame": 1, "x": 2, "y": 2, "grid": ".R\nR.\n" })).unwrap();

    assert_eq!(rows(&host, &[], "slime", 1), "......../......../...R..../..RGG.../...GG.../......../......../........");
    assert!(done.get("erased").is_none() && done.get("warnings").is_none());
}

// Checks the explicit choice: `skip_transparent` false erases under the `.`,
// and the answer says how many pixels (1) and warns.
#[test]
fn an_explicit_false_erases_and_warns() {
    let (host, core, session) = setup();
    with_block(&host, &core, &session);

    let done = call(&core, &session, "apply_grid", json!({ "name": "slime", "frame": 1, "x": 3, "y": 3, "grid": ".G\nGG\n", "skip_transparent": false })).unwrap();

    assert_eq!(rows(&host, &[], "slime", 1), "......../......../......../....G.../...GG.../......../......../........");
    assert_eq!(done["erased"], 1);
    assert!(done["warnings"][0].as_str().unwrap().starts_with("1 opaque pixel(s) were erased"));
}

// Checks that a whole frame grid still erases (no warning, as it is expected)
// and reports `erased`: the 4 pixels of the block.
#[test]
fn a_whole_frame_grid_still_erases_and_counts() {
    let (host, core, session) = setup();
    with_block(&host, &core, &session);
    let blank = format!("frame 1\n{}\n", "........\n".repeat(8).trim_end());

    let done = call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": blank })).unwrap();

    assert_eq!(rows(&host, &[], "slime", 1), "......../......../......../......../......../......../......../........");
    assert_eq!(done["erased"], 4);
    assert!(done.get("warnings").is_none());
}

// Checks another tool: moving the block 2 pixels right clears its 4 old
// pixels, so `transform_region` reports `erased` 4. A dry run reports it too.
#[test]
fn other_writes_report_erased() {
    let (host, core, session) = setup();
    with_block(&host, &core, &session);

    let dry = call(&core, &session, "transform_region", json!({ "name": "slime", "operation": "move", "dx": 2, "dy": 0, "dry_run": true })).unwrap();
    assert_eq!(dry["erased"], 4);
    let done = call(&core, &session, "transform_region", json!({ "name": "slime", "operation": "move", "dx": 2, "dy": 0 })).unwrap();
    assert_eq!(done["erased"], 4);
    assert_eq!(rows(&host, &[], "slime", 1), "......../......../......../.....GG./.....GG./......../......../........");
    let draw = call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 5, "y": 3, "key": "." }] })).unwrap();
    assert_eq!(draw["erased"], 1);
}
