//! Task 3.2: `create_folder`, `create_sprite` and `add_frame`, with the same
//! rules and error codes as the app.

use crate::ipc::{browser, project};
use crate::mcp::testing::{call, open_demo_project, revision, rows, setup};
use serde_json::json;

fn authors(host: &crate::mcp::testing::TestHost) -> Vec<String> {
    project::read(&host.project, |s| s.history.entries().iter().map(|e| e.author.clone()).collect()).unwrap()
}

// Checks the "Create a sprite" scenario: a 16x16 sprite "orc" in the folder
// "Enemigos" exists after the call, the answer has the new revision, the UI
// is told, and the whole call is ONE history entry by the client.
#[test]
fn create_sprite_makes_the_sprite_in_the_folder() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "create_folder", json!({ "name": "Enemigos" })).unwrap();
    let before = revision(&host);

    let made = call(&core, &session, "create_sprite", json!({ "name": "orc", "folder": "Enemigos", "width": 16, "height": 16 })).unwrap();

    assert_eq!(made["revision"], revision(&host));
    assert!(revision(&host) > before);
    let listed = call(&core, &session, "list_sprites", json!({ "folder": "Enemigos" })).unwrap();
    assert_eq!(listed["sprites"][0]["name"], "orc");
    assert_eq!(listed["sprites"][0]["width"], 16);
    assert_eq!(authors(&host), vec!["Claude Code", "Claude Code"]);
    let events = host.events.lock().unwrap();
    assert!(events.iter().any(|(e, p)| e == crate::ipc::browser::CHANGED_EVENT && p["revision"] == made["revision"]));
}

// Checks that a project with no palette gets `main` in the same history
// entry (one undo removes both), like the app does.
#[test]
fn create_sprite_without_a_palette_creates_main_in_one_step() {
    let (host, core, session) = setup();
    project::create_project(&host.project, host.dir.path(), "Empty").unwrap();

    let made = call(&core, &session, "create_sprite", json!({ "name": "orc", "width": 8, "height": 8 })).unwrap();

    assert_eq!(made["palette"], "main");
    assert_eq!(project::read(&host.project, |s| s.history.entries().len()).unwrap(), 1);
    browser::undo(&host.project).unwrap();
    assert!(project::read(&host.project, |s| s.project.palettes.is_empty() && s.project.root.sprites.is_empty()).unwrap());
}

// Checks the shared error codes: the same wrong input gives the same code in
// the tool and in the app, and nothing changes.
#[test]
fn create_sprite_errors_match_the_app() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let run = |args| call(&core, &session, "create_sprite", args).unwrap_err();

    assert_eq!(run(json!({ "name": "big", "width": 513, "height": 8 }))["code"], "too_big");
    assert_eq!(run(json!({ "name": "flat", "width": 0, "height": 8 }))["code"], "bad_size");
    assert_eq!(run(json!({ "name": "GOBLIN", "folder": "characters", "width": 8, "height": 8 }))["code"], "name_exists");
    assert_eq!(run(json!({ "name": "Characters", "width": 8, "height": 8 }))["code"], "name_exists");
    assert_eq!(run(json!({ "name": "a/b", "width": 8, "height": 8 }))["code"], "invalid_name");
    assert_eq!(run(json!({ "name": "x", "folder": "Nope", "width": 8, "height": 8 }))["code"], "not_found");
    assert_eq!(run(json!({ "name": "x", "width": 8, "height": 8, "palette": "ghost" }))["code"], "palette_not_found");
    assert_eq!(run(json!({ "name": "x", "width": "8", "height": 8 }))["code"], "bad_argument");
    assert_eq!(run(json!({ "name": "x", "folder": "../..", "width": 8, "height": 8 }))["code"], "invalid_path");
    assert_eq!(revision(&host), before, "nothing changed");

    // The app gives the same codes for the same input.
    let app = |name: &str, w, h| browser::create_sprite(&host.project, &[], name, w, h, None).unwrap_err().code;
    assert_eq!(app("big", 513, 8), "too_big");
    assert_eq!(app("flat", 0, 8), "bad_size");
    assert_eq!(app("slime", 8, 8), "name_exists");
}

