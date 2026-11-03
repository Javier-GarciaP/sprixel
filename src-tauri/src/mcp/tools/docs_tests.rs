//! Task 3.7: what a client sees when it lists the tools: names, descriptions
//! and schemas. They are the instructions of the AI, so they must explain the
//! text grid format and the palette keys.

use crate::mcp::testing::{connect_allowed, core_with, rpc};
use crate::mcp::tools;
use serde_json::Value;

const WRITE_TOOLS: [&str; 11] = [
    "create_folder", "create_sprite", "add_frame", "apply_grid", "set_pixels", "draw_shape",
    "delete_frame", "create_tag", "set_palette_color", "delete_sprite", "undo_last_ai_action",
];

fn listed() -> Vec<Value> {
    let (_host, core) = core_with(tools::all());
    let session = connect_allowed(&core, "Claude Code");
    rpc(&core, Some(&session), "tools/list", serde_json::json!({}))["result"]["tools"].as_array().unwrap().clone()
}

fn find<'a>(tools: &'a [Value], name: &str) -> &'a Value {
    tools.iter().find(|t| t["name"] == name).unwrap_or_else(|| panic!("no tool {name}"))
}

// Checks that a client sees every tool of the docs, each with a real
// description and an input schema that is an object.
#[test]
fn a_client_sees_every_tool_with_a_description_and_a_schema() {
    let tools = listed();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for expected in [
        "get_project", "list_sprites", "get_sprite", "get_palette", "render_preview", "open_sprite",
        "create_folder", "create_sprite", "apply_grid", "set_pixels", "draw_shape", "add_frame",
        "delete_frame", "create_tag", "set_palette_color", "delete_sprite", "export", "undo_last_ai_action",
        "list_projects", "open_project",
    ] {
        assert!(names.contains(&expected), "missing {expected}: {names:?}");
    }
    let mut unique = names.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), names.len(), "tool names are unique");
    for tool in &tools {
        let name = tool["name"].as_str().unwrap();
        assert!(tool["description"].as_str().unwrap().len() >= 40, "{name} needs a real description");
        assert_eq!(tool["inputSchema"]["type"], "object", "{name}");
        // Every required argument is a property of the schema.
        for required in tool["inputSchema"]["required"].as_array().into_iter().flatten() {
            assert!(tool["inputSchema"]["properties"][required.as_str().unwrap()].is_object(), "{name}.{required}");
        }
        // Every property is explained (a type, and a description or a name that says it).
        for (property, schema) in tool["inputSchema"]["properties"].as_object().unwrap() {
            assert!(schema["type"].is_string(), "{name}.{property} has a type");
        }
    }
}

// Checks that `apply_grid` explains the text grid: the keywords, the legend
// that adds a color, the frame numbers, the region, how `.` works and the
// errors, so an AI can write a grid without other help.
#[test]
fn apply_grid_explains_the_grid_format() {
    let tools = listed();
    let text = find(&tools, "apply_grid")["description"].as_str().unwrap();
    for part in ["palette key", "legend", "#", "frame 1", "transparent", "same length", "`x` and `y`", "skip_transparent", "row_length", "unknown_key", "revision"] {
        assert!(text.contains(part), "the description of apply_grid does not say {part:?}");
    }
}

// Checks that every drawing tool tells the AI to use palette keys.
#[test]
fn drawing_tools_explain_palette_keys() {
    let tools = listed();
    for name in ["set_pixels", "draw_shape", "apply_grid", "get_palette"] {
        let text = find(&tools, name)["description"].as_str().unwrap();
        assert!(text.contains("key"), "{name} should explain keys");
    }
    assert!(find(&tools, "set_pixels")["description"].as_str().unwrap().contains("get_palette"));
}

// Checks that every write tool says what a revision is and accepts
// `base_revision` in its schema.
#[test]
fn write_tools_describe_revisions() {
    let tools = listed();
    for name in WRITE_TOOLS {
        let tool = find(&tools, name);
        assert!(tool["description"].as_str().unwrap().contains("base_revision"), "{name}");
        assert_eq!(tool["inputSchema"]["properties"]["base_revision"]["type"], "integer", "{name}");
    }
    for name in ["get_project", "get_sprite", "render_preview"] {
        assert!(find(&tools, name)["inputSchema"]["properties"]["base_revision"].is_null(), "{name} is a read tool");
    }
}

// Checks that `initialize` gives the short guide (workflow, coordinates,
// revisions and errors) that a client can show to its model.
#[test]
fn initialize_gives_instructions() {
    let (_host, core) = core_with(vec![]);
    let body = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} });

    let reply = core.handle(None, body.to_string().as_bytes()).body.unwrap();

    let text = reply["result"]["instructions"].as_str().unwrap();
    for part in ["get_palette", "render_preview", "from 0", "base_revision", "permission_denied", "stopped_by_user"] {
        assert!(text.contains(part), "the instructions do not say {part:?}");
    }
    assert!(text.len() < 2000, "short enough to put in every prompt");
}
