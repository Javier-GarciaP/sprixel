//! Spec `agent-guides-v2`: the worked example of each guide is run through the
//! real tools, so a guide cannot teach calls that do not work.

use super::docs::{GEM_GRID, TOOLS_GUIDE};
use crate::mcp::testing::{call, open_demo_project, rows, setup};
use serde_json::json;

// Checks the example of the `tools` guide: the gem has a gap in its outline,
// `analyze_sprite` finds it at x 6, y 3, `set_pixels` closes it and the second
// `analyze_sprite` finds nothing. It also checks that the guide prints the grid
// that the test uses.
#[test]
fn the_tools_example_finds_and_fixes_a_gap() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    assert!(TOOLS_GUIDE.contains(GEM_GRID));

    call(&core, &session, "create_sprite", json!({ "name": "gem", "width": 8, "height": 8 })).unwrap();
    call(&core, &session, "apply_grid", json!({ "name": "gem", "grid": GEM_GRID })).unwrap();

    let before = call(&core, &session, "analyze_sprite", json!({ "name": "gem", "outline_key": "K" })).unwrap();
    assert_eq!(before["frame_reports"][0]["outline_gaps"], json!({ "count": 1, "at": [[6, 3]] }));
    assert_eq!(before["frame_reports"][0]["loose_pixels"]["count"], 0);

    call(&core, &session, "set_pixels", json!({ "name": "gem", "pixels": [{ "x": 7, "y": 3, "key": "K" }] })).unwrap();

    let after = call(&core, &session, "analyze_sprite", json!({ "name": "gem", "outline_key": "K" })).unwrap();
    assert_eq!(after["frame_reports"][0]["outline_gaps"]["count"], 0);
    assert_eq!(after["frame_reports"][0]["loose_pixels"]["count"], 0);
    assert_eq!(rows(&host, &[], "gem", 1).split('/').nth(3).unwrap(), "KJJJJJJK");
}
