//! Task 3.4: `delete_frame`, `create_tag`, `set_palette_color` and
//! `delete_sprite`, with validation that matches the app.

use crate::ipc::{browser, frames, palettes, project};
use crate::mcp::Permissions;
use crate::mcp::testing::{call, open_demo_project, revision, rows, setup};
use serde_json::json;

fn goblin() -> frames::SpriteArg {
    frames::SpriteArg { folder: vec!["Characters".into()], name: "goblin".into() }
}

fn entries(host: &crate::mcp::testing::TestHost) -> usize {
    project::read(&host.project, |s| s.history.entries().len()).unwrap()
}

// Checks `delete_frame`: the frame goes away, the pixels of the others stay,
// the animation follows, and it is one history entry.
#[test]
fn delete_frame_removes_the_frame() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "set_pixels", json!({ "name": "goblin", "frame": 2, "pixels": [{ "x": 1, "y": 1, "key": "G" }] })).unwrap();
    let before = entries(&host);

    let done = call(&core, &session, "delete_frame", json!({ "name": "goblin", "frame": 1 })).unwrap();

    assert_eq!(done["frames"], 1);
    assert_eq!(done["revision"], revision(&host));
    assert_eq!(rows(&host, &["Characters"], "goblin", 1), "..../.G../..../....");
    assert_eq!(entries(&host), before + 1);
    let sprite = call(&core, &session, "get_sprite", json!({ "name": "goblin" })).unwrap();
    assert_eq!(sprite["frames"], 1);
    assert_eq!(sprite["animations"][0]["to"], 1, "the animation follows the frames");
}

// Checks that the errors of `delete_frame` are the ones of the app: the last
// frame cannot be deleted and a frame that does not exist is `bad_frame`.
#[test]
fn delete_frame_errors_match_the_app() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let slime = json!({ "name": "slime", "frame": 1 });
    let before = revision(&host);

    let tool = call(&core, &session, "delete_frame", slime).unwrap_err();
    let app = frames::delete_frame(&host.project, &frames::SpriteArg { folder: vec![], name: "slime".into() }, 1).unwrap_err();
    assert_eq!(tool["code"], "last_frame");
    assert_eq!(tool["code"], app.code);

    let tool = call(&core, &session, "delete_frame", json!({ "name": "goblin", "frame": 9 })).unwrap_err();
    let app = frames::delete_frame(&host.project, &goblin(), 9).unwrap_err();
    assert_eq!(tool["code"], "bad_frame");
    assert_eq!(tool["code"], app.code);
    assert_eq!(revision(&host), before, "the app call failed too, so nothing changed");
}

// Checks the "Create an animation" scenario: "walk" with frames 1 to 4 at
// 12 fps exists and shows in the details of the sprite.
#[test]
fn create_tag_creates_an_animation() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "add_frame", json!({ "name": "goblin" })).unwrap();
    call(&core, &session, "add_frame", json!({ "name": "goblin" })).unwrap();

    let done = call(&core, &session, "create_tag", json!({ "sprite": "goblin", "name": "run", "from": 3, "to": 4, "fps": 12 })).unwrap();

    assert_eq!(done["mode"], "loop");
    let sprite = call(&core, &session, "get_sprite", json!({ "name": "goblin" })).unwrap();
    assert!(sprite["animations"].as_array().unwrap().contains(&json!({ "name": "run", "from": 3, "to": 4, "fps": 12, "mode": "loop" })));
    call(&core, &session, "add_frame", json!({ "name": "goblin" })).unwrap();
    call(&core, &session, "create_tag", json!({ "sprite": "goblin", "name": "bounce", "from": 5, "to": 5, "fps": 6, "mode": "pingpong" })).unwrap();
}

