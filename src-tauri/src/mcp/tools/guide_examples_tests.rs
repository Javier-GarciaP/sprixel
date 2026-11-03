//! Spec `agent-guides-v2`: the worked example of each guide is run through the
//! real tools, so a guide cannot teach calls that do not work.

use super::docs::{
    CAT_GRID, FACES_GUIDE, GEM_GRID, LARGE_SPRITES_GUIDE, MUSHROOM_CAP_GRID, MUSHROOM_STEM_GRID, TOOLS_GUIDE, WALK_GUIDE,
    WALKER_GRID,
};
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

// Checks the example of the `faces-and-animals` guide: the half head, the eyes
// and the nose are written with a horizontal symmetry, so the face is equal on
// both sides, the eyes have their light pixel at the top left, and after the
// outline `analyze_sprite` finds no loose pixel and no gap.
#[test]
fn the_faces_example_draws_a_cat_head() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    assert!(FACES_GUIDE.contains(CAT_GRID));

    call(&core, &session, "create_sprite", json!({ "name": "cat", "width": 16, "height": 16 })).unwrap();
    call(&core, &session, "apply_grid", json!({ "name": "cat", "x": 0, "y": 2, "frame": 1, "symmetry": "horizontal", "grid": CAT_GRID })).unwrap();
    let eyes = [(5, 6, "E"), (4, 7, "E"), (5, 7, "E"), (4, 6, "W")].map(|(x, y, key)| json!({ "x": x, "y": y, "key": key }));
    call(&core, &session, "set_pixels", json!({ "name": "cat", "symmetry": "horizontal", "pixels": eyes })).unwrap();
    call(&core, &session, "set_pixels", json!({ "name": "cat", "symmetry": "horizontal", "pixels": [{ "x": 7, "y": 9, "key": "N" }] })).unwrap();
    call(&core, &session, "auto_outline", json!({ "name": "cat", "key": "K" })).unwrap();

    let all = rows(&host, &[], "cat", 1);
    let lines: Vec<&str> = all.split('/').collect();
    assert_eq!(lines[6], "KOOOWEOOOOEWOOOK", "{all}");
    assert_eq!(lines[7], "KOOOEEOOOOEEOOOK", "{all}");
    assert_eq!(lines[9], "KOOOOOONNOOOOOOK", "{all}");
    assert!(lines.iter().all(|line| line.chars().eq(line.chars().rev())), "both sides are equal: {all}");

    let report = call(&core, &session, "analyze_sprite", json!({ "name": "cat", "outline_key": "K" })).unwrap();
    assert_eq!(report["frame_reports"][0]["loose_pixels"]["count"], 0, "{report}");
    assert_eq!(report["frame_reports"][0]["outline_gaps"]["count"], 0, "{report}");
}

/// Makes the 4 frame walker of the `walk-and-attack-cycles` guide (steps 1 to 3).
fn make_walker(core: &crate::mcp::Core, session: &str) {
    call(core, session, "create_sprite", json!({ "name": "walker", "width": 12, "height": 10 })).unwrap();
    for _ in 0..3 {
        call(core, session, "add_frame", json!({ "name": "walker" })).unwrap();
    }
    call(core, session, "apply_grid", json!({ "name": "walker", "grid": WALKER_GRID })).unwrap();
}

