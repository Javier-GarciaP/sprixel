//! Tests of the power tools of the editor. Most of them run the command and
//! then the MCP tool for an agent on the same picture, and check that both
//! give the same pixels.

use super::*;
use crate::ipc::browser;
use crate::mcp::testing::{TestHost, call, open_demo_project, replay_done, rows, setup};
use serde_json::json;

fn slime() -> SpriteArg {
    SpriteArg { folder: vec![], name: "slime".to_string() }
}

fn entries(host: &TestHost) -> usize {
    project::read(&host.project, |s| s.history.entries().len()).unwrap()
}

/// A host with the demo project and this picture on frame 1 of `slime` (8x8).
fn drawn() -> (std::sync::Arc<TestHost>, crate::mcp::Core, String) {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let grid = "frame 1\nRRG.....\nR.......\n........\n........\n........\n........\n........\n.......G\n";
    call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": grid })).unwrap();
    replay_done(&host);
    (host, core, session)
}

fn rect(x: u32, y: u32, width: u32, height: u32) -> Option<Rect> {
    Some(Rect { x, y, width, height })
}

/// The command, then the tool of the agent on the same picture. Returns the
/// frame after each one.
fn both(action: RegionAction, region: Option<Rect>, tool_args: serde_json::Value) -> (String, String) {
    let (host, core, session) = drawn();
    let before = entries(&host);
    region_apply(&host.project, &host.agents, &slime(), 1, region, &action).unwrap();
    assert_eq!(entries(&host), before + 1, "one history entry");
    let by_person = rows(&host, &[], "slime", 1);
    browser::undo(&host.project).unwrap();
    let mut args = json!({ "name": "slime" });
    args.as_object_mut().unwrap().extend(tool_args.as_object().unwrap().clone());
    call(&core, &session, "transform_region", args).unwrap();
    (by_person, rows(&host, &[], "slime", 1))
}

// Checks that every region action of a person gives the pixels the agent tool
// gives: one history entry each (checked in `both`).
#[test]
fn region_actions_equal_the_agent_tool() {
    let region = rect(0, 0, 4, 2);
    let r = json!({ "x": 0, "y": 0, "width": 4, "height": 2 });
    let cases = [
        (RegionAction::FlipHorizontal, region, json!({ "operation": "flip_horizontal", "region": r })),
        (RegionAction::FlipVertical, None, json!({ "operation": "flip_vertical" })),
        (RegionAction::Rotate { degrees: 90 }, rect(0, 0, 3, 3), json!({ "operation": "rotate", "degrees": 90, "region": { "x": 0, "y": 0, "width": 3, "height": 3 } })),
        (RegionAction::Rotate { degrees: 180 }, region, json!({ "operation": "rotate", "degrees": 180, "region": r })),
        (RegionAction::Move { dx: 2, dy: 1, wrap: true }, None, json!({ "operation": "move", "dx": 2, "dy": 1, "wrap": true })),
        (RegionAction::Move { dx: -1, dy: 0, wrap: false }, region, json!({ "operation": "move", "dx": -1, "dy": 0, "region": r })),
        (RegionAction::Mirror { direction: "left_to_right".into() }, None, json!({ "operation": "mirror", "direction": "left_to_right" })),
    ];
    for (action, region, args) in cases {
        let label = format!("{action:?}");
        let (person, agent) = both(action, region, args);
        assert_eq!(person, agent, "{label}");
    }
}

// Checks that one Undo restores the picture (the entry belongs to the user).
#[test]
fn a_region_action_is_undone_with_one_undo() {
    let (host, _core, _session) = drawn();
    let before = rows(&host, &[], "slime", 1);
    region_apply(&host.project, &host.agents, &slime(), 1, None, &RegionAction::FlipVertical).unwrap();
    assert_ne!(rows(&host, &[], "slime", 1), before);
    browser::undo(&host.project).unwrap();
    assert_eq!(rows(&host, &[], "slime", 1), before);
}