// Checks that the errors of `create_tag` are the ones of the app for the same
// input: a range outside the frames, a speed out of range, a repeated name
// and a mode that does not exist.
#[test]
fn create_tag_errors_match_the_app() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let tool = |from, to, fps, name: &str| {
        call(&core, &session, "create_tag", json!({ "sprite": "goblin", "name": name, "from": from, "to": to, "fps": fps })).unwrap_err()
    };
    let app = |from, to, fps, name: &str| {
        let tag = frames::TagArg { name: name.into(), from, to, fps, mode: "loop".into() };
        frames::add_tag(&host.project, &goblin(), &tag).unwrap_err().code
    };

    for (from, to, fps, name, code) in [
        (1, 9, 12, "x", "bad_range"),
        (2, 1, 12, "x", "bad_range"),
        (1, 2, 0, "x", "bad_fps"),
        (1, 2, 61, "x", "bad_fps"),
        (1, 2, 12, "WALK", "tag_exists"),
    ] {
        let error = tool(from, to, fps, name);
        assert_eq!(error["code"], code, "{name} {from}-{to}");
        assert_eq!(error["code"], app(from, to, fps, name), "same code as the app");
    }
    let mode = call(&core, &session, "create_tag", json!({ "sprite": "goblin", "name": "x", "from": 1, "to": 2, "fps": 6, "mode": "zigzag" })).unwrap_err();
    assert_eq!(mode["code"], "bad_mode");
    assert_eq!(call(&core, &session, "create_tag", json!({ "name": "x", "from": 1, "to": 2, "fps": 6 })).unwrap_err()["code"], "bad_argument");
    assert_eq!(revision(&host), before);
}

// Checks the "Overlap refused" scenario for the tool: a second animation that
// shares a frame is refused with `animation_overlap` and nothing changes.
#[test]
fn create_tag_refuses_an_overlap() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    call(&core, &session, "add_frame", json!({ "name": "goblin" })).unwrap();
    call(&core, &session, "add_frame", json!({ "name": "goblin" })).unwrap();
    call(&core, &session, "create_tag", json!({ "sprite": "goblin", "name": "run", "from": 3, "to": 3, "fps": 12 })).unwrap();
    let before = revision(&host);

    let error = call(&core, &session, "create_tag", json!({ "sprite": "goblin", "name": "jump", "from": 3, "to": 4, "fps": 12 })).unwrap_err();

    assert_eq!(error["code"], "animation_overlap");
    assert_eq!(revision(&host), before);
    call(&core, &session, "create_tag", json!({ "sprite": "goblin", "name": "jump", "from": 4, "to": 4, "fps": 12 })).unwrap();
}

// Checks `set_palette_color`: a color is added with a key, added with the
// next free key, and changed (the pixels that use it change color without
// changing key).
#[test]
fn set_palette_color_adds_and_changes_colors() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let added = call(&core, &session, "set_palette_color", json!({ "key": "B", "name": "blue", "hex": "#2f6bff" })).unwrap();
    assert_eq!((added["action"].as_str(), added["hex"].as_str()), (Some("added"), Some("#2F6BFF")));
    let next = call(&core, &session, "set_palette_color", json!({ "hex": "#112233" })).unwrap();
    assert_eq!((next["key"].as_str(), next["name"].as_str()), (Some("A"), Some("112233")));

    let changed = call(&core, &session, "set_palette_color", json!({ "key": "R", "hex": "#AA0000", "name": "dark red" })).unwrap();
    assert_eq!(changed["action"], "changed");
    let palette = call(&core, &session, "get_palette", json!({})).unwrap();
    assert_eq!(palette["colors"][0], json!({ "key": "R", "name": "dark red", "hex": "#AA0000", "used": true }));
    assert_eq!(rows(&host, &["Characters"], "goblin", 1), "R.../..../..../...G", "the pixel keeps its key");
    assert_eq!(entries(&host), 3, "one history entry for each call");
}

