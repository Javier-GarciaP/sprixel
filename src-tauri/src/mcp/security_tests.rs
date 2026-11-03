//! Task 5.1: tests for the findings of the security review. Each test says
//! which finding it covers (F1, F2...); the findings are written in the
//! commit message and in `PR_DESCRIPTION.md`.

use super::testing::{call, connect, connect_allowed, core_with, open_demo_project, revision, setup};
use super::*;
use serde_json::{Value, json};

fn initialize_as(core: &Core, name: &str, version: &str) -> Value {
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "clientInfo": { "name": name, "version": version } } });
    core.handle(None, body.to_string().as_bytes()).body.unwrap()
}

// F1: a client chooses its own name, and the name is shown to the person in
// the approval prompt. A direction mark or a zero-width space could make
// "Claude Code" look like another name, so they are removed from the name and
// from the version.
#[test]
fn f1_invisible_characters_are_removed_from_names_and_versions() {
    let (_host, core) = core_with(vec![]);
    initialize_as(&core, "Claude\u{202E} Code\u{200B}", "1.0\u{0}\u{202E}");

    let clients = core.clients();

    assert_eq!(clients.len(), 1);
    assert_eq!((clients[0].name.as_str(), clients[0].version.as_str()), ("Claude Code", "1.0"));
}

// F2: a program with the token could ask to connect with a new name again and
// again, to fill the list of the person (and the memory of the app). At most
// 16 clients wait for an answer; the next one is refused with `limit_exceeded`
// and its session does not stay.
#[test]
fn f2_the_list_of_waiting_clients_is_limited() {
    let (host, core) = core_with(vec![]);
    for i in 0..clients::MAX_PENDING {
        let reply = initialize_as(&core, &format!("client {i}"), "1");
        assert!(reply["result"].is_object(), "{i}");
    }
    let sessions = core.sessions.lock().unwrap().len();

    let refused = initialize_as(&core, "one more", "1");

    assert!(refused["error"]["message"].as_str().unwrap().starts_with("limit_exceeded"));
    assert_eq!(core.clients().len(), clients::MAX_PENDING);
    assert_eq!(core.sessions.lock().unwrap().len(), sessions, "the refused session was ended");
    // Removing one makes room.
    core.remove_client("client 0", "1").unwrap();
    assert!(initialize_as(&core, "one more", "1")["result"].is_object());
    let _ = host;
}

// F2: the same limit for the answers kept in the file when "ask me" is off.
#[test]
fn f2_the_file_of_answers_is_limited() {
    let dir = tempfile::tempdir().unwrap();
    let mut clients = clients::Clients::load(dir.path());
    for i in 0..clients::MAX_KNOWN {
        assert!(clients.request(&format!("c{i}"), "1", false).is_some());
    }
    assert!(clients.request("one more", "1", false).is_none());
    assert!(clients.request("c0", "1", false).is_some(), "a client that is known still works");
}

// F3: the log keeps names that come from the client. A huge name (or one with
// control characters) is cut and cleaned, so a client cannot use the log to
// fill the memory of the app.
#[test]
fn f3_the_activity_log_cuts_what_the_client_sent() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let huge = format!("a\u{7}{}", "x".repeat(1_000_000));

    call(&core, &session, "set_pixels", json!({ "name": huge, "pixels": [] })).unwrap_err();
    call(&core, &session, "create_sprite", json!({ "name": huge, "width": 4, "height": 4 })).unwrap_err();

    let log = core.activity();
    assert_eq!(log.len(), 2);
    for row in &log {
        assert!(row.entry.sprite.as_ref().unwrap().chars().count() <= 200);
        assert!(row.entry.summary.chars().count() <= 300);
        assert!(!row.entry.sprite.as_ref().unwrap().contains('\u{7}'));
    }
}