// Checks that an action that changes nothing (a flip of an empty frame) makes
// no history entry.
#[test]
fn nothing_to_change_makes_no_entry() {
    let (host, _core, _session) = drawn();
    open_demo_project_frame_two(&host);
    let before = entries(&host);
    let (applied, _) = region_apply(&host.project, &host.agents, &slime(), 2, None, &RegionAction::FlipHorizontal).unwrap();
    assert_eq!(applied.pixels_changed, 0);
    assert_eq!(entries(&host), before);
}

fn open_demo_project_frame_two(host: &TestHost) {
    project::edit(&host.project, |s| {
        s.apply(USER, vec![Operation::AddFrame { sprite: slime().to_ref(), duration_ms: 100 }]).unwrap();
    })
    .unwrap();
}

// Checks that a person cannot change a frame an agent is writing to.
#[test]
fn a_busy_frame_is_refused() {
    let (host, _core, _session) = drawn();
    host.agents.begin(crate::ipc::editor::AgentWrite { agent: "Claude Code".into(), sprite: slime(), frame: 1 });
    let before = rows(&host, &[], "slime", 1);
    let error = region_apply(&host.project, &host.agents, &slime(), 1, None, &RegionAction::FlipVertical).unwrap_err();
    assert_eq!(error.code, "frame_busy");
    assert_eq!(rows(&host, &[], "slime", 1), before);
}

// Checks the refusals: a rectangle cannot turn 90 degrees, a bad region and a
// bad direction change nothing.
#[test]
fn bad_actions_change_nothing() {
    let (host, _core, _session) = drawn();
    let before = rows(&host, &[], "slime", 1);
    let rotate = RegionAction::Rotate { degrees: 90 };
    let error = region_apply(&host.project, &host.agents, &slime(), 1, rect(0, 0, 4, 2), &rotate).unwrap_err();
    assert_eq!(error.code, "not_square");
    let error = region_apply(&host.project, &host.agents, &slime(), 1, rect(6, 6, 4, 4), &RegionAction::FlipVertical).unwrap_err();
    assert_eq!(error.code, "region_out");
    let sideways = RegionAction::Mirror { direction: "sideways".into() };
    assert_eq!(region_apply(&host.project, &host.agents, &slime(), 1, None, &sideways).unwrap_err().code, "bad_argument");
    let error = region_apply(&host.project, &host.agents, &slime(), 9, None, &RegionAction::FlipVertical).unwrap_err();
    assert_eq!(error.code, "bad_frame");
    assert_eq!(rows(&host, &[], "slime", 1), before);
}

// Checks that copy and paste give what `copy_region` gives, to another frame,
// and that the source frame does not change.
#[test]
fn paste_equals_copy_region() {
    let (host, core, session) = drawn();
    open_demo_project_frame_two(&host);
    let clip = region_copy(&host.project, &slime(), 1, rect(0, 0, 3, 2)).unwrap();
    assert_eq!(clip.keys, "RRGR..");
    let before = entries(&host);
    region_paste(&host.project, &host.agents, &slime(), 2, &clip, 4, 5, false).unwrap();
    assert_eq!(entries(&host), before + 1);
    let by_person = rows(&host, &[], "slime", 2);
    browser::undo(&host.project).unwrap();
    call(&core, &session, "copy_region", json!({
        "name": "slime", "frame": 1, "region": { "x": 0, "y": 0, "width": 3, "height": 2 }, "to_frame": 2, "to_x": 4, "to_y": 5,
    })).unwrap();
    assert_eq!(by_person, rows(&host, &[], "slime", 2));
    assert!(rows(&host, &[], "slime", 1).starts_with("RRG....."), "the source frame is unchanged");
}

// Checks `skip_transparent`: the `.` of the copy keep the pixels under them.
#[test]
fn paste_can_skip_transparent() {
    let (host, _core, _session) = drawn();
    let clip = region_copy(&host.project, &slime(), 1, rect(0, 0, 3, 2)).unwrap();
    // Pasted over itself one pixel down: the `.` of the copy must not erase.
    region_paste(&host.project, &host.agents, &slime(), 1, &clip, 0, 1, true).unwrap();
    assert_eq!(rows(&host, &[], "slime", 1).split('/').nth(1).unwrap(), "RRG.....");
}