// Checks `create_folder`: a folder in the root and inside a folder, the path
// in the answer, and the errors for a repeated name and a missing parent.
#[test]
fn create_folder_creates_nested_folders_and_checks_names() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let made = call(&core, &session, "create_folder", json!({ "name": "Humans", "parent": "Characters" })).unwrap();

    assert_eq!(made["path"], "Characters/Humans");
    assert!(call(&core, &session, "get_project", json!({})).unwrap()["folders"].as_array().unwrap().contains(&json!("Characters/Humans")));
    assert_eq!(call(&core, &session, "create_folder", json!({ "name": "humans", "parent": "Characters" })).unwrap_err()["code"], "name_exists");
    assert_eq!(call(&core, &session, "create_folder", json!({ "name": "X", "parent": "Nope" })).unwrap_err()["code"], "not_found");
    assert_eq!(call(&core, &session, "create_folder", json!({ "name": "..", })).unwrap_err()["code"], "invalid_name");
    assert_eq!(call(&core, &session, "create_folder", json!({})).unwrap_err()["code"], "bad_argument");
}

// Checks `add_frame`: an empty frame at the end, a copy of a frame (with its
// pixels) and a frame in the middle.
#[test]
fn add_frame_adds_an_empty_frame_a_copy_or_one_in_the_middle() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let empty = call(&core, &session, "add_frame", json!({ "name": "goblin" })).unwrap();
    assert_eq!((empty["frame"].as_u64(), empty["frames"].as_u64()), (Some(3), Some(3)));
    assert_eq!(rows(&host, &["Characters"], "goblin", 3), "..../..../..../....");

    let copy = call(&core, &session, "add_frame", json!({ "name": "goblin", "copy_of": 1 })).unwrap();
    assert_eq!(copy["frame"], 4);
    assert_eq!(rows(&host, &["Characters"], "goblin", 4), "R.../..../..../...G");

    let middle = call(&core, &session, "add_frame", json!({ "name": "goblin", "at": 1, "copy_of": 1 })).unwrap();
    assert_eq!(middle["frame"], 1);
    assert_eq!(rows(&host, &["Characters"], "goblin", 1), "R.../..../..../...G");
    assert_eq!(rows(&host, &["Characters"], "goblin", 2), "R.../..../..../...G");
}

// Checks the errors of `add_frame`: a frame to copy that does not exist, a
// place that does not exist and an unknown sprite. Nothing changes.
#[test]
fn add_frame_errors_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);

    assert_eq!(call(&core, &session, "add_frame", json!({ "name": "goblin", "copy_of": 9 })).unwrap_err()["code"], "bad_frame");
    assert_eq!(call(&core, &session, "add_frame", json!({ "name": "goblin", "at": 9 })).unwrap_err()["code"], "bad_frame");
    assert_eq!(call(&core, &session, "add_frame", json!({ "name": "goblin", "at": 0 })).unwrap_err()["code"], "bad_frame");
    assert_eq!(call(&core, &session, "add_frame", json!({ "name": "ghost" })).unwrap_err()["code"], "not_found");
    assert_eq!(revision(&host), before);
}

// Checks that the busy marker is gone when the call ends (the frame is free
// for the person again) and that the UI got a begin and an end.
#[test]
fn the_busy_marker_ends_with_the_call() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "add_frame", json!({ "name": "goblin" })).unwrap();

    assert_eq!(host.agents.writing(), None);
    let events = host.events.lock().unwrap();
    let agent: Vec<_> = events.iter().filter(|(e, _)| e == crate::ipc::editor::AGENT_EVENT).collect();
    assert_eq!(agent.len(), 2);
    assert_eq!(agent[0].1["agent"], "Claude Code");
    assert_eq!(agent[0].1["frame"], 3);
    assert!(agent[1].1.is_null());
}

// Checks that a create tool needs the "Create sprites" permission.
#[test]
fn create_tools_need_the_create_permission() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(crate::mcp::Permissions { create: false, ..core.permissions() }).unwrap();
    let before = revision(&host);

    for (tool, args) in [
        ("create_folder", json!({ "name": "X" })),
        ("create_sprite", json!({ "name": "x", "width": 4, "height": 4 })),
        ("add_frame", json!({ "name": "goblin" })),
    ] {
        assert_eq!(call(&core, &session, tool, args).unwrap_err()["code"], "permission_denied", "{tool}");
    }
    assert_eq!(revision(&host), before);
}