// Checks that the errors of `set_palette_color` are the ones of the app: a
// repeated name, a color that is not #RRGGBB, a reserved key, and an unknown
// palette. Nothing changes.
#[test]
fn set_palette_color_errors_match_the_app() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let run = |args| call(&core, &session, "set_palette_color", args).unwrap_err();

    assert_eq!(run(json!({ "key": "B", "name": "RED", "hex": "#2f6bff" }))["code"], "duplicate_name");
    assert_eq!(run(json!({ "key": "B", "name": "blue", "hex": "blue" }))["code"], "bad_color");
    assert_eq!(run(json!({ "key": ".", "name": "blue", "hex": "#2f6bff" }))["code"], "reserved_key");
    assert_eq!(run(json!({ "key": "BB", "hex": "#2f6bff" }))["code"], "bad_key");
    assert_eq!(run(json!({ "palette": "ghost", "hex": "#2f6bff" }))["code"], "not_found");
    assert_eq!(run(json!({ "key": "B" }))["code"], "bad_argument");
    assert_eq!(revision(&host), before);

    // The palette editor of the app gives the same codes.
    let app = |key: &str, name: &str, hex: &str| {
        let mut colors: Vec<palettes::ColorArg> = vec![];
        let existing = palettes::palette_info(&host.project, "main").unwrap();
        for c in existing.colors {
            colors.push(palettes::ColorArg { from: Some(c.key.clone()), key: c.key, name: c.name, hex: c.hex });
        }
        colors.push(palettes::ColorArg { from: None, key: key.into(), name: name.into(), hex: hex.into() });
        palettes::apply_edit(&host.project, &host.agents, "main", &colors, &[]).unwrap_err().code
    };
    assert_eq!(app("B", "RED", "#2f6bff"), "duplicate_name");
    assert_eq!(app("B", "blue", "blue"), "bad_color");
    assert_eq!(app(".", "blue", "#2f6bff"), "reserved_key");
}

// Checks the limit of 93 colors: with a full palette a new color without a
// key is `palette_full`.
#[test]
fn set_palette_color_refuses_color_94() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    for i in 0..91 {
        call(&core, &session, "set_palette_color", json!({ "hex": format!("#{:06X}", i + 100) })).unwrap();
    }
    assert_eq!(call(&core, &session, "get_palette", json!({})).unwrap()["colors"].as_array().unwrap().len(), 93);

    let error = call(&core, &session, "set_palette_color", json!({ "hex": "#FFFFFF" })).unwrap_err();

    assert_eq!(error["code"], "palette_full");
}

// Checks `delete_sprite` and the permission: it is off by default, so the sprite
// stays; with the permission on the sprite goes to the trash, the call is one
// history entry, and the person can undo it.
#[test]
fn delete_sprite_needs_its_permission_and_can_be_undone() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let denied = call(&core, &session, "delete_sprite", json!({ "name": "slime" })).unwrap_err();
    assert_eq!(denied["code"], "permission_denied");
    assert_eq!(denied["permission"], "delete");
    assert_eq!(call(&core, &session, "get_project", json!({})).unwrap()["sprite_count"], 2);

    core.set_permissions(Permissions { delete: true, ..core.permissions() }).unwrap();
    let done = call(&core, &session, "delete_sprite", json!({ "name": "slime" })).unwrap();

    assert_eq!(done["deleted"], "slime");
    assert_eq!(done["revision"], revision(&host));
    assert_eq!(call(&core, &session, "get_project", json!({})).unwrap()["sprite_count"], 1);
    browser::undo(&host.project).unwrap();
    assert_eq!(call(&core, &session, "get_project", json!({})).unwrap()["sprite_count"], 2);
}

// Checks that `delete_sprite` fails like the app for a sprite that does not
// exist, and does not delete a sprite found by a similar name.
#[test]
fn delete_sprite_errors_match_the_app() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(Permissions { delete: true, ..core.permissions() }).unwrap();
    let before = revision(&host);

    let tool = call(&core, &session, "delete_sprite", json!({ "name": "ghost" })).unwrap_err();
    let app = browser::delete_sprites(&host.project, &[browser::SpriteTarget { folder: vec![], name: "ghost".into() }]).unwrap_err();

    assert_eq!(tool["code"], "not_found");
    assert_eq!(tool["code"], app.code);
    assert_eq!(call(&core, &session, "delete_sprite", json!({ "name": "gobli" })).unwrap_err()["code"], "not_found");
    assert_eq!(revision(&host), before);
}

// Checks that the structure tools need the "Edit pixels" permission
// (`delete_sprite` has its own permission, tested above).
#[test]
fn structure_tools_need_the_edit_permission() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(Permissions { edit: false, ..core.permissions() }).unwrap();

    for (tool, args) in [
        ("delete_frame", json!({ "name": "goblin", "frame": 1 })),
        ("create_tag", json!({ "sprite": "goblin", "name": "x", "from": 1, "to": 2, "fps": 6 })),
        ("set_palette_color", json!({ "hex": "#112233" })),
    ] {
        assert_eq!(call(&core, &session, tool, args).unwrap_err()["code"], "permission_denied", "{tool}");
    }
}
