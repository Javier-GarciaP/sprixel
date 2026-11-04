//! Task 3.6: revisions. Every write tool returns the new revision and takes
//! an optional `base_revision`.

use crate::mcp::Permissions;
use crate::mcp::testing::{call, open_demo_project, revision, rows, setup};
use serde_json::{Value, json};

/// A valid call of every write tool.
fn write_calls() -> Vec<(&'static str, Value)> {
    vec![
        ("create_folder", json!({ "name": "New" })),
        ("create_sprite", json!({ "name": "orc", "width": 4, "height": 4 })),
        ("add_frame", json!({ "name": "goblin" })),
        ("apply_grid", json!({ "name": "slime", "grid": format!("frame 1\n{}", "R.......\n".repeat(8)) })),
        ("set_pixels", json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "G" }] })),
        ("draw_shape", json!({ "name": "slime", "shape": "line", "key": "G", "x0": 0, "y0": 0, "x1": 7, "y1": 0 })),
        ("delete_frame", json!({ "name": "goblin", "frame": 2 })),
        ("create_tag", json!({ "sprite": "slime", "name": "idle", "from": 1, "to": 1, "fps": 6 })),
        ("set_palette_color", json!({ "key": "B", "hex": "#0000FF" })),
        ("delete_sprite", json!({ "name": "slime" })),
    ]
}

fn with_base(mut args: Value, base: u64) -> Value {
    args["base_revision"] = json!(base);
    args
}

// Checks the "Stale revision" scenario for EVERY write tool: a call with a
// base_revision older than the project fails with `stale_revision`, says the
// current revision, and changes nothing (not the project, not the history).
#[test]
fn every_write_tool_fails_with_a_stale_revision() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(Permissions { delete: true, ..core.permissions() }).unwrap();
    // Move the project to a newer revision than the agent read.
    let old = revision(&host);
    call(&core, &session, "create_folder", json!({ "name": "Newer" })).unwrap();
    call(&core, &session, "create_folder", json!({ "name": "Newest" })).unwrap();
    let current = revision(&host);
    assert_eq!(current, old + 2);
    let entries = crate::ipc::project::read(&host.project, |s| s.history.entries().len()).unwrap();

    for (tool, args) in write_calls() {
        let error = call(&core, &session, tool, with_base(args, old)).unwrap_err();
        assert_eq!(error["code"], "stale_revision", "{tool}");
        assert_eq!(error["base_revision"], old, "{tool}");
        assert_eq!(error["revision"], current, "{tool}");
        assert!(error["message"].as_str().unwrap().contains("read it again"), "{tool}");
    }

    assert_eq!(revision(&host), current);
    assert_eq!(crate::ipc::project::read(&host.project, |s| s.history.entries().len()).unwrap(), entries);
    assert_eq!(rows(&host, &[], "slime", 1), "......../......../......../......../......../......../......../........");
}

// Checks that the exact revision is accepted by every write tool, and that
// every answer has the new revision, which is higher.
#[test]
fn every_write_tool_accepts_the_current_revision_and_returns_the_new_one() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(Permissions { delete: true, ..core.permissions() }).unwrap();

    for (tool, args) in write_calls() {
        let before = revision(&host);
        let done = call(&core, &session, tool, with_base(args, before)).unwrap_or_else(|e| panic!("{tool}: {e}"));
        assert_eq!(done["revision"], revision(&host), "{tool}");
        assert!(done["revision"].as_u64().unwrap() > before, "{tool}");
    }
}

// Checks that a chain works the way an agent uses it: it passes the revision
// of each answer to the next call.
#[test]
fn an_agent_can_chain_calls_with_the_revision_of_the_last_answer() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let mut base = call(&core, &session, "get_project", json!({})).unwrap()["revision"].as_u64().unwrap();

    for x in 0..4 {
        let done = call(&core, &session, "set_pixels", json!({ "name": "slime", "base_revision": base, "pixels": [{ "x": x, "y": 0, "key": "R" }] })).unwrap();
        base = done["revision"].as_u64().unwrap();
    }

    assert!(rows(&host, &[], "slime", 1).starts_with("RRRR"));
}

// Checks that a revision from the future is stale too, and that the
// argument must be a whole number.
#[test]
fn a_revision_from_the_future_or_a_bad_one_is_refused() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let now = revision(&host);
    let pixels = json!([{ "x": 0, "y": 0, "key": "R" }]);

    let future = call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": pixels, "base_revision": now + 5 })).unwrap_err();
    assert_eq!(future["code"], "stale_revision");
    for bad in [json!("12"), json!(-1), json!(1.5), json!(true)] {
        let error = call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": pixels, "base_revision": bad })).unwrap_err();
        assert_eq!(error["code"], "bad_argument", "{bad}");
    }
    assert_eq!(revision(&host), now);
}

// Checks that the person's own change also makes an agent's call stale: the
// agent read, the person undid something, and the agent's write is refused.
#[test]
fn a_change_by_the_person_makes_the_agent_stale() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let read = call(&core, &session, "get_sprite", json!({ "name": "slime" })).unwrap()["revision"].as_u64().unwrap();
    crate::ipc::project::edit(&host.project, |s| {
        s.apply("user", vec![crate::document::Operation::CreateFolder { parent: vec![], name: "Mine".into() }]).unwrap();
    })
    .unwrap();

    let error = call(&core, &session, "set_pixels", json!({ "name": "slime", "base_revision": read, "pixels": [{ "x": 0, "y": 0, "key": "R" }] })).unwrap_err();

    assert_eq!(error["code"], "stale_revision");
}

// Checks the rule under pressure: eight threads write at the same time with
// the same base revision. The writes are applied one at a time, so exactly
// one wins and the other seven are stale; none is half applied.
#[test]
fn concurrent_writes_with_the_same_base_revision_let_only_one_win() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let base = revision(&host);
    let core = std::sync::Arc::new(core);

    let results: Vec<Result<Value, Value>> = (0..8u32)
        .map(|x| {
            let core = core.clone();
            let session = session.clone();
            std::thread::spawn(move || {
                call(&core, &session, "set_pixels", json!({ "name": "slime", "base_revision": base, "pixels": [{ "x": x, "y": 0, "key": "R" }] }))
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();

    let wins = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(wins, 1);
    assert!(results.iter().filter_map(|r| r.as_ref().err()).all(|e| e["code"] == "stale_revision"));
    assert_eq!(revision(&host), base + 1);
    assert_eq!(rows(&host, &[], "slime", 1).matches('R').count(), 1);
}
