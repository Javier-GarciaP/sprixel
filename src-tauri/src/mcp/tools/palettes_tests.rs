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

// Checks "Clean": a palette with 3 colors where sprites use one keeps that
// one, the other keys are returned in order, a second call removes nothing
// (and adds no history entry), and undo brings the colors back.
#[test]
fn remove_unused_colors_keeps_the_used_ones() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    // The goblin uses R and G; B is added and used by nobody.
    call(&core, &session, "set_palette_color", json!({ "key": "B", "hex": "#0000FF" })).unwrap();
    call(&core, &session, "set_palette_color", json!({ "key": "Y", "hex": "#FFFF00" })).unwrap();
    let before = entries(&host);

    let done = call(&core, &session, "remove_unused_colors", json!({ "palette": "MAIN" })).unwrap();

    assert_eq!(done["removed"], json!(["B", "Y"]));
    let keys = |core: &_, session: &str| {
        let palette = call(core, session, "get_palette", json!({ "palette": "main" })).unwrap();
        palette["colors"].as_array().unwrap().iter().map(|c| c["key"].as_str().unwrap().to_string()).collect::<Vec<_>>()
    };
    assert_eq!(keys(&core, &session), ["R", "G"]);
    assert_eq!(entries(&host), before + 1);

    let again = call(&core, &session, "remove_unused_colors", json!({ "palette": "main" })).unwrap();
    assert_eq!((again["removed"].clone(), again["changed"].clone()), (json!([]), json!(false)));
    assert_eq!(entries(&host), before + 1);
    browser::undo(&host.project).unwrap();
    assert_eq!(keys(&core, &session), ["R", "G", "B", "Y"]);
}

// Checks an unknown palette is `not_found` and the permission is needed.
#[test]
fn remove_unused_colors_errors() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    assert_eq!(call(&core, &session, "remove_unused_colors", json!({ "palette": "ghost" })).unwrap_err()["code"], "not_found");
    assert_eq!(call(&core, &session, "remove_unused_colors", json!({})).unwrap_err()["code"], "bad_argument");
    core.set_permissions(crate::mcp::Permissions { edit: false, ..core.permissions() }).unwrap();
    assert_eq!(call(&core, &session, "remove_unused_colors", json!({ "palette": "main" })).unwrap_err()["code"], "permission_denied");
}

fn tone_hexes(answer: &serde_json::Value) -> Vec<String> {
    answer["tones"].as_array().unwrap().iter().map(|t| t["hex"].as_str().unwrap().to_string()).collect()
}

// Checks the "Three tones" scenario through the tool: dark to light, the
// base in the middle, new keys A, B, C (R and G are taken), names with the
// prefix, one history entry, and undo removes the three colors.
#[test]
fn generate_ramp_adds_three_tones() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = entries(&host);

    let done = call(&core, &session, "generate_ramp", json!({ "palette": "main", "base": "#ff0000", "steps": 3, "name": "fire" })).unwrap();

    assert_eq!(tone_hexes(&done), ["#590000", "#FF0000", "#FFA6A6"]);
    // The base is already `R`: it is reused and not added twice.
    assert_eq!(done["keys"], json!(["A", "R", "B"]));
    assert_eq!(done["tones"][0]["name"], "fire-dark1");
    assert_eq!(done["tones"][2]["name"], "fire-light1");
    assert_eq!(entries(&host), before + 1);
    browser::undo(&host.project).unwrap();
    let palette = call(&core, &session, "get_palette", json!({ "palette": "main" })).unwrap();
    assert_eq!(palette["colors"].as_array().unwrap().len(), 2);
}

// Checks a second call with the same arguments reuses every key and changes
// nothing, and a call with other steps never repeats a color name.
#[test]
fn generate_ramp_reuses_equal_colors_and_keeps_names_unique() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "generate_ramp", json!({ "palette": "main", "base": "#3366CC", "steps": 3 })).unwrap();
    let before = entries(&host);

    let again = call(&core, &session, "generate_ramp", json!({ "palette": "main", "base": "#3366CC", "steps": 3 })).unwrap();
    assert_eq!(again["changed"], false);
    assert_eq!(entries(&host), before);

    let more = call(&core, &session, "generate_ramp", json!({ "palette": "main", "base": "#3366CC", "steps": 5 })).unwrap();
    assert_eq!(more["changed"], true);
    let palette = call(&core, &session, "get_palette", json!({ "palette": "main" })).unwrap();
    let names: Vec<&str> = palette["colors"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap()).collect();
    let mut unique = names.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), names.len(), "{names:?}");
}

