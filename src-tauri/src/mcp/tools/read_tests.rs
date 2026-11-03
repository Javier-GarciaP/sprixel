//! Task 3.1: the read tools and `render_preview`. None of them changes the
//! project: the revision and the history stay as they were.

use crate::mcp::testing::{call, open_demo_project, revision, setup};
use serde_json::json;

fn history_len(host: &crate::mcp::testing::TestHost) -> usize {
    crate::ipc::project::read(&host.project, |s| s.history.entries().len()).unwrap()
}

// Checks that without an open project a read tool says so, in a way an AI can
// tell the user.
#[test]
fn no_project_gives_a_clear_error() {
    let (_host, core, session) = setup();
    let error = call(&core, &session, "get_project", json!({})).unwrap_err();
    assert_eq!(error["code"], "no_project");
}

// Checks `get_project`: the name, the revision, the folders, the number of
// sprites and the palettes with their color count.
#[test]
fn get_project_describes_the_project() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let project = call(&core, &session, "get_project", json!({})).unwrap();

    assert_eq!(project["name"], "Kit");
    assert_eq!(project["revision"], revision(&host));
    assert_eq!(project["folders"], json!(["Characters"]));
    assert_eq!(project["sprite_count"], 2);
    assert_eq!(project["palettes"], json!([{ "name": "main", "colors": 2 }]));
}

// Checks `list_sprites`: every sprite with size, frames, palette and
// animations, the folder filter, and an unknown folder.
#[test]
fn list_sprites_lists_and_filters() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let all = call(&core, &session, "list_sprites", json!({})).unwrap();
    let names: Vec<&str> = all["sprites"].as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"goblin") && names.contains(&"slime"));
    let goblin = all["sprites"].as_array().unwrap().iter().find(|s| s["name"] == "goblin").unwrap();
    assert_eq!(goblin["folder"], "Characters");
    assert_eq!((goblin["width"].as_u64(), goblin["height"].as_u64(), goblin["frames"].as_u64()), (Some(4), Some(4), Some(2)));
    assert_eq!(goblin["animations"], json!([{ "name": "walk", "from": 1, "to": 2, "fps": 12, "mode": "loop" }]));

    let only = call(&core, &session, "list_sprites", json!({ "folder": "characters" })).unwrap();
    assert_eq!(only["sprites"].as_array().unwrap().len(), 1);
    assert_eq!(call(&core, &session, "list_sprites", json!({ "folder": "Nope" })).unwrap_err()["code"], "not_found");
}

// Checks the "Read a sprite" scenario: the size, the palette name, the
// frames as a text grid and the animations.
#[test]
fn get_sprite_gives_size_palette_grid_and_animations() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let sprite = call(&core, &session, "get_sprite", json!({ "name": "goblin" })).unwrap();

    assert_eq!((sprite["width"].as_u64(), sprite["height"].as_u64()), (Some(4), Some(4)));
    assert_eq!(sprite["palette"], "main");
    assert_eq!(sprite["animations"][0]["name"], "walk");
    let grid = sprite["grid"].as_str().unwrap();
    assert!(grid.starts_with("palette: main"), "{grid}");
    assert!(grid.contains("frame 1\nR...\n....\n....\n...G\n"), "{grid}");
    assert!(grid.contains("frame 2\n....\n"), "{grid}");
    assert!(grid.contains("R red"), "the legend lists the colors: {grid}");
}

// Checks that one frame can be read alone, a bad frame gives `bad_frame`
// with the valid range, and a path in `name` works like `folder`.
#[test]
fn get_sprite_reads_one_frame_and_accepts_a_path() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let one = call(&core, &session, "get_sprite", json!({ "name": "Characters/goblin", "frame": 2 })).unwrap();
    let grid = one["grid"].as_str().unwrap();
    assert!(grid.contains("frame 2") && !grid.contains("frame 1"));

    let bad = call(&core, &session, "get_sprite", json!({ "name": "goblin", "frame": 3 })).unwrap_err();
    assert_eq!(bad["code"], "bad_frame");
    assert_eq!(bad["frames"], 2);
}

// Checks that `get_sprite` can read a part of a frame: the grid has only the
// rows and columns of the region, it says which region, and the text is a
// valid grid that `apply_grid` takes back with `x` and `y`.
#[test]
fn get_sprite_reads_a_region_as_a_valid_grid() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let part = call(&core, &session, "get_sprite", json!({
        "name": "goblin", "frame": 1, "region": { "x": 2, "y": 2, "width": 2, "height": 2 },
    })).unwrap();

    let grid = part["grid"].as_str().unwrap();
    assert!(grid.contains("frame 1\n..\n.G\n"), "{grid}");
    assert_eq!(part["region"]["width"], 2);
    assert_eq!(part["width"], 4, "the size is still the size of the sprite");
    // The text parses and writes back into the same place.
    let back = call(&core, &session, "apply_grid", json!({ "name": "goblin", "grid": grid, "x": 0, "y": 0 }));
    assert!(back.is_ok(), "{back:?}");
}

