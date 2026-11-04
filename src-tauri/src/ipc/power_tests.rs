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
