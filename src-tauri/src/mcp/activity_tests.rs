//! Task 4.1: the activity log, attribution and the "last edited by" data.

use super::*;
use super::testing::{call, connect_allowed, core_with, open_demo_project, setup};
use crate::ipc::{browser, project};
use serde_json::json;

fn rows(core: &Core) -> Vec<ActivityRow> {
    core.activity()
}

// Checks the "An agent creates a sprite" scenario: the log shows the client
// "OpenCode", the action "Created the sprite slime", and the time.
#[test]
fn the_log_records_who_did_what_and_when() {
    let (host, core) = core_with(tools::all());
    open_demo_project(&host);
    let session = connect_allowed(&core, "OpenCode");
    let start = activity::now_ms();

    call(&core, &session, "create_sprite", json!({ "name": "blob", "width": 8, "height": 8 })).unwrap();

    let log = rows(&core);
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].entry.client, "OpenCode");
    assert_eq!(log[0].entry.summary, "Created the sprite blob");
    assert_eq!(log[0].entry.tool, "create_sprite");
    assert_eq!(log[0].entry.result, "ok");
    assert!(log[0].entry.at_ms >= start && log[0].entry.at_ms <= activity::now_ms());
    assert_eq!(log[0].entry.revision, Some(super::testing::revision(&host)));
}

// Checks that every kind of action has a clear description and the sprite it
// was about, and that the list is newest first.
#[test]
fn every_action_has_a_description_and_the_list_is_newest_first() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(Permissions { delete: true, ..core.permissions() }).unwrap();
    for (tool, args) in [
        ("create_folder", json!({ "name": "Enemigos" })),
        ("create_sprite", json!({ "name": "orc", "folder": "Enemigos", "width": 4, "height": 4 })),
        ("add_frame", json!({ "name": "orc" })),
        ("set_pixels", json!({ "name": "orc", "pixels": [{ "x": 0, "y": 0, "key": "R" }, { "x": 1, "y": 0, "key": "R" }] })),
        ("draw_shape", json!({ "name": "orc", "shape": "line", "key": "G", "x0": 0, "y0": 3, "x1": 3, "y1": 3 })),
        ("apply_grid", json!({ "name": "orc", "frame": 1, "grid": "RRRR\n....\n....\n....\n" })),
        ("create_tag", json!({ "sprite": "orc", "name": "idle", "from": 1, "to": 2, "fps": 6 })),
        ("set_palette_color", json!({ "key": "B", "hex": "#0000FF" })),
        ("create_palette", json!({ "name": "skin", "colors": [{ "key": "R", "hex": "#FFCC99" }, { "key": "G", "hex": "#99CC66" }] })),
        ("set_sprite_palette", json!({ "name": "orc", "palette": "skin" })),
        ("generate_ramp", json!({ "palette": "skin", "base": "#FFCC99", "steps": 3 })),
        ("remove_unused_colors", json!({ "palette": "main" })),
        ("delete_frame", json!({ "name": "orc", "frame": 2 })),
        ("export", json!({ "scope": "sprites", "sprites": ["orc"] })),
        ("delete_sprite", json!({ "name": "orc" })),
    ] {
        call(&core, &session, tool, args).unwrap_or_else(|e| panic!("{tool}: {e}"));
    }

    let summaries: Vec<String> = rows(&core).iter().map(|r| r.entry.summary.clone()).collect();
    assert_eq!(
        summaries,
        vec![
            "Deleted the sprite orc",
            "Exported 2 files to assets/sprites",
            "Deleted frame 2 of Enemigos/orc",
            "Removed 1 unused colors from palette main",
            "Made a ramp of 3 tones in palette skin",
            "Pointed Enemigos/orc to the palette skin",
            "Created the palette skin",
            "Added the color B in palette main",
            "Created the animation idle on Enemigos/orc",
            "Applied a grid to Enemigos/orc",
            "Drew a line on Enemigos/orc",
            "Set 2 pixels on Enemigos/orc",
            "Added frame 2 to Enemigos/orc",
            "Created the sprite orc",
            "Created the folder Enemigos",
        ]
    );
    assert_eq!(rows(&core)[7].entry.sprite, None, "a palette color is not about a sprite");
}

