//! Task 1.1: a client can talk MCP and list a tool.

use super::testing::{connect, core_with, rpc};
use super::*;
use serde_json::{Value, json};

fn echo(_ctx: &Ctx, args: &Value) -> ToolResult {
    Ok(Output::Json(json!({ "echo": args["text"] })))
}

fn example_tool() -> Tool {
    Tool {
        name: "echo",
        description: "Gives back the text it receives.",
        schema: json!({ "type": "object", "properties": { "text": { "type": "string" } } }),
        need: Need::Nothing,
        run: echo,
    }
}

// Checks the "List tools" scenario: after `initialize`, `tools/list` gives
// every tool with its name, description and input schema.
#[test]
fn a_client_lists_the_tools_with_their_schemas() {
    let (_host, core) = core_with(vec![example_tool()]);
    let session = connect(&core, "Claude Code");

    let reply = rpc(&core, Some(&session), "tools/list", json!({}));

    let tool = &reply["result"]["tools"][0];
    assert_eq!(tool["name"], "echo");
    assert_eq!(tool["description"], "Gives back the text it receives.");
    assert_eq!(tool["inputSchema"]["properties"]["text"]["type"], "string");
}

// Checks that `initialize` answers with a protocol version, the tools
// capability and the server name, and gives a session id.
#[test]
fn initialize_answers_with_the_capabilities() {
    let (_host, core) = core_with(vec![]);
    let body = json!({ "jsonrpc": "2.0", "id": 7, "method": "initialize", "params": {
        "protocolVersion": "2024-11-05", "clientInfo": { "name": "OpenCode", "version": "2" } } });

    let reply = core.handle(None, body.to_string().as_bytes());

    assert!(reply.session.is_some());
    let result = &reply.body.unwrap()["result"];
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert!(result["capabilities"]["tools"].is_object());
    assert_eq!(result["serverInfo"]["name"], "sprixel");
}

// Checks that a tool call returns its result as MCP text content, and that a
// tool that fails gives `isError` with the exact error.
#[test]
fn a_tool_call_returns_content_and_errors_are_marked() {
    fn fail(_ctx: &Ctx, _args: &Value) -> ToolResult {
        Err(ToolError::new("row_length", "row 7 has 7 columns, expected 8").with("row", 7))
    }
    let failing = Tool { name: "fail", run: fail, ..example_tool() };
    let (_host, core) = core_with(vec![example_tool(), failing]);
    let session = connect(&core, "Claude Code");

    let ok = rpc(&core, Some(&session), "tools/call", json!({ "name": "echo", "arguments": { "text": "hi" } }));
    assert_eq!(ok["result"]["isError"], false);
    let text = ok["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(serde_json::from_str::<Value>(text).unwrap(), json!({ "echo": "hi" }));

    let bad = rpc(&core, Some(&session), "tools/call", json!({ "name": "fail", "arguments": {} }));
    assert_eq!(bad["result"]["isError"], true);
    let text = bad["result"]["content"][0]["text"].as_str().unwrap();
    let error: Value = serde_json::from_str(text).unwrap();
    assert_eq!(error["code"], "row_length");
    assert_eq!(error["row"], 7);
}

// Checks that a tool that does not exist gives `unknown_tool` and names the
// valid tools, so the AI can correct itself.
#[test]
fn an_unknown_tool_lists_the_valid_ones() {
    let (_host, core) = core_with(vec![example_tool()]);
    let session = connect(&core, "Claude Code");

    let reply = rpc(&core, Some(&session), "tools/call", json!({ "name": "nope" }));

    let text = reply["result"]["content"][0]["text"].as_str().unwrap();
    let error: Value = serde_json::from_str(text).unwrap();
    assert_eq!(error["code"], "unknown_tool");
    assert!(error["message"].as_str().unwrap().contains("echo"));
}

// Checks that tools cannot be listed or called without a session, and that
// an id that was never given does not work either.
#[test]
fn tools_need_a_session() {
    let (_host, core) = core_with(vec![example_tool()]);
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }).to_string();

    assert_eq!(core.handle(None, body.as_bytes()).status, 400);
    assert_eq!(core.handle(Some("made-up"), body.as_bytes()).status, 400);
}

// Checks the protocol edge cases: a notification gets no answer, `ping`
// works, and bad JSON, a batch and an unknown method give JSON-RPC errors.
#[test]
fn protocol_edge_cases() {
    let (_host, core) = core_with(vec![]);
    let notification = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }).to_string();
    let reply = core.handle(None, notification.as_bytes());
    assert_eq!((reply.status, reply.body), (202, None));

    assert_eq!(rpc(&core, None, "ping", json!({}))["result"], json!({}));
    assert_eq!(rpc(&core, None, "nope", json!({}))["error"]["code"], -32601);
    assert_eq!(core.handle(None, b"{oops").body.unwrap()["error"]["code"], -32700);
    assert_eq!(core.handle(None, b"[]").body.unwrap()["error"]["code"], -32600);
}

// Checks that the limit of connected clients answers `limit_exceeded`.
#[test]
fn too_many_clients_give_limit_exceeded() {
    let (_host, core) = core_with(vec![]);
    for _ in 0..MAX_SESSIONS {
        connect(&core, "a");
    }
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} });

    let reply = core.handle(None, body.to_string().as_bytes());

    let message = reply.body.unwrap()["error"]["message"].as_str().unwrap().to_string();
    assert!(message.starts_with("limit_exceeded"), "{message}");
}
