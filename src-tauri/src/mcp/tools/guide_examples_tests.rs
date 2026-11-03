//! Spec `agent-guides-v2`: the worked example of each guide is run through the
//! real tools, so a guide cannot teach calls that do not work.

use super::docs::{GEM_GRID, LARGE_SPRITES_GUIDE, MUSHROOM_CAP_GRID, MUSHROOM_STEM_GRID, TOOLS_GUIDE};
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

// Checks the example of the `large-sprites` guide: two halves written with a
// horizontal symmetry make a 16x16 mushroom that is the same on both sides,
// `auto_outline` and `auto_shade` finish it, and `analyze_sprite` finds
// nothing. The last call reads one part with `region`.
#[test]
fn the_large_sprites_example_draws_a_mushroom_part_by_part() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    assert!(LARGE_SPRITES_GUIDE.contains(MUSHROOM_CAP_GRID) && LARGE_SPRITES_GUIDE.contains(MUSHROOM_STEM_GRID));

    call(&core, &session, "create_sprite", json!({ "name": "mushroom", "width": 16, "height": 16 })).unwrap();
    let cap = call(&core, &session, "apply_grid", json!({ "name": "mushroom", "x": 0, "y": 2, "frame": 1, "symmetry": "horizontal", "grid": MUSHROOM_CAP_GRID })).unwrap();
    assert_eq!(cap["colors_added"], 6);
    call(&core, &session, "apply_grid", json!({ "name": "mushroom", "x": 0, "y": 8, "frame": 1, "symmetry": "horizontal", "grid": MUSHROOM_STEM_GRID })).unwrap();
    let lines = rows(&host, &[], "mushroom", 1);
    let lines: Vec<&str> = lines.split('/').collect();
    assert_eq!(lines[2], "....CCCCCCCC....");
    assert_eq!(lines[4], ".CCWWCCCCCCWWCC.");
    assert_eq!(lines[9], ".....SSSSSS.....");
    assert!(lines.iter().all(|line| line.chars().eq(line.chars().rev())), "both sides are equal: {lines:?}");

    call(&core, &session, "auto_outline", json!({ "name": "mushroom", "key": "K" })).unwrap();
    let shade = call(&core, &session, "auto_shade", json!({ "name": "mushroom", "light": "top-left", "base": "C", "keys": ["D", "C", "L"] })).unwrap();
    assert!(shade["lit"].as_u64().unwrap() > 0 && shade["shadow"].as_u64().unwrap() > 0, "{shade}");

    let report = call(&core, &session, "analyze_sprite", json!({ "name": "mushroom", "outline_key": "K" })).unwrap();
    assert_eq!(report["frame_reports"][0]["loose_pixels"]["count"], 0, "{report}");
    assert_eq!(report["frame_reports"][0]["outline_gaps"]["count"], 0, "{report}");

    let part = call(&core, &session, "get_sprite", json!({ "name": "mushroom", "frame": 1, "region": { "x": 0, "y": 8, "width": 16, "height": 6 }, "include_palette": false })).unwrap();
    assert_eq!(part["grid"].as_str().unwrap().lines().filter(|l| l.len() == 16).count(), 6, "{part}");
}