// Checks that failed calls are logged with their error code (a denied
// permission, a stale revision, an invalid grid), and that reading tools are
// not logged at all.
#[test]
fn failures_are_logged_and_reads_are_not() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "get_project", json!({})).unwrap();
    call(&core, &session, "get_sprite", json!({ "name": "slime" })).unwrap();
    call(&core, &session, "render_preview", json!({ "name": "slime" })).unwrap();
    assert!(rows(&core).is_empty(), "reads are not actions");

    call(&core, &session, "delete_sprite", json!({ "name": "slime" })).unwrap_err();
    call(&core, &session, "set_pixels", json!({ "name": "slime", "base_revision": 0, "pixels": [] })).unwrap_err();
    call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": "frame 1\nR\n" })).unwrap_err();

    let log = rows(&core);
    let codes: Vec<Option<String>> = log.iter().map(|r| r.entry.error.clone()).collect();
    assert_eq!(codes, vec![Some("row_count".into()), Some("stale_revision".into()), Some("permission_denied".into())]);
    assert!(log.iter().all(|r| r.entry.result == "error" && r.entry.revision.is_none() && !r.can_undo));
    assert_eq!(log[2].entry.summary, "Could not run delete_sprite: permission_denied");
    assert_eq!(log[2].entry.sprite.as_deref(), Some("slime"));
}

// Checks that the UI is told about each new entry.
#[test]
fn a_new_entry_is_announced_to_the_ui() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "create_folder", json!({ "name": "X" })).unwrap();

    let events = host.events.lock().unwrap();
    let (_, entry) = events.iter().rev().find(|(e, _)| e == ACTIVITY_EVENT).unwrap();
    assert_eq!(entry["client"], "Claude Code");
    assert_eq!(entry["summary"], "Created the folder X");
}

// Checks attribution: every operation an agent applies has the name of the
// agent as its author, and the change of the person has the author `user`.
#[test]
fn operations_carry_the_agent_name_as_author() {
    let (host, core) = core_with(tools::all());
    open_demo_project(&host);
    let one = connect_allowed(&core, "Claude Code");
    let two = connect_allowed(&core, "OpenCode");

    call(&core, &one, "create_folder", json!({ "name": "A" })).unwrap();
    call(&core, &two, "create_folder", json!({ "name": "B" })).unwrap();
    browser::create_folder(&host.project, &[], "C").unwrap();

    let authors: Vec<String> = project::read(&host.project, |s| s.history.entries().iter().map(|e| e.author.clone()).collect()).unwrap();
    assert_eq!(authors, vec!["Claude Code", "OpenCode", "user"]);
}

// Checks the "Last edited by" scenario: the data the app shows for a sprite
// names the agent that edited it last, and the person when the person edits
// it afterwards. The read tools show it too.
#[test]
fn a_sprite_shows_who_edited_it_last() {
    let (host, core) = core_with(tools::all());
    open_demo_project(&host);
    let session = connect_allowed(&core, "OpenCode");
    let last = |name: &str| {
        let tree = browser::project_tree(&host.project).unwrap();
        let all = tree.root.sprites.iter().chain(tree.root.folders.iter().flat_map(|f| f.sprites.iter()));
        all.into_iter().find(|s| s.name == name).unwrap().last_editor.as_ref().map(|e| e.author.clone())
    };
    assert_eq!(last("slime"), None);

    call(&core, &session, "set_pixels", json!({ "name": "slime", "pixels": [{ "x": 0, "y": 0, "key": "R" }] })).unwrap();

    assert_eq!(last("slime").as_deref(), Some("OpenCode"));
    assert_eq!(last("goblin"), None);
    let sprite = call(&core, &session, "get_sprite", json!({ "name": "slime" })).unwrap();
    assert_eq!(sprite["last_edited_by"]["author"], "OpenCode");

    project::edit(&host.project, |s| {
        let ops = s.project.stroke_operations(&crate::document::SpriteRef::new(&[], "slime"), 0, 1, &[crate::document::ColorChange { x: 1, y: 1, color: None }]).unwrap();
        s.apply("user", ops).unwrap();
    }).unwrap();
    assert_eq!(last("slime").as_deref(), Some("user"));
}

// Checks that a client cannot pretend to be the person: a client that calls
// itself "user" gets another label, so its changes are never taken for the
// ones of the person (which would change what Undo may revert).
#[test]
fn a_client_cannot_use_the_name_of_the_person() {
    let (host, core) = core_with(tools::all());
    open_demo_project(&host);
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "clientInfo": { "name": "USER", "version": "1.0" } } });
    let session = core.handle(None, body.to_string().as_bytes()).session.unwrap();
    assert_eq!(core.clients()[0].name, "USER (agent)");
    core.answer_client("USER (agent)", "1.0", true).unwrap();

    call(&core, &session, "create_folder", json!({ "name": "A" })).unwrap();

    let author = project::read(&host.project, |s| s.history.entries()[0].author.clone()).unwrap();
    assert_eq!(author, "USER (agent)");
}