// Checks the walk of the example: 4 different frames, the body 1 pixel higher
// in the passing frames, the lowest row of the feet is row 9 in every frame,
// the durations and the tag are saved, and `analyze_sprite` finds nothing
// wrong. `ground_kept` is false between frames because a foot moves.
#[test]
fn the_walk_example_makes_a_four_frame_walk() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    assert!(WALK_GUIDE.contains(WALKER_GRID));
    make_walker(&core, &session);
    call(&core, &session, "render_frames", json!({ "name": "walker", "labels": true })).unwrap();

    let frame = |n: usize| rows(&host, &[], "walker", n);
    for n in 1..=4 {
        assert_eq!(frame(n).split('/').nth(9).unwrap().contains('K'), true, "frame {n} has its feet on row 9");
    }
    assert_eq!(frame(1).split('/').next().unwrap(), "............");
    assert_eq!(frame(2).split('/').next().unwrap(), "...KKKKKK...");
    assert_ne!(frame(1), frame(3), "the legs change color places");

    for (frame, milliseconds) in [(1, 120), (2, 80), (3, 120), (4, 80)] {
        call(&core, &session, "set_frame_duration", json!({ "name": "walker", "frame": frame, "milliseconds": milliseconds })).unwrap();
    }
    call(&core, &session, "create_tag", json!({ "sprite": "walker", "name": "walk", "from": 1, "to": 4, "fps": 10, "mode": "loop" })).unwrap();

    let sprite = call(&core, &session, "get_sprite", json!({ "name": "walker", "frame": 1 })).unwrap();
    assert_eq!(sprite["frame_durations"], json!([120, 80, 120, 80]));
    assert_eq!(sprite["animations"][0]["name"], "walk");

    let report = call(&core, &session, "analyze_sprite", json!({ "name": "walker", "outline_key": "K" })).unwrap();
    for n in 0..4 {
        assert_eq!(report["frame_reports"][n]["loose_pixels"]["count"], 0, "{report}");
        assert_eq!(report["frame_reports"][n]["outline_gaps"]["count"], 0, "{report}");
    }
    let changes = report["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 3, "{report}");
    assert!(changes.iter().all(|change| change["ground_kept"] == false), "{report}");
}

// Checks the attack of the example: frames 5 to 7 are copies of frame 1; the
// body of frame 5 is 1 pixel to the left, the body of frame 6 is 2 pixels to
// the right, the legs stay, frame 7 is equal to frame 1, and the tag `attack`
// has mode once and the durations of the three phases.
#[test]
fn the_walk_example_makes_a_three_frame_attack() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    make_walker(&core, &session);

    for _ in 0..3 {
        call(&core, &session, "add_frame", json!({ "name": "walker", "copy_of": 1 })).unwrap();
    }
    let body = json!({ "x": 0, "y": 1, "width": 12, "height": 7 });
    call(&core, &session, "transform_region", json!({ "name": "walker", "frame": 5, "operation": "move", "dx": -1, "dy": 0, "region": body })).unwrap();
    call(&core, &session, "transform_region", json!({ "name": "walker", "frame": 6, "operation": "move", "dx": 2, "dy": 0, "region": body })).unwrap();
    for (frame, milliseconds) in [(5, 200), (6, 60), (7, 120)] {
        call(&core, &session, "set_frame_duration", json!({ "name": "walker", "frame": frame, "milliseconds": milliseconds })).unwrap();
    }
    call(&core, &session, "create_tag", json!({ "sprite": "walker", "name": "attack", "from": 5, "to": 7, "fps": 10, "mode": "once" })).unwrap();

    call(&core, &session, "render_frames", json!({ "name": "walker", "from": 5, "to": 7, "labels": true })).unwrap();
    call(&core, &session, "render_onion", json!({ "name": "walker", "frames": [5, 6, 7] })).unwrap();

    let row = |frame: usize, y: usize| rows(&host, &[], "walker", frame).split('/').nth(y).unwrap().to_string();
    assert_eq!(row(1, 4), "..KBBBBEBK..");
    assert_eq!(row(5, 4), ".KBBBBEBK...", "leaning back");
    assert_eq!(row(6, 4), "....KBBBBEBK", "lunging");
    assert_eq!(row(5, 8), row(1, 8), "the legs do not move");
    assert_eq!(rows(&host, &[], "walker", 7), rows(&host, &[], "walker", 1));

    let sprite = call(&core, &session, "get_sprite", json!({ "name": "walker", "frame": 7 })).unwrap();
    assert_eq!(sprite["frame_durations"], json!([100, 100, 100, 100, 200, 60, 120]));
    assert_eq!(sprite["animations"][0]["mode"], "once", "{sprite}");
    let report = call(&core, &session, "analyze_sprite", json!({ "name": "walker", "outline_key": "K" })).unwrap();
    assert_eq!(report["frame_reports"][5]["loose_pixels"]["count"], 0, "{report}");
}
