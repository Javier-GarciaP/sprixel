//! Task 2.1: approval of new clients (pending, allowed, rejected), its
//! persistence and removal, and that tools are blocked before approval.

use super::testing::{connect, core_with, rpc, TestHost};
use super::*;
use serde_json::{Value, json};
use std::sync::Arc;

fn ran(ctx: &Ctx, _args: &Value) -> ToolResult {
    ctx.core.host.emit("tool-ran", json!(null));
    Ok(Output::Json(json!({ "ok": true })))
}

fn tool() -> Tool {
    Tool { name: "work", description: "Works.", schema: json!({ "type": "object" }), need: Need::Nothing, run: ran }
}

fn call(core: &Core, session: &str) -> Value {
    rpc(core, Some(session), "tools/call", json!({ "name": "work", "arguments": {} }))["result"].clone()
}

fn error_code(result: &Value) -> String {
    let text = result["content"][0]["text"].as_str().unwrap_or("{}");
    serde_json::from_str::<Value>(text).unwrap()["code"].as_str().unwrap_or("").to_string()
}

fn tools_listed(core: &Core, session: &str) -> usize {
    rpc(core, Some(session), "tools/list", json!({}))["result"]["tools"].as_array().unwrap().len()
}

fn events(host: &TestHost, name: &str) -> Vec<Value> {
    host.events.lock().unwrap().iter().filter(|(e, _)| e == name).map(|(_, v)| v.clone()).collect()
}

// Checks the "New client" scenario: a client that connects for the first time
// is pending, the app is told (so it shows Allow and Reject, and a system
// notification when it is in the background), and the client sees no tool and
// cannot call one until the user answers.
#[test]
fn a_new_client_is_pending_and_blocked() {
    let (host, core) = core_with(vec![tool()]);
    let session = connect(&core, "Claude Code");

    assert_eq!(events(&host, CLIENT_PENDING_EVENT), vec![json!({ "name": "Claude Code", "version": "1.0" })]);
    assert_eq!(core.clients(), vec![ClientInfo { name: "Claude Code".into(), version: "1.0".into(), state: "pending" }]);
    assert_eq!(tools_listed(&core, &session), 0);
    assert_eq!(error_code(&call(&core, &session)), "awaiting_approval");
    assert!(events(&host, "tool-ran").is_empty());
}

// Checks that the user is told once: a client that connects again while it
// waits does not send a second request.
#[test]
fn a_waiting_client_asks_only_once() {
    let (host, core) = core_with(vec![tool()]);
    connect(&core, "Claude Code");
    connect(&core, "Claude Code");
    assert_eq!(events(&host, CLIENT_PENDING_EVENT).len(), 1);
    assert_eq!(core.clients().len(), 1);
}

// Checks that Allow opens the tools to that client and to its other sessions.
#[test]
fn allow_opens_the_tools() {
    let (host, core) = core_with(vec![tool()]);
    let session = connect(&core, "Claude Code");

    assert!(core.answer_client("Claude Code", "1.0", true).unwrap());

    assert_eq!(tools_listed(&core, &session), 1);
    assert_eq!(call(&core, &session)["isError"], false);
    assert_eq!(events(&host, "tool-ran").len(), 1);
    assert_eq!(core.clients()[0].state, "allowed");
}

// Checks that Reject keeps the client out, with its own error, and that
// answering a client that does not wait changes nothing.
#[test]
fn reject_keeps_the_client_out() {
    let (host, core) = core_with(vec![tool()]);
    let session = connect(&core, "Claude Code");

    assert!(core.answer_client("Claude Code", "1.0", false).unwrap());
    assert!(!core.answer_client("Claude Code", "1.0", true).unwrap(), "already answered");
    assert!(!core.answer_client("Other", "1.0", true).unwrap(), "never asked");

    assert_eq!(tools_listed(&core, &session), 0);
    assert_eq!(error_code(&call(&core, &session)), "client_rejected");
    assert!(events(&host, "tool-ran").is_empty());
    // Connecting again does not ask again: the rejection is remembered.
    let again = connect(&core, "Claude Code");
    assert_eq!(error_code(&call(&core, &again)), "client_rejected");
    assert_eq!(events(&host, CLIENT_PENDING_EVENT).len(), 1);
}

