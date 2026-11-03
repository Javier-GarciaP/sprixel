//! Task 4.3: `undo_last_ai_action` and the rule of when Undo is offered.

use crate::ipc::{browser, editor, project};
use crate::mcp::testing::{call, connect_allowed, core_with, open_demo_project, revision, rows, setup};
use crate::mcp::{Permissions, tools};
use serde_json::json;

fn draw(core: &crate::mcp::Core, session: &str, x: u32) -> serde_json::Value {
    call(core, session, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": x, "y": 0, "key": "R" }] })).unwrap()
}

fn undo_buttons(core: &crate::mcp::Core) -> Vec<bool> {
    core.activity().iter().map(|row| row.can_undo).collect()
}

// Checks the "Undo the last action" scenario: right after its own drawing
// action, the agent undoes it; the drawing is reverted (all of the call, in
// one step), the revision grows and the UI is told.
#[test]
fn the_agent_undoes_its_own_last_action() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let blank = rows(&host, &[], "slime", 1);
    call(&core, &session, "draw_shape", json!({ "name": "slime", "shape": "rectangle", "key": "R", "x": 0, "y": 0, "width": 8, "height": 8, "filled": true })).unwrap();
    let drawn = revision(&host);
    assert_ne!(rows(&host, &[], "slime", 1), blank);

    let done = call(&core, &session, "undo_last_ai_action", json!({})).unwrap();

    assert_eq!(rows(&host, &[], "slime", 1), blank);
    assert_eq!(done["revision"], revision(&host));
    assert!(revision(&host) > drawn, "undo is a change too");
    let events = host.events.lock().unwrap();
    assert!(events.iter().any(|(e, p)| e == crate::ipc::browser::CHANGED_EVENT && p["revision"] == done["revision"]));
}

// Checks the `newer_changes` case: after the agent drew, the person drew.
// The agent cannot undo (it would hide the work of the person), the error
// says who made the latest change, and nothing changes.
#[test]
fn a_change_by_the_person_after_the_action_blocks_the_undo() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session, 0);
    crate::mcp::testing::replay_done(&host);
    editor::apply_changes(
        &host.project,
        &host.agents,
        &editor::SpriteArg { folder: vec![], name: "slime".into() },
        1,
        &[editor::Change { x: 5, y: 5, color: Some("#00FF00".into()) }],
    )
    .unwrap();
    let before = (revision(&host), rows(&host, &[], "slime", 1));

    let error = call(&core, &session, "undo_last_ai_action", json!({})).unwrap_err();

    assert_eq!(error["code"], "newer_changes");
    assert_eq!(error["latest_by"], "the person");
    assert_eq!((revision(&host), rows(&host, &[], "slime", 1)), before);
}

// Checks that an agent does not undo the work of ANOTHER agent: the latest
// change belongs to OpenCode, so Claude Code gets `newer_changes`.
#[test]
fn an_agent_does_not_undo_another_agent() {
    let (host, core) = core_with(tools::all());
    open_demo_project(&host);
    let claude = connect_allowed(&core, "Claude Code");
    let opencode = connect_allowed(&core, "OpenCode");
    draw(&core, &claude, 0);
    draw(&core, &opencode, 1);
    let before = rows(&host, &[], "slime", 1);

    let error = call(&core, &claude, "undo_last_ai_action", json!({})).unwrap_err();

    assert_eq!(error["code"], "newer_changes");
    assert_eq!(error["latest_by"], "OpenCode");
    assert_eq!(rows(&host, &[], "slime", 1), before);
    call(&core, &opencode, "undo_last_ai_action", json!({})).unwrap();
}

// Checks the rest of the rules: nothing to undo, repeated undo goes back one
// action at a time, the permission, base_revision, and a stopped agent.
#[test]
fn undo_rules() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    assert_eq!(call(&core, &session, "undo_last_ai_action", json!({})).unwrap_err()["code"], "nothing_to_undo");

    draw(&core, &session, 0);
    draw(&core, &session, 1);
    call(&core, &session, "undo_last_ai_action", json!({})).unwrap();
    assert!(rows(&host, &[], "slime", 1).starts_with("R."));
    call(&core, &session, "undo_last_ai_action", json!({})).unwrap();
    assert!(rows(&host, &[], "slime", 1).starts_with(".."));
    assert_eq!(call(&core, &session, "undo_last_ai_action", json!({})).unwrap_err()["code"], "nothing_to_undo");

    draw(&core, &session, 0);
    let stale = call(&core, &session, "undo_last_ai_action", json!({ "base_revision": 0 })).unwrap_err();
    assert_eq!(stale["code"], "stale_revision");
    core.set_permissions(Permissions { edit: false, ..core.permissions() }).unwrap();
    assert_eq!(call(&core, &session, "undo_last_ai_action", json!({})).unwrap_err()["code"], "permission_denied");
    core.set_permissions(Permissions::default()).unwrap();
    core.stop_agent("Claude Code");
    assert_eq!(call(&core, &session, "undo_last_ai_action", json!({})).unwrap_err()["code"], "stopped_by_user");
    assert!(rows(&host, &[], "slime", 1).starts_with("R."), "the action is still there");
}

// Checks the Undo rule of the activity list ("An older row"): only the most
// recent AI action offers Undo, older rows never do, and failed actions and
// actions that changed nothing never do.
#[test]
fn only_the_latest_action_offers_undo() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    assert_eq!(undo_buttons(&core), Vec::<bool>::new());

    draw(&core, &session, 0);
    assert_eq!(undo_buttons(&core), vec![true]);
    draw(&core, &session, 1);
    assert_eq!(undo_buttons(&core), vec![true, false], "newest first: only the newest row");

    call(&core, &session, "set_pixels", json!({ "name": "ghost", "pixels": [] })).unwrap_err();
    assert_eq!(undo_buttons(&core), vec![false, true, false], "a failed call is not undoable and does not take the button");
    call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": [] })).unwrap();
    assert_eq!(undo_buttons(&core), vec![false, false, true, false], "a call that changed nothing has no Undo");
}

// Checks that the Undo button goes away when the person changes something
// after the agent, and also after an undo by the person.
#[test]
fn undo_is_not_offered_after_a_newer_change() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session, 0);
    assert_eq!(undo_buttons(&core), vec![true]);

    browser::create_folder(&host.project, &[], "Mine").unwrap();
    assert_eq!(undo_buttons(&core), vec![false]);

    browser::undo(&host.project).unwrap();
    assert_eq!(undo_buttons(&core), vec![false], "the revision is not the one of the action any more");
}

// Checks that after the agent's own undo no row offers Undo, and that the
// undo itself is in the log.
#[test]
fn after_an_undo_the_log_has_the_undo_and_no_button() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    draw(&core, &session, 0);

    call(&core, &session, "undo_last_ai_action", json!({})).unwrap();

    assert_eq!(undo_buttons(&core), vec![false, false]);
    assert_eq!(core.activity()[0].entry.summary, "Undid the last AI action");
    assert_eq!(project::read(&host.project, |s| s.history.entries().len()).unwrap(), 0);
}