// Checks that a paste in a sprite with another palette is refused with a
// clear code, and a paste that does not fit too.
#[test]
fn paste_checks_the_palette_and_the_size() {
    let (host, _core, _session) = drawn();
    let clip = region_copy(&host.project, &slime(), 1, rect(0, 0, 3, 2)).unwrap();
    project::edit(&host.project, |s| {
        let palette = crate::document::Palette { name: "other".into(), colors: vec![] };
        s.apply(USER, vec![
            Operation::CreatePalette { palette, index: None },
            Operation::CreateSprite { folder: vec![], name: "orc".into(), width: 8, height: 8, palette: "other".into() },
        ]).unwrap();
    })
    .unwrap();
    let orc = SpriteArg { folder: vec![], name: "orc".into() };
    let error = region_paste(&host.project, &host.agents, &orc, 1, &clip, 0, 0, false).unwrap_err();
    assert_eq!(error.code, "palette_mismatch");
    let error = region_paste(&host.project, &host.agents, &slime(), 1, &clip, 7, 7, false).unwrap_err();
    assert_eq!(error.code, "region_out");
}

// ---- outline, shade, review, tile check ----

fn outline(key: &str) -> OutlineArgs {
    OutlineArgs { key: key.into(), connectivity: 8, outside_only: false, inner: false }
}

// Checks that the Outline preview lists the pixels that Apply then changes,
// that the preview writes nothing (same revision, no entry), and that Apply
// gives the picture `auto_outline` gives to an agent.
#[test]
fn outline_preview_apply_and_the_agent_tool_agree() {
    let (host, core, session) = drawn();
    let (before, revision, rows_before) = (entries(&host), crate::mcp::testing::revision(&host), rows(&host, &[], "slime", 1));
    let preview = outline_preview(&host.project, &slime(), 1, &outline("G")).unwrap();
    assert!(!preview.pixels.is_empty());
    assert_eq!((entries(&host), crate::mcp::testing::revision(&host), rows(&host, &[], "slime", 1)), (before, revision, rows_before));

    outline_apply(&host.project, &host.agents, &slime(), 1, &outline("G")).unwrap();
    assert_eq!(entries(&host), before + 1);
    let by_person = rows(&host, &[], "slime", 1);
    let grid: Vec<&str> = by_person.split('/').collect();
    for (x, y, key) in &preview.pixels {
        assert_eq!(&grid[*y as usize][*x as usize..*x as usize + 1], key, "the preview shows what Apply writes");
    }
    browser::undo(&host.project).unwrap();
    call(&core, &session, "auto_outline", json!({ "name": "slime", "key": "G" })).unwrap();
    assert_eq!(by_person, rows(&host, &[], "slime", 1));
}

// Checks the same for Shade, and that a bad ramp is refused.
#[test]
fn shade_preview_apply_and_the_agent_tool_agree() {
    let (host, core, session) = drawn();
    let args = ShadeArgs { light: "top-left".into(), base: "R".into(), keys: vec!["G".into(), "R".into()], region: None };
    let preview = shade_preview(&host.project, &slime(), 1, &args).unwrap();
    assert_eq!(entries(&host), 1, "the preview made no entry");
    shade_apply(&host.project, &host.agents, &slime(), 1, &args).unwrap();
    let by_person = rows(&host, &[], "slime", 1);
    browser::undo(&host.project).unwrap();
    call(&core, &session, "auto_shade", json!({ "name": "slime", "light": "top-left", "base": "R", "keys": ["G", "R"] })).unwrap();
    assert_eq!(by_person, rows(&host, &[], "slime", 1));
    assert_eq!(preview.pixels.len(), by_person.matches('G').count() - 2, "only the shaded pixels are in the preview");

    let bad = ShadeArgs { keys: vec!["G".into()], ..args.clone() };
    assert_eq!(shade_preview(&host.project, &slime(), 1, &bad).unwrap_err().code, "bad_argument");
    let missing = ShadeArgs { base: "Z".into(), ..args };
    assert_eq!(shade_preview(&host.project, &slime(), 1, &missing).unwrap_err().code, "unknown_key");
}

