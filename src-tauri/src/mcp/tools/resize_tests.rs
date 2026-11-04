//! Spec `format-and-compat`: the tool `resize_sprite`.

use crate::document::SpriteRef;
use crate::ipc::project;
use crate::mcp::testing::{call, open_demo_project, revision, setup};
use crate::mcp::Permissions;
use serde_json::json;

fn size_of(host: &crate::mcp::testing::TestHost) -> (u32, u32) {
    project::read(&host.project, |s| {
        let sprite = s.project.find_sprite(&SpriteRef::new(&["Characters"], "goblin")).unwrap();
        (sprite.width, sprite.height)
    })
    .unwrap()
}

// Checks the spec scenario "Tool": the goblin (4x4, red pixel at 0,0 and green
// at 3,3) grows to 8x8 with `center`, as ONE history entry, the answer has the
// new size and an image, and the person can undo it.
#[test]
fn resize_sprite_changes_the_canvas_in_one_undoable_step() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);

    let done = call(&core, &session, "resize_sprite", json!({ "name": "goblin", "folder": "Characters", "width": 8, "height": 8, "anchor": "center" })).unwrap();

    assert_eq!((done["width"].as_u64(), done["height"].as_u64()), (Some(8), Some(8)));
    assert_eq!(done["anchor"], "center");
    assert_eq!(done["revision"], before + 1);
    assert_eq!(done["dropped_anchors"], json!([]));
    assert_eq!(size_of(&host), (8, 8));
    let rows = crate::mcp::testing::rows(&host, &["Characters"], "goblin", 1);
    assert_eq!(rows.split('/').nth(2).unwrap(), "..R.....", "the red pixel moved by 2,2");

    call(&core, &session, "undo_last_ai_action", json!({})).unwrap();
    assert_eq!(size_of(&host), (4, 4));
    assert_eq!(crate::mcp::testing::rows(&host, &["Characters"], "goblin", 1), "R.../..../..../...G");
}

// Checks that the default anchor is `center`, and that the answer lists the
// anchors that fell outside as warnings (the demo anchors are set first).
#[test]
fn anchors_that_fall_outside_are_reported() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    for (name, x, y) in [("feet", 1, 3), ("head", 3, 0)] {
        call(&core, &session, "set_anchor", json!({ "sprite": "goblin", "folder": "Characters", "name": name, "x": x, "y": y })).unwrap();
    }

    // 4x4 -> 2x2 with the top-left: both anchors are outside or at the edge.
    let done = call(&core, &session, "resize_sprite", json!({ "name": "goblin", "folder": "Characters", "width": 2, "height": 2, "anchor": "top-left" })).unwrap();

    let dropped: Vec<_> = done["dropped_anchors"].as_array().unwrap().iter().map(|a| a["name"].as_str().unwrap()).collect();
    assert_eq!(dropped, vec!["feet", "head"]);
    assert!(done["warnings"][0].as_str().unwrap().contains("feet"), "{done}");

    let default = call(&core, &session, "resize_sprite", json!({ "name": "goblin", "folder": "Characters", "width": 4, "height": 4 })).unwrap();
    assert_eq!(default["anchor"], "center");
}

// Checks the errors: sizes out of range or equal to the current one, an
// unknown anchor name and an unknown sprite. A failed call changes nothing.
#[test]
fn resize_sprite_checks_its_arguments() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let before = revision(&host);
    let run = |args| call(&core, &session, "resize_sprite", args).unwrap_err();
    let base = |w: i64, h: i64| json!({ "name": "goblin", "folder": "Characters", "width": w, "height": h });

    assert_eq!(run(base(0, 4))["code"], "bad_argument");
    assert_eq!(run(base(4, 513))["code"], "bad_argument");
    assert_eq!(run(base(4, 4))["code"], "bad_argument", "the same size");
    let mut bad_anchor = base(8, 8);
    bad_anchor["anchor"] = json!("middle");
    let error = run(bad_anchor);
    assert_eq!(error["code"], "bad_argument");
    assert_eq!(error["argument"], "anchor");
    assert_eq!(run(json!({ "name": "ghost", "width": 8, "height": 8 }))["code"], "not_found");
    assert_eq!(run(json!({ "name": "goblin", "folder": "Characters", "width": 8 }))["code"], "bad_argument");
    assert_eq!(revision(&host), before);
    assert_eq!(size_of(&host), (4, 4));
}

// Checks that the tool needs the Edit permission, and `base_revision` works like in the other write tools.
#[test]
fn resize_sprite_needs_edit_and_checks_the_revision() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let args = json!({ "name": "goblin", "folder": "Characters", "width": 8, "height": 8, "base_revision": revision(&host) + 5 });
    assert_eq!(call(&core, &session, "resize_sprite", args).unwrap_err()["code"], "stale_revision");

    core.set_permissions(Permissions { edit: false, ..core.permissions() }).unwrap();
    let denied = call(&core, &session, "resize_sprite", json!({ "name": "goblin", "folder": "Characters", "width": 8, "height": 8 })).unwrap_err();
    assert_eq!(denied["code"], "permission_denied");
    assert_eq!(size_of(&host), (4, 4));
}

// Checks that `preview: false` skips the image, and that the log says what happened.
#[test]
fn resize_sprite_has_a_preview_and_an_activity_line() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let tool = core.tools.iter().find(|t| t.name == "resize_sprite").unwrap();
    let ctx = crate::mcp::Ctx { core: &core, client: "Claude Code", tool: tool.name };
    let output = (tool.run)(&ctx, &json!({ "name": "goblin", "folder": "Characters", "width": 6, "height": 6 })).unwrap();
    assert!(matches!(output, crate::mcp::Output::Image { .. }), "the answer carries an image");
    let without = (tool.run)(&ctx, &json!({ "name": "goblin", "folder": "Characters", "width": 5, "height": 5, "preview": false })).unwrap();
    assert!(matches!(without, crate::mcp::Output::Json(_)));

    call(&core, &session, "resize_sprite", json!({ "name": "goblin", "folder": "Characters", "width": 4, "height": 4 })).unwrap();
    let log = core.activity();
    assert_eq!(log[0].entry.summary, "Changed the canvas of Characters/goblin to 4x4");
}
