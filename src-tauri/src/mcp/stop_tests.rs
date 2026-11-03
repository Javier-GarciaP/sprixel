//! Task 4.2: busy frames, user strokes that wait, and Stop with
//! `stopped_by_user`, checked with a simulated agent: a write call that is
//! paused in the middle (the test hook of `Core::pause`).

use super::*;
use super::testing::{TestHost, call, open_demo_project, revision, rows, setup};
use crate::ipc::editor::{self, AGENT_EVENT, AgentWrite, Change, SpriteArg};
use crate::ipc::project;
use serde_json::json;
use std::sync::mpsc::{channel, sync_channel};
use std::sync::{Arc, Mutex};

fn goblin() -> SpriteArg {
    SpriteArg { folder: vec!["Characters".into()], name: "goblin".into() }
}

fn stroke(x: u32, y: u32) -> Vec<Change> {
    vec![Change { x, y, color: Some("#00FF00".into()) }]
}

/// Makes the next write call stop in the middle, after its frame is marked
/// busy, until the test sends on the returned sender. The receiver tells the
/// test when the call has reached that point.
fn pause_in_the_middle(core: &Core) -> (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) {
    let (release, wait) = channel::<()>();
    let (reached, reached_rx) = sync_channel::<()>(1);
    let wait = Mutex::new(wait);
    *core.pause.lock().unwrap() = Some(Arc::new(move || {
        let _ = reached.send(());
        let _ = wait.lock().unwrap().recv();
    }));
    (release, reached_rx)
}

/// Every sprite of the project follows the rules of its palette.
fn project_is_valid(host: &TestHost) -> bool {
    fn check(project: &crate::document::Project, folder: &crate::document::Folder) -> bool {
        folder.sprites.iter().all(|s| project.palette(&s.palette).is_some_and(|p| s.validate(p).is_ok()))
            && folder.folders.iter().all(|f| check(project, f))
    }
    project::read(&host.project, |s| check(&s.project, &s.project.root)).unwrap()
}

fn busy_events(host: &TestHost) -> Vec<serde_json::Value> {
    host.events.lock().unwrap().iter().filter(|(e, _)| e == AGENT_EVENT).map(|(_, v)| v.clone()).collect()
}

// Checks the "Busy frame" scenario with an agent in the middle of a write to
// frame 2: the frame is marked busy and says who is drawing; when the call
// ends the marker is gone.
#[test]
fn a_frame_is_busy_while_the_agent_writes() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let core = Arc::new(core);
    let (release, reached) = pause_in_the_middle(&core);
    let agent = {
        let (core, session) = (core.clone(), session.clone());
        std::thread::spawn(move || call(&core, &session, "set_pixels", json!({ "name": "goblin", "frame": 2, "pixels": [{ "x": 0, "y": 0, "key": "R" }] })))
    };
    reached.recv().unwrap();

    let writing = host.agents.writing().expect("the frame is busy");
    assert_eq!(writing, AgentWrite { agent: "Claude Code".into(), sprite: goblin(), frame: 2 });
    assert_eq!(busy_events(&host)[0]["agent"], "Claude Code");
    assert_eq!(busy_events(&host)[0]["frame"], 2);

    release.send(()).unwrap();
    agent.join().unwrap().unwrap();
    assert!(host.agents.writing().is_some(), "the marker stays until the replay is done");
    super::testing::replay_done(&host);
    assert_eq!(host.agents.writing(), None);
    assert!(busy_events(&host).last().unwrap().is_null());
}

// Checks the "User draws on a busy frame" scenario: the stroke of the person
// on the frame the agent writes is refused with `frame_busy` (the editor keeps
// it and sends it again), another frame is not blocked, and right after the
// agent's call ends the same stroke is applied. Both changes are in the
// project, in this order.
#[test]
fn a_user_stroke_waits_for_the_agent_and_is_applied_after() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let core = Arc::new(core);
    let (release, reached) = pause_in_the_middle(&core);
    let agent = {
        let (core, session) = (core.clone(), session.clone());
        std::thread::spawn(move || call(&core, &session, "set_pixels", json!({ "name": "goblin", "frame": 2, "pixels": [{ "x": 0, "y": 0, "key": "R" }] })))
    };
    reached.recv().unwrap();

    let refused = editor::apply_changes(&host.project, &host.agents, &goblin(), 2, &stroke(3, 0)).unwrap_err();
    assert_eq!(refused.code, "frame_busy");
    // Another frame is not marked busy (its stroke would only wait for the
    // project lock, because the core applies changes one at a time).
    assert!(!host.agents.blocks(&goblin(), 1));

    release.send(()).unwrap();
    agent.join().unwrap().unwrap();
    // Still busy until the editor says the replay is done.
    assert_eq!(editor::apply_changes(&host.project, &host.agents, &goblin(), 2, &stroke(3, 0)).unwrap_err().code, "frame_busy");
    super::testing::replay_done(&host);
    editor::apply_changes(&host.project, &host.agents, &goblin(), 2, &stroke(3, 0)).unwrap();
    editor::apply_changes(&host.project, &host.agents, &goblin(), 1, &stroke(2, 0)).unwrap();
    assert_eq!(rows(&host, &["Characters"], "goblin", 2), "R..G/..../..../....");
    let authors: Vec<String> = project::read(&host.project, |s| s.history.entries().iter().map(|e| e.author.clone()).collect()).unwrap();
    assert_eq!(authors, vec!["Claude Code", "user", "user"]);
}

