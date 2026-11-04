//! Task 2.2: the permission policy and `permission_denied`.

use super::testing::{connect_allowed, core_with, rpc};
use super::*;
use serde_json::{Value, json};

fn ran(ctx: &Ctx, _args: &Value) -> ToolResult {
    ctx.core.host.emit("tool-ran", json!(null));
    Ok(Output::Json(json!({ "ok": true })))
}

fn tool(name: &'static str, need: Need) -> Tool {
    Tool { name, description: "Test.", schema: json!({ "type": "object" }), need, run: ran }
}

fn tools() -> Vec<Tool> {
    vec![
        tool("read", Need::Nothing),
        tool("make", Need::Create),
        tool("change", Need::Edit),
        tool("remove", Need::Delete),
        tool("send", Need::Export),
        tool("open", Need::OpenProject),
    ]
}

/// The code of the error of a call, or `""` when it worked.
fn code(core: &Core, session: &str, name: &str) -> String {
    let result = &rpc(core, Some(session), "tools/call", json!({ "name": name }))["result"];
    if result["isError"] == false {
        return String::new();
    }
    let text = result["content"][0]["text"].as_str().unwrap();
    serde_json::from_str::<Value>(text).unwrap()["code"].as_str().unwrap().to_string()
}

fn runs(host: &testing::TestHost) -> usize {
    host.events.lock().unwrap().iter().filter(|(e, _)| e == "tool-ran").count()
}

// Checks the defaults of the docs: create, edit and export on, delete off,
// ask on connect on.
#[test]
fn the_defaults_are_the_safe_ones() {
    assert_eq!(
        Permissions::default(),
        Permissions { create: true, edit: true, delete: false, export: true, open_project: false, ask_on_connect: true }
    );
}

// Checks the "Delete is off" scenario: a delete tool is denied with
// `permission_denied`, the tool does not run, and the message names the
// setting so the AI can tell the user.
#[test]
fn delete_is_off_by_default() {
    let (host, core) = core_with(tools());
    let session = connect_allowed(&core, "Claude Code");

    let result = &rpc(&core, Some(&session), "tools/call", json!({ "name": "remove" }))["result"];

    assert_eq!(result["isError"], true);
    let error: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(error["code"], "permission_denied");
    assert_eq!(error["permission"], "delete");
    assert!(error["message"].as_str().unwrap().contains("Delete sprites"));
    assert_eq!(runs(&host), 0);
}

// Checks each of the four permissions on its own: turning one off denies
// only the tools that need it, and read tools always work.
#[test]
fn each_permission_denies_only_its_tools() {
    let cases = [("create", "make"), ("edit", "change"), ("delete", "remove"), ("export", "send"), ("openProject", "open")];
    for (switch, denied) in cases {
        let (_host, core) = core_with(tools());
        let session = connect_allowed(&core, "Claude Code");
        // Everything on, then one switch off.
        let mut all_on = Permissions { create: true, edit: true, delete: true, export: true, open_project: true, ask_on_connect: true };
        core.set_permissions(all_on).unwrap();
        for name in ["read", "make", "change", "remove", "send", "open"] {
            assert_eq!(code(&core, &session, name), "", "{name} with all on");
        }
        match switch {
            "create" => all_on.create = false,
            "edit" => all_on.edit = false,
            "delete" => all_on.delete = false,
            "openProject" => all_on.open_project = false,
            _ => all_on.export = false,
        }
        core.set_permissions(all_on).unwrap();
        for name in ["read", "make", "change", "remove", "send", "open"] {
            let expected = if name == denied { "permission_denied" } else { "" };
            assert_eq!(code(&core, &session, name), expected, "{name} with {switch} off");
        }
    }
}

// Checks the "Turn export off" scenario: the permission changes for a client
// that is already connected, on its very next call, with no reconnecting.
#[test]
fn a_change_applies_to_the_next_call_without_reconnecting() {
    let (_host, core) = core_with(tools());
    let session = connect_allowed(&core, "Claude Code");
    assert_eq!(code(&core, &session, "send"), "");

    core.set_permissions(Permissions { export: false, ..core.permissions() }).unwrap();
    assert_eq!(code(&core, &session, "send"), "permission_denied");

    core.set_permissions(Permissions { export: true, ..core.permissions() }).unwrap();
    assert_eq!(code(&core, &session, "send"), "");
}

// Checks the scenario "Turn it on": "Open projects" is off by default, and
// turning it on works for a client that is already connected, on its next
// call. An old permissions file without the field also gives "off".
#[test]
fn open_projects_is_off_and_changes_without_reconnecting() {
    let (host, core) = core_with(tools());
    let session = connect_allowed(&core, "Claude Code");
    assert_eq!(code(&core, &session, "open"), "permission_denied");

    core.set_permissions(Permissions { open_project: true, ..core.permissions() }).unwrap();
    assert_eq!(code(&core, &session, "open"), "");

    let dir = host.dir.path().join(".sprixel");
    std::fs::write(dir.join(policy::PERMISSIONS_FILE), r#"{"create":true,"delete":true}"#).unwrap();
    let old = Core::new(host.clone(), tools(), dir).unwrap().permissions();
    assert!(old.delete && !old.open_project);
}

// Checks that the permissions apply to every client, not only one.
#[test]
fn permissions_apply_to_every_client() {
    let (_host, core) = core_with(tools());
    let first = connect_allowed(&core, "Claude Code");
    let second = connect_allowed(&core, "OpenCode");
    core.set_permissions(Permissions { edit: false, ..core.permissions() }).unwrap();
    assert_eq!(code(&core, &first, "change"), "permission_denied");
    assert_eq!(code(&core, &second, "change"), "permission_denied");
}

// Checks that the permissions are saved and read again after a restart, in a
// file only the user can read; and that a damaged file gives the defaults.
#[test]
fn permissions_are_saved_and_a_damaged_file_gives_the_defaults() {
    let (host, core) = core_with(tools());
    let changed = Permissions { delete: true, export: false, ..Permissions::default() };
    core.set_permissions(changed).unwrap();
    drop(core);
    let dir = host.dir.path().join(".sprixel");
    assert_eq!(Core::new(host.clone(), tools(), dir.clone()).unwrap().permissions(), changed);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(dir.join(policy::PERMISSIONS_FILE)).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    std::fs::write(dir.join(policy::PERMISSIONS_FILE), "oops").unwrap();
    assert_eq!(Core::new(host.clone(), tools(), dir).unwrap().permissions(), Permissions::default());
}

// Checks that the UI is told when the permissions change.
#[test]
fn a_change_is_announced_to_the_ui() {
    let (host, core) = core_with(tools());
    core.set_permissions(Permissions { create: false, ..Permissions::default() }).unwrap();
    let events = host.events.lock().unwrap();
    let (name, payload) = events.last().unwrap();
    assert_eq!(name, PERMISSIONS_EVENT);
    assert_eq!(payload["create"], false);
}

// Checks "Ask me when a new agent connects": with it off, a new client is
// allowed at once and the user is not asked; with it on (default) it waits.
#[test]
fn ask_on_connect_off_allows_new_clients() {
    let (host, core) = core_with(tools());
    core.set_permissions(Permissions { ask_on_connect: false, ..Permissions::default() }).unwrap();

    let session = testing::connect(&core, "Claude Code");

    assert_eq!(code(&core, &session, "read"), "");
    assert_eq!(core.clients()[0].state, "allowed");
    assert!(host.events.lock().unwrap().iter().all(|(e, _)| e != CLIENT_PENDING_EVENT));
}