// Checks the errors: steps 2 and 10, a bad hue shift and a bad color are
// `bad_argument`; an unknown palette is `not_found`; a palette with 92 colors
// cannot take 3 new tones (`palette_full`). Nothing changes.
#[test]
fn generate_ramp_errors_change_nothing() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let run = |args| call(&core, &session, "generate_ramp", args).unwrap_err();
    let before = revision(&host);

    assert_eq!(run(json!({ "palette": "main", "base": "#FF0000", "steps": 2 }))["code"], "bad_argument");
    assert_eq!(run(json!({ "palette": "main", "base": "#FF0000", "steps": 10 }))["code"], "bad_argument");
    assert_eq!(run(json!({ "palette": "main", "base": "#FF0000", "hue_shift": 90 }))["code"], "bad_argument");
    assert_eq!(run(json!({ "palette": "main", "base": "red" }))["code"], "bad_argument");
    assert_eq!(run(json!({ "palette": "ghost", "base": "#FF0000" }))["code"], "not_found");
    assert_eq!(revision(&host), before);

    for i in 0..90 {
        call(&core, &session, "set_palette_color", json!({ "hex": format!("#{:06X}", i + 100) })).unwrap();
    }
    let before = revision(&host);
    assert_eq!(run(json!({ "palette": "main", "base": "#123456", "steps": 5 }))["code"], "palette_full");
    assert_eq!(revision(&host), before);
}

// Checks `create_palette` with `ramp`: the colors come first, the tones follow
// with free keys, and the answer lists the ramp keys.
#[test]
fn create_palette_can_add_a_ramp() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let made = call(&core, &session, "create_palette", json!({
        "name": "fire", "colors": [{ "key": "A", "hex": "#000000" }],
        "ramp": { "base": "#FF0000", "steps": 3 },
    })).unwrap();

    let keys: Vec<&str> = made["colors"].as_array().unwrap().iter().map(|c| c["key"].as_str().unwrap()).collect();
    assert_eq!(keys, ["A", "B", "C", "D"]);
    assert_eq!(made["ramp_keys"], json!(["B", "C", "D"]));
    assert_eq!(made["colors"][2]["hex"], "#FF0000");
    let error = call(&core, &session, "create_palette", json!({ "name": "x", "ramp": { "base": "#FF0000", "steps": 1 } })).unwrap_err();
    assert_eq!(error["code"], "bad_argument");
}

const SLIME_ROWS: &str = "frame 1\nRRRR....\n........\n........\n........\n........\n........\n........\n........\n";

// Checks that `palette:` with the palette of the sprite works (any letter
// case), and that a grid with no `palette:` line still works.
#[test]
fn a_grid_palette_line_with_the_sprite_palette_is_fine() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let grid = format!("palette: MAIN\n{SLIME_ROWS}");
    call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": grid })).unwrap();
    call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": SLIME_ROWS })).unwrap();
}

// Checks "Other palette": the palette exists but the sprite uses `main`. The
// error names both and says how to fix it; nothing changes. After
// `set_sprite_palette` it works.
#[test]
fn a_grid_palette_line_for_another_palette_is_a_mismatch() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "create_palette", json!({ "name": "skin", "colors": [{ "key": "R", "hex": "#AA0000" }] })).unwrap();
    let before = revision(&host);
    let grid = format!("palette: skin\n{SLIME_ROWS}");

    let error = call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": grid })).unwrap_err();

    assert_eq!(error["code"], "palette_mismatch");
    let message = error["message"].as_str().unwrap();
    assert!(message.contains("main") && message.contains("skin") && message.contains("set_sprite_palette"), "{message}");
    assert_eq!(revision(&host), before);

    // A sprite with no pixels can take the other palette, and then the grid works.
    call(&core, &session, "create_sprite", json!({ "name": "blank", "width": 8, "height": 8, "palette": "skin" })).unwrap();
    call(&core, &session, "apply_grid", json!({ "name": "blank", "grid": grid })).unwrap();
}

// Checks "Unknown palette": a name that does not exist is `palette_not_found`
// and the message lists the palettes that do exist.
#[test]
fn a_grid_palette_line_for_a_missing_palette_is_not_found() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let grid = format!("palette: ghost\n{SLIME_ROWS}");

    let error = call(&core, &session, "apply_grid", json!({ "name": "slime", "grid": grid })).unwrap_err();

    assert_eq!(error["code"], "palette_not_found");
    assert!(error["message"].as_str().unwrap().contains("main"));
    assert_eq!(revision(&host), before);
}