// Checks `include_palette` false: no legend, but the palette name stays, in
// the grid and in the answer. The default keeps the legend.
#[test]
fn get_sprite_can_leave_out_the_legend() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let short = call(&core, &session, "get_sprite", json!({ "name": "goblin", "frame": 1, "include_palette": false })).unwrap();

    let grid = short["grid"].as_str().unwrap();
    assert!(!grid.contains("legend") && !grid.contains("red"), "{grid}");
    assert!(grid.starts_with("palette: main\n"), "{grid}");
    assert!(grid.contains("frame 1\nR...\n....\n....\n...G\n"), "{grid}");
    assert_eq!(short["palette"], "main");
    let full = call(&core, &session, "get_sprite", json!({ "name": "goblin", "frame": 1 })).unwrap();
    assert!(full["grid"].as_str().unwrap().contains("legend"));
    assert!(full.get("region").is_none());
}

// Checks that a region that does not fit gives `region_out`, and one that is
// badly written gives `bad_argument`.
#[test]
fn get_sprite_refuses_a_bad_region() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let code = |region| {
        call(&core, &session, "get_sprite", json!({ "name": "goblin", "region": region })).unwrap_err()["code"]
            .as_str().unwrap().to_string()
    };

    assert_eq!(code(json!({ "x": 3, "y": 0, "width": 2, "height": 1 })), "region_out");
    assert_eq!(code(json!({ "x": 0, "y": 0, "width": 0, "height": 1 })), "bad_argument");
    assert_eq!(code(json!({ "x": 0, "y": 0 })), "bad_argument");
    assert_eq!(code(json!("all")), "bad_argument");
}

// Checks the errors of the sprite lookup: an unknown sprite lists the ones
// that exist, and two sprites with one name ask for the folder.
#[test]
fn sprite_lookup_errors_help_the_ai() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let missing = call(&core, &session, "get_sprite", json!({ "name": "gobln" })).unwrap_err();
    assert_eq!(missing["code"], "not_found");
    assert!(missing["message"].as_str().unwrap().contains("goblin"));

    crate::ipc::project::edit(&host.project, |s| {
        s.apply("user", vec![crate::document::Operation::CreateSprite {
            folder: vec![], name: "goblin".into(), width: 2, height: 2, palette: "main".into(),
        }]).unwrap();
    }).unwrap();
    let ambiguous = call(&core, &session, "get_sprite", json!({ "name": "goblin" })).unwrap_err();
    assert_eq!(ambiguous["code"], "ambiguous_name");
    assert_eq!(ambiguous["folders"].as_array().unwrap().len(), 2);
    assert!(call(&core, &session, "get_sprite", json!({ "name": "goblin", "folder": "" })).is_ok());
}

// Checks that arguments of the wrong kind or with a path that tries to leave
// the project are refused with a clear code.
#[test]
fn bad_arguments_are_refused() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let code = |args| call(&core, &session, "get_sprite", args).unwrap_err()["code"].as_str().unwrap().to_string();

    assert_eq!(code(json!({})), "bad_argument");
    assert_eq!(code(json!({ "name": 5 })), "bad_argument");
    assert_eq!(code(json!({ "name": "goblin", "frame": "one" })), "bad_argument");
    assert_eq!(code(json!({ "name": "goblin", "frame": -1 })), "bad_argument");
    assert_eq!(code(json!({ "name": "goblin", "folder": "../x" })), "invalid_path");
    assert_eq!(code(json!({ "name": "goblin", "folder": "/etc" })), "invalid_path");
    assert_eq!(code(json!({ "name": "../goblin" })), "invalid_path");
    assert_eq!(code(json!({ "name": "goblin", "folder": "C:\\x" })), "invalid_path");
}

// Checks `get_palette`: keys, names, hex values, which are used, and the
// three ways to ask (default, by name, by sprite).
#[test]
fn get_palette_lists_the_colors() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    let palette = call(&core, &session, "get_palette", json!({})).unwrap();

    assert_eq!(palette["name"], "main");
    assert_eq!(palette["max_colors"], 93);
    assert_eq!(palette["colors"][0], json!({ "key": "R", "name": "red", "hex": "#FF0000", "used": true }));
    assert_eq!(palette["colors"][1]["key"], "G");
    assert_eq!(call(&core, &session, "get_palette", json!({ "palette": "MAIN" })).unwrap()["name"], "main");
    assert_eq!(call(&core, &session, "get_palette", json!({ "name": "slime" })).unwrap()["name"], "main");
    assert_eq!(call(&core, &session, "get_palette", json!({ "palette": "nope" })).unwrap_err()["code"], "not_found");
}