// Checks that a client with no answer never runs a tool, even after many
// calls (the "unanswered client" rule).
#[test]
fn an_unanswered_client_never_runs_a_tool() {
    let (host, core) = core_with(vec![tool()]);
    let session = connect(&core, "Claude Code");
    for _ in 0..5 {
        assert_eq!(error_code(&call(&core, &session)), "awaiting_approval");
    }
    assert!(events(&host, "tool-ran").is_empty());
}

// Checks persistence: allowed and rejected clients are remembered after the
// app restarts (a new core on the same data folder); a pending client is not.
#[test]
fn answers_are_remembered_after_a_restart() {
    let (host, core) = core_with(vec![tool()]);
    connect(&core, "Allowed One");
    connect(&core, "Rejected One");
    connect(&core, "Pending One");
    core.answer_client("Allowed One", "1.0", true).unwrap();
    core.answer_client("Rejected One", "1.0", false).unwrap();
    drop(core);

    let restarted = Core::new(host.clone(), vec![tool()], host.dir.path().join(".sprixel")).unwrap();

    let state = |name: &str| restarted.clients().iter().find(|c| c.name == name).map(|c| c.state);
    assert_eq!(state("Allowed One"), Some("allowed"));
    assert_eq!(state("Rejected One"), Some("rejected"));
    assert_eq!(state("Pending One"), None);
    let session = connect(&restarted, "Allowed One");
    assert_eq!(call(&restarted, &session)["isError"], false);
}

// Checks that the clients file is private to the user, like the token.
#[cfg(unix)]
#[test]
fn the_clients_file_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let (host, core) = core_with(vec![tool()]);
    connect(&core, "Claude Code");
    core.answer_client("Claude Code", "1.0", true).unwrap();
    let path = host.dir.path().join(".sprixel").join(clients::CLIENTS_FILE);
    assert_eq!(std::fs::metadata(path).unwrap().permissions().mode() & 0o777, 0o600);
}

// Checks that a damaged clients file means "nobody is known", not a crash
// and not "everybody is allowed".
#[test]
fn a_damaged_clients_file_allows_nobody() {
    let (host, core) = core_with(vec![tool()]);
    drop(core);
    std::fs::write(host.dir.path().join(".sprixel").join(clients::CLIENTS_FILE), "{ not json").unwrap();

    let core = Core::new(host.clone(), vec![tool()], host.dir.path().join(".sprixel")).unwrap();

    assert!(core.clients().is_empty());
    let session = connect(&core, "Claude Code");
    assert_eq!(error_code(&call(&core, &session)), "awaiting_approval");
}

// Checks removal: a removed client is blocked on its very next call, its
// session ends, and when it connects again it has to ask again.
#[test]
fn a_removed_client_must_ask_again() {
    let (host, core) = core_with(vec![tool()]);
    let session = connect(&core, "Claude Code");
    core.answer_client("Claude Code", "1.0", true).unwrap();
    assert_eq!(call(&core, &session)["isError"], false);

    assert!(core.remove_client("Claude Code", "1.0").unwrap());
    assert!(!core.remove_client("Claude Code", "1.0").unwrap());

    let blocked = core.handle(Some(&session), json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": { "name": "work" } }).to_string().as_bytes());
    assert_eq!(blocked.status, 400, "the session ended");
    assert!(core.clients().is_empty());
    let again = connect(&core, "Claude Code");
    assert_eq!(error_code(&call(&core, &again)), "awaiting_approval");
    assert_eq!(events(&host, CLIENT_PENDING_EVENT).len(), 2);
}

// Checks that approval is for the name and the version together: a new
// version of the same client asks again.
#[test]
fn a_new_version_asks_again() {
    let (_host, core) = core_with(vec![tool()]);
    connect(&core, "Claude Code");
    core.answer_client("Claude Code", "1.0", true).unwrap();
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "clientInfo": { "name": "Claude Code", "version": "2.0" } } });

    let session = core.handle(None, body.to_string().as_bytes()).session.unwrap();

    assert_eq!(error_code(&call(&core, &session)), "awaiting_approval");
}

// Checks that a client cannot get another client's approval by changing the
// letter case or adding spaces to its name.
#[test]
fn a_look_alike_name_is_a_different_client() {
    let (_host, core) = core_with(vec![tool()]);
    connect(&core, "Claude Code");
    core.answer_client("Claude Code", "1.0", true).unwrap();

    let other = connect(&core, "claude code");

    assert_eq!(error_code(&call(&core, &other)), "awaiting_approval");
}