// F4: a bug in a tool (here a tool that panics) must not take the connection
// down or change the project. The client gets `internal_error` and the next
// call works.
#[test]
fn f4_a_panic_in_a_tool_becomes_an_error() {
    fn boom(_ctx: &Ctx, _args: &Value) -> ToolResult {
        panic!("a bug");
    }
    let mut tools = super::tools::all();
    tools.push(Tool { name: "boom", description: "Panics.", schema: json!({ "type": "object" }), need: Need::Nothing, run: boom });
    let (host, core) = core_with(tools);
    open_demo_project(&host);
    let session = connect_allowed(&core, "Claude Code");
    let before = revision(&host);

    let error = call(&core, &session, "boom", json!({})).unwrap_err();

    assert_eq!(error["code"], "internal_error");
    assert_eq!(revision(&host), before);
    assert!(call(&core, &session, "get_project", json!({})).is_ok());
}

// F5: where `export` may write, even inside the project. Hidden folders (like
// `.git`), the folders that hold the project files and names that are not
// valid on every system are refused; so is a naming pattern that makes such a
// name. Nothing is written.
#[test]
fn f5_export_refuses_hidden_data_and_invalid_folders() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let root = crate::ipc::project::read(&host.project, |s| s.root().to_path_buf()).unwrap();
    let before = std::fs::read_dir(&root).unwrap().count();
    let run = |args: Value| call(&core, &session, "export", args).unwrap_err();

    for output in [".git/hooks", "a/.hidden", "sprites", "Sprites/Characters", "palettes", "assets/CON", "assets/nul.txt", "assets/dir.", "assets/ x"] {
        assert_eq!(run(json!({ "output": output }))["code"], "invalid_path", "{output}");
    }
    assert_eq!(run(json!({ "naming": "CON" }))["code"], "bad_options");
    assert_eq!(run(json!({ "naming": "{sprite}." }))["code"], "bad_options");

    assert_eq!(std::fs::read_dir(&root).unwrap().count(), before, "nothing was created");
    // A normal folder still works, also inside `assets` and with sub-folders.
    assert!(call(&core, &session, "export", json!({ "output": "assets/my art/v1" })).is_ok());
}

// F5: the size of an export is limited. Five frames of 512x512 at scale 8 are
// 84 million pixels; the tool says so before it draws anything.
#[test]
fn f5_a_huge_export_is_refused_before_it_is_built() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "create_sprite", json!({ "name": "huge", "width": 512, "height": 512 })).unwrap();
    for _ in 0..4 {
        call(&core, &session, "add_frame", json!({ "name": "huge" })).unwrap();
    }

    let error = call(&core, &session, "export", json!({ "scope": "sprites", "sprites": ["huge"], "scale": 8 })).unwrap_err();

    assert_eq!(error["code"], "too_big");
    assert_eq!(error["limit"], 64 * 1024 * 1024);
    let root = crate::ipc::project::read(&host.project, |s| s.root().to_path_buf()).unwrap();
    assert!(!root.join("assets").exists());
    // The same sprites at a smaller scale are fine.
    assert!(call(&core, &session, "export", json!({ "scope": "sprites", "sprites": ["huge"], "scale": 2 })).is_ok());
}

// Regression of the main rules in one place (the details are tested in each
// task): a client that is not approved has no tool, and an approved one cannot
// skip the permissions, with the same session.
#[test]
fn the_main_boundaries_hold_together() {
    let (host, core) = core_with(super::tools::all());
    open_demo_project(&host);
    let session = connect(&core, "Claude Code");
    let list = |core: &Core, s: &str| super::testing::rpc(core, Some(s), "tools/list", json!({}))["result"]["tools"].as_array().unwrap().len();

    assert_eq!(list(&core, &session), 0);
    assert_eq!(call(&core, &session, "delete_sprite", json!({ "name": "slime" })).unwrap_err()["code"], "awaiting_approval");
    core.answer_client("Claude Code", "1.0", true).unwrap();
    assert!(list(&core, &session) > 10);
    assert_eq!(call(&core, &session, "delete_sprite", json!({ "name": "slime" })).unwrap_err()["code"], "permission_denied");
    assert_eq!(call(&core, &session, "get_project", json!({})).unwrap()["sprite_count"], 2);
}