// Checks the busy frame and the key checks of Outline.
#[test]
fn outline_refuses_a_busy_frame_and_a_bad_key() {
    let (host, _core, _session) = drawn();
    assert_eq!(outline_preview(&host.project, &slime(), 1, &outline(".")).unwrap_err().code, "unknown_key");
    host.agents.begin(crate::ipc::editor::AgentWrite { agent: "Claude Code".into(), sprite: slime(), frame: 1 });
    assert_eq!(outline_apply(&host.project, &host.agents, &slime(), 1, &outline("G")).unwrap_err().code, "frame_busy");
}

// Checks that the review finds what `analyze_sprite` finds.
#[test]
fn review_equals_analyze_sprite() {
    let (host, core, session) = drawn();
    let mine = review(&host.project, &slime(), Some("G"), 2).unwrap();
    let theirs = call(&core, &session, "analyze_sprite", json!({ "name": "slime", "outline_key": "G" })).unwrap();
    let frame = &theirs["frame_reports"][0];
    assert_eq!(mine.frames[0].loose.len() as u64, frame["loose_pixels"]["count"].as_u64().unwrap());
    assert_eq!(mine.frames[0].gaps.len() as u64, frame["outline_gaps"]["count"].as_u64().unwrap());
    assert_eq!(mine.unused.len(), theirs["colors"]["unused"].as_array().unwrap().len());
    // The demo picture has a loose pixel at the bottom right.
    assert!(mine.frames[0].loose.contains(&(7, 7)));
    assert!(review(&host.project, &slime(), Some("Z"), 2).is_err());
}

// Checks the comparison of frames when a sprite has more than one.
#[test]
fn review_compares_each_frame_with_the_next() {
    let (host, core, session) = drawn();
    open_demo_project_frame_two(&host);
    let mine = review(&host.project, &slime(), None, 2).unwrap();
    let theirs = call(&core, &session, "analyze_sprite", json!({ "name": "slime" })).unwrap();
    assert_eq!(mine.changes.len(), 1);
    assert_eq!(mine.changes[0].changed_pixels as u64, theirs["changes"][0]["changed_pixels"].as_u64().unwrap());
    assert_eq!(mine.changes[0].ground_kept, theirs["changes"][0]["ground_kept"].as_bool().unwrap());
}

// Checks that the tile check gives the verdict of `check_tile`.
#[test]
fn tile_check_equals_check_tile() {
    let (host, core, session) = drawn();
    let mine = tile_check(&host.project, &slime(), 1).unwrap();
    let theirs = call(&core, &session, "check_tile", json!({ "name": "slime" })).unwrap();
    assert_eq!(mine.seamless, theirs["seamless"].as_bool().unwrap());
    assert_eq!(mine.mismatches.len() as u64, theirs["mismatch_count"].as_u64().unwrap());
    assert!(!mine.seamless, "the demo picture has different edges");
}

// ---- palette, durations, anchors ----

fn main_colors(host: &TestHost) -> Vec<crate::document::Color> {
    project::read(&host.project, |s| s.project.palette("main").unwrap().colors.clone()).unwrap()
}

// Checks that Generate ramp adds the tones `generate_ramp` adds, that the
// preview gives the same hex values, and that it is one entry.
#[test]
fn ramp_equals_generate_ramp() {
    let (host, core, session) = drawn();
    let before = entries(&host);
    let tones = ramp_preview("#3366CC", 5, 20.0).unwrap();
    assert_eq!(tones.len(), 5);
    ramp_apply(&host.project, "main", "#3366cc", 5, 20.0).unwrap();
    assert_eq!(entries(&host), before + 1);
    let by_person = main_colors(&host);
    assert!(tones.iter().all(|t| by_person.iter().any(|c| &c.hex == t)), "the preview shows the added tones");
    browser::undo(&host.project).unwrap();
    call(&core, &session, "generate_ramp", json!({ "palette": "main", "base": "#3366CC", "steps": 5, "hue_shift": 20 })).unwrap();
    assert_eq!(by_person, main_colors(&host));
}

// Checks that a tone the palette already has is reused, not added twice, and
// that bad arguments are refused.
#[test]
fn ramp_reuses_tones_and_checks_arguments() {
    let (host, _core, _session) = drawn();
    ramp_apply(&host.project, "main", "#3366CC", 5, 0.0).unwrap();
    let count = main_colors(&host).len();
    ramp_apply(&host.project, "main", "#3366CC", 5, 0.0).unwrap();
    assert_eq!(main_colors(&host).len(), count);
    assert_eq!(ramp_preview("#3366CC", 2, 0.0).unwrap_err().code, "bad_argument");
    assert_eq!(ramp_apply(&host.project, "nope", "#3366CC", 5, 0.0).unwrap_err().code, "not_found");
}