// Checks the "Stop while drawing" scenario: the person presses Stop while the
// agent's call is in progress. The call ends WITHOUT applying anything, later
// write calls fail with `stopped_by_user`, and the project is valid and has
// the same revision and history as before.
#[test]
fn stop_cancels_the_call_in_progress_and_later_calls_fail() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let core = Arc::new(core);
    let before = (revision(&host), project::read(&host.project, |s| s.history.entries().len()).unwrap());
    let rows_before = rows(&host, &["Characters"], "goblin", 2);
    let (release, reached) = pause_in_the_middle(&core);
    let agent = {
        let (core, session) = (core.clone(), session.clone());
        std::thread::spawn(move || call(&core, &session, "apply_grid", json!({ "name": "goblin", "frame": 2, "grid": "RRRR\nRRRR\nRRRR\nRRRR\n" })))
    };
    reached.recv().unwrap();

    host.agents.stop();
    assert_eq!(host.agents.writing(), None, "the frame is free at once");
    release.send(()).unwrap();

    let error = agent.join().unwrap().unwrap_err();
    assert_eq!(error["code"], "stopped_by_user");
    assert_eq!(rows(&host, &["Characters"], "goblin", 2), rows_before, "nothing is half applied");
    assert_eq!((revision(&host), project::read(&host.project, |s| s.history.entries().len()).unwrap()), before);
    assert!(project_is_valid(&host));
    *core.pause.lock().unwrap() = None;
    for (tool, args) in [
        ("set_pixels", json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "R" }] })),
        ("create_folder", json!({ "name": "X" })),
        ("export", json!({})),
    ] {
        assert_eq!(call(&core, &session, tool, args).unwrap_err()["code"], "stopped_by_user", "{tool}");
    }
    assert_eq!(revision(&host), before.0);
    // Reading is still allowed: the agent can look at what is there.
    assert!(call(&core, &session, "get_sprite", json!({ "name": "slime" })).is_ok());
}

// Checks that Stop from the Agents tab stops an agent that is not writing at
// this moment, applies to every session of that agent (by name) and to no
// other agent, and that "let it continue" opens the writes again.
#[test]
fn stop_by_name_applies_until_the_person_lets_the_agent_continue() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let other = testing::connect_allowed(&core, "OpenCode");
    let same_agent_again = testing::connect(&core, "Claude Code"); // already allowed
    let write = json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "R" }] });

    core.stop_agent("Claude Code");

    assert_eq!(call(&core, &session, "set_pixels", write.clone()).unwrap_err()["code"], "stopped_by_user");
    assert_eq!(call(&core, &same_agent_again, "set_pixels", write.clone()).unwrap_err()["code"], "stopped_by_user");
    assert!(call(&core, &other, "set_pixels", write.clone()).is_ok(), "another agent is not stopped");
    assert!(core.clients().iter().find(|c| c.name == "Claude Code").unwrap().stopped);
    assert!(!core.clients().iter().find(|c| c.name == "OpenCode").unwrap().stopped);

    core.resume_agent("Claude Code");

    assert!(call(&core, &session, "set_pixels", write).is_ok());
    assert!(!core.clients().iter().any(|c| c.stopped));
}

// Checks that a new write call by the same agent does not clear the Stop (the
// old flag was cleared by a new write; the stop now lasts until resume).
#[test]
fn a_stop_is_not_cleared_by_a_new_write() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    host.agents.begin(AgentWrite { agent: "Claude Code".into(), sprite: goblin(), frame: 1 });
    host.agents.stop();
    host.agents.begin(AgentWrite { agent: "Claude Code".into(), sprite: goblin(), frame: 1 });
    host.agents.end();

    let error = call(&core, &session, "create_folder", json!({ "name": "X" })).unwrap_err();

    assert_eq!(error["code"], "stopped_by_user");
}

// Checks that a stopped write is in the activity log with its error, and that
// the message tells the AI what to do.
#[test]
fn a_stopped_call_is_logged_and_the_message_helps() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.stop_agent("Claude Code");

    let error = call(&core, &session, "create_folder", json!({ "name": "X" })).unwrap_err();

    assert!(error["message"].as_str().unwrap().contains("Do not continue"));
    let log = core.activity();
    assert_eq!(log[0].entry.error.as_deref(), Some("stopped_by_user"));
}
