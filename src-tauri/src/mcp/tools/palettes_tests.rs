//! Tasks 2.1 to 2.4: `create_palette`, `set_sprite_palette`, `own_palette`,
//! `remove_unused_colors` and `generate_ramp`.

use crate::ipc::{browser, project};
use crate::mcp::testing::{call, open_demo_project, revision, setup};
use serde_json::json;

fn entries(host: &crate::mcp::testing::TestHost) -> usize {
    project::read(&host.project, |s| s.history.entries().len()).unwrap()
}

fn palette_of(host: &crate::mcp::testing::TestHost, sprite: &str) -> String {
    project::read(&host.project, |s| {
        s.project.find_sprite(&crate::document::SpriteRef::new(&["Characters"], sprite)).unwrap().palette.clone()
    })
    .unwrap()
}

// Checks "Palette with colors": explicit keys are kept, a color with no key
// gets the next free key (not one written later in the list), the hex is
// stored in capitals, and the call is ONE history entry that undo removes.
#[test]
fn create_palette_adds_colors_with_keys_and_one_history_entry() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = entries(&host);

    let made = call(&core, &session, "create_palette", json!({ "name": "skin", "colors": [
        { "hex": "#ffcc99", "name": "light" }, { "key": "A", "hex": "#aa7744" },
    ]})).unwrap();

    assert_eq!(made["palette"], "skin");
    assert_eq!(made["colors"][0], json!({ "key": "B", "name": "light", "hex": "#FFCC99" }));
    assert_eq!(made["colors"][1], json!({ "key": "A", "name": "AA7744", "hex": "#AA7744" }));
    let listed = call(&core, &session, "get_palette", json!({ "palette": "skin" })).unwrap();
    assert_eq!(listed["colors"].as_array().unwrap().len(), 2);
    assert_eq!(entries(&host), before + 1);
    browser::undo(&host.project).unwrap();
    assert_eq!(call(&core, &session, "get_palette", json!({ "palette": "skin" })).unwrap_err()["code"], "not_found");
}

// Checks "Name taken": any letter case is refused, a bad color is refused,
// and nothing changes (not even the revision).
#[test]
fn create_palette_errors_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let run = |args| call(&core, &session, "create_palette", args).unwrap_err();

    assert_eq!(run(json!({ "name": "MAIN" }))["code"], "name_exists");
    assert_eq!(run(json!({ "name": "x", "colors": [{ "hex": "red" }] }))["code"], "bad_color");
    assert_eq!(run(json!({ "name": "x", "colors": [{ "key": "A", "hex": "#000000" }, { "key": "A", "hex": "#111111" }] }))["code"], "duplicate_key");
    assert_eq!(run(json!({ "name": "x", "colors": [{ "key": "AB", "hex": "#000000" }] }))["code"], "bad_key");
    assert_eq!(run(json!({}))["code"], "bad_argument");
    assert_eq!(revision(&host), before);
}

// Checks "Change palette": the sprite uses the new palette (found ignoring
// case) and undo gives the old one back. `keys_missing` and an unknown palette
// change nothing.
#[test]
fn set_sprite_palette_points_and_undoes() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "create_palette", json!({ "name": "Same", "colors": [
        { "key": "R", "hex": "#FF0000" }, { "key": "G", "hex": "#00FF00" },
    ]})).unwrap();
    call(&core, &session, "create_palette", json!({ "name": "tiny", "colors": [{ "key": "R", "hex": "#FF0000" }] })).unwrap();
    let before = revision(&host);

    let missing = call(&core, &session, "set_sprite_palette", json!({ "name": "goblin", "palette": "tiny" })).unwrap_err();
    assert_eq!(missing["code"], "keys_missing");
    let unknown = call(&core, &session, "set_sprite_palette", json!({ "name": "goblin", "palette": "ghost" })).unwrap_err();
    assert_eq!(unknown["code"], "not_found");
    assert_eq!(revision(&host), before);

    let done = call(&core, &session, "set_sprite_palette", json!({ "name": "goblin", "palette": "same" })).unwrap();
    assert_eq!((done["palette"].as_str(), done["sprite"].as_str()), (Some("Same"), Some("Characters/goblin")));
    assert_eq!(palette_of(&host, "goblin"), "Same");
    browser::undo(&host.project).unwrap();
    assert_eq!(palette_of(&host, "goblin"), "main");
}

// Checks that the new tools need the "Edit pixels" permission.
#[test]
fn palette_tools_need_the_edit_permission() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(crate::mcp::Permissions { edit: false, ..core.permissions() }).unwrap();
    for (tool, args) in [
        ("create_palette", json!({ "name": "x" })),
        ("set_sprite_palette", json!({ "name": "goblin", "palette": "main" })),
    ] {
        assert_eq!(call(&core, &session, tool, args).unwrap_err()["code"], "permission_denied", "{tool}");
    }
}

// Checks "Sprite with own palette": the sprite uses a new empty palette with
// its name, the answer says so, and ONE undo removes the sprite and the palette.
#[test]
fn own_palette_creates_a_palette_named_like_the_sprite() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = entries(&host);

    let made = call(&core, &session, "create_sprite", json!({ "name": "orc", "width": 8, "height": 8, "own_palette": true })).unwrap();

    assert_eq!(made["palette"], "orc");
    let palette = call(&core, &session, "get_palette", json!({ "palette": "orc" })).unwrap();
    assert_eq!(palette["colors"].as_array().unwrap().len(), 0);
    assert_eq!(entries(&host), before + 1);
    browser::undo(&host.project).unwrap();
    assert_eq!(call(&core, &session, "get_palette", json!({ "palette": "orc" })).unwrap_err()["code"], "not_found");
    assert_eq!(call(&core, &session, "list_sprites", json!({})).unwrap()["sprites"].as_array().unwrap().len(), 2);
}

// Checks the conflicts: `own_palette` with `palette` is a bad argument; a
// palette that already has the sprite name is `name_exists`; a sprite name that
// is taken leaves no palette behind. Nothing changes in any case.
#[test]
fn own_palette_conflicts_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let run = |args| call(&core, &session, "create_sprite", args).unwrap_err();

    assert_eq!(run(json!({ "name": "orc", "width": 8, "height": 8, "own_palette": true, "palette": "main" }))["code"], "bad_argument");
    assert_eq!(run(json!({ "name": "main", "width": 8, "height": 8, "own_palette": true }))["code"], "name_exists");
    assert_eq!(run(json!({ "name": "SLIME", "width": 8, "height": 8, "own_palette": true }))["code"], "name_exists");
    assert_eq!(revision(&host), before);
    let project = call(&core, &session, "get_project", json!({})).unwrap();
    assert_eq!(project["palettes"].as_array().unwrap().len(), 1);
}