// Checks Remove unused colors against `remove_unused_colors`, and that a
// palette with nothing unused makes no entry.
#[test]
fn unused_colors_are_removed_like_the_agent_tool() {
    let (host, core, session) = drawn();
    let original = main_colors(&host);
    let (none, _) = unused_remove(&host.project, "main").unwrap();
    assert!(none.is_empty());
    ramp_apply(&host.project, "main", "#3366CC", 5, 0.0).unwrap();
    let before = entries(&host);
    let (removed, _) = unused_remove(&host.project, "main").unwrap();
    assert_eq!(removed.len(), 5);
    assert_eq!(entries(&host), before + 1);
    assert_eq!(main_colors(&host), original);
    browser::undo(&host.project).unwrap();
    call(&core, &session, "remove_unused_colors", json!({ "palette": "main" })).unwrap();
    assert_eq!(main_colors(&host), original);
}

// Checks the frame duration against `set_frame_duration`, and its limits.
#[test]
fn duration_equals_set_frame_duration() {
    let (host, core, session) = drawn();
    open_demo_project_frame_two(&host);
    let read = |host: &TestHost| project::read(&host.project, |s| s.project.find_sprite(&slime().to_ref()).unwrap().frame_durations.clone()).unwrap();
    duration_set(&host.project, &slime(), 2, 200).unwrap();
    let by_person = read(&host);
    assert_eq!(by_person[1], 200);
    browser::undo(&host.project).unwrap();
    call(&core, &session, "set_frame_duration", json!({ "name": "slime", "frame": 2, "milliseconds": 200 })).unwrap();
    assert_eq!(by_person, read(&host));
    assert_eq!(duration_set(&host.project, &slime(), 2, 5).unwrap_err().code, "bad_argument");
    assert_eq!(duration_set(&host.project, &slime(), 9, 200).unwrap_err().code, "bad_frame");
}

// Checks anchors: create, move, rename (one entry for a rename) and delete,
// and that a point outside the sprite is refused.
#[test]
fn anchors_are_created_moved_renamed_and_deleted() {
    let (host, core, session) = drawn();
    let feet = AnchorArg { name: "feet".into(), x: 3, y: 7, frame: None };
    anchor_save(&host.project, &slime(), &feet, None).unwrap();
    let by_person = anchors_of(&host.project, &slime()).unwrap();
    assert_eq!(by_person, vec![feet.clone()]);
    browser::undo(&host.project).unwrap();
    call(&core, &session, "set_anchor", json!({ "sprite": "slime", "name": "feet", "x": 3, "y": 7 })).unwrap();
    assert_eq!(by_person, anchors_of(&host.project, &slime()).unwrap());

    let key = AnchorKey { name: "feet".into(), frame: None };
    let moved = AnchorArg { x: 4, ..feet.clone() };
    anchor_save(&host.project, &slime(), &moved, Some(&key)).unwrap();
    assert_eq!(anchors_of(&host.project, &slime()).unwrap(), vec![moved.clone()]);

    let before = entries(&host);
    let renamed = AnchorArg { name: "hand".into(), ..moved };
    anchor_save(&host.project, &slime(), &renamed, Some(&key)).unwrap();
    assert_eq!(entries(&host), before + 1, "a rename is one entry");
    assert_eq!(anchors_of(&host.project, &slime()).unwrap(), vec![renamed.clone()]);
    browser::undo(&host.project).unwrap();
    assert_eq!(anchors_of(&host.project, &slime()).unwrap()[0].name, "feet");

    let out = AnchorArg { x: 99, ..feet };
    assert_eq!(anchor_save(&host.project, &slime(), &out, None).unwrap_err().code, "bad_anchor");
    anchor_delete(&host.project, &slime(), &key).unwrap();
    assert!(anchors_of(&host.project, &slime()).unwrap().is_empty());
}