fn png_size(png: &[u8]) -> (u32, u32) {
    let be = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
    (be(16), be(20))
}

// Checks the "See the work" scenario: `render_preview` returns a PNG image
// of the frame, scaled with sharp pixels, with the right colors.
#[test]
fn render_preview_returns_a_png_of_the_frame() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let reply = crate::mcp::testing::rpc(
        &core,
        Some(&session),
        "tools/call",
        json!({ "name": "render_preview", "arguments": { "name": "goblin", "frame": 1, "scale": 4 } }),
    );

    let content = &reply["result"]["content"];
    assert_eq!(content[0]["type"], "image");
    assert_eq!(content[0]["mimeType"], "image/png");
    let data = content[0]["data"].as_str().unwrap();
    // PNG files start with these bytes; in base64 they start with "iVBOR".
    assert!(data.starts_with("iVBORw0KGgo"), "{data}");
    let info: serde_json::Value = serde_json::from_str(content[1]["text"].as_str().unwrap()).unwrap();
    assert_eq!((info["width"].as_u64(), info["height"].as_u64()), (Some(16), Some(16)));
    assert_eq!(info["frame"], 1);
    let _ = host;
}

// Checks the real image: decodes the PNG bytes and looks at two pixels (the
// red one at the top left and a transparent one).
#[test]
fn render_preview_pixels_are_right() {
    let (host, core, _session) = setup();
    open_demo_project(&host);
    let state = &host.project;
    let png = crate::ipc::project::read(state, |s| {
        let sprite = s.project.find_sprite(&crate::document::SpriteRef::new(&["Characters"], "goblin")).unwrap();
        let palette = s.project.palette("main").unwrap();
        crate::export::encode_png(&crate::render::render_frame(sprite, palette, 0, 2)).unwrap()
    }).unwrap();
    assert_eq!(png_size(&png), (8, 8));
    let image = image::load_from_memory(&png).unwrap().to_rgba8();
    assert_eq!(image.get_pixel(0, 0).0, [255, 0, 0, 255]);
    assert_eq!(image.get_pixel(1, 1).0, [255, 0, 0, 255]);
    assert_eq!(image.get_pixel(4, 0).0[3], 0);
    let _ = core;
}

// Checks the limits of `render_preview`: a bad frame, a scale that is not
// allowed and an image that would be too big (the error says the scale to use).
#[test]
fn render_preview_limits() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let run = |args| call(&core, &session, "render_preview", args);

    assert_eq!(run(json!({ "name": "goblin", "frame": 9 })).unwrap_err()["code"], "bad_frame");
    assert_eq!(run(json!({ "name": "goblin", "scale": 0 })).unwrap_err()["code"], "bad_argument");
    assert_eq!(run(json!({ "name": "goblin", "scale": 65 })).unwrap_err()["code"], "bad_argument");
    crate::ipc::project::edit(&host.project, |s| {
        s.apply("user", vec![crate::document::Operation::CreateSprite {
            folder: vec![], name: "huge".into(), width: 300, height: 300, palette: "main".into(),
        }]).unwrap();
    }).unwrap();
    let big = run(json!({ "name": "huge", "scale": 4 })).unwrap_err();
    assert_eq!(big["code"], "too_big");
    assert_eq!(big["max_scale"], 3);
    assert!(run(json!({ "name": "huge", "scale": 3 })).is_ok());
}

// Checks `open_sprite`: the UI is asked to show the sprite, an unknown sprite
// is an error and the project does not change.
#[test]
fn open_sprite_asks_the_ui_to_show_it() {
    let (host, core, session) = setup();
    open_demo_project(&host);

    call(&core, &session, "open_sprite", json!({ "name": "goblin" })).unwrap();

    let events = host.events.lock().unwrap();
    let (name, payload) = events.last().unwrap();
    assert_eq!(name, super::OPEN_SPRITE_EVENT);
    assert_eq!(payload, &json!({ "folder": ["Characters"], "name": "goblin" }));
    drop(events);
    assert_eq!(call(&core, &session, "open_sprite", json!({ "name": "nope" })).unwrap_err()["code"], "not_found");
}

// Checks the rule of the spec: read tools do not change the project. After
// calling every one of them the revision and the history are the same.
#[test]
fn read_tools_do_not_change_the_revision() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let (before, entries) = (revision(&host), history_len(&host));

    for (tool, args) in [
        ("get_project", json!({})),
        ("list_sprites", json!({})),
        ("get_sprite", json!({ "name": "goblin" })),
        ("get_palette", json!({})),
        ("render_preview", json!({ "name": "goblin" })),
        ("open_sprite", json!({ "name": "goblin" })),
    ] {
        call(&core, &session, tool, args).unwrap_or_else(|e| panic!("{tool}: {e}"));
    }

    assert_eq!(revision(&host), before);
    assert_eq!(history_len(&host), entries);
}
