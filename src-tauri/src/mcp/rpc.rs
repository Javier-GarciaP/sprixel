//! The Model Context Protocol on top of JSON-RPC 2.0, written by hand: it
//! needs only `initialize`, `ping`, `tools/list` and `tools/call` (see
//! `design.md`). This layer knows nothing about HTTP: it takes the id of the
//! session and the body of a request and gives a reply.

use super::sessions::{Session, clean_name, clean_text};
use super::tool::{Output, ToolError, ToolResult};
use super::clients::Standing;
use super::{tools, CLIENTS_EVENT, CLIENT_PENDING_EVENT, Core, base64};
use serde_json::{Value, json};

/// The protocol versions the server speaks. The first is the newest.
const VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

/// What the HTTP layer sends back.
#[derive(Debug, PartialEq)]
pub struct Reply {
    pub status: u16,
    /// The id of a new session (the `Mcp-Session-Id` header).
    pub session: Option<String>,
    /// The JSON body, or `None` for an empty answer (a notification).
    pub body: Option<Value>,
}

impl Reply {
    fn json(body: Value) -> Reply {
        Reply { status: 200, session: None, body: Some(body) }
    }

    fn empty(status: u16) -> Reply {
        Reply { status, session: None, body: None }
    }
}

fn result(id: &Value, result: Value) -> Reply {
    Reply::json(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn error(id: &Value, code: i64, message: &str) -> Reply {
    Reply::json(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }))
}

impl Core {
    /// Answers one JSON-RPC message. `session` is the id the client sent, if any.
    pub fn handle(&self, session: Option<&str>, body: &[u8]) -> Reply {
        let Ok(message) = serde_json::from_slice::<Value>(body) else {
            return error(&Value::Null, -32700, "the body is not valid JSON");
        };
        let (Some(method), true) = (message["method"].as_str(), message.is_object()) else {
            return error(&Value::Null, -32600, "expected one JSON-RPC request (batches are not supported)");
        };
        // A message with no id is a notification: it gets no answer.
        let Some(id) = message.get("id").filter(|id| !id.is_null()) else {
            return Reply::empty(202);
        };
        let params = &message["params"];
        match method {
            "initialize" => self.initialize(id, params),
            "ping" => result(id, json!({})),
            "tools/list" | "tools/call" => {
                let Some(client) = session.and_then(|s| self.sessions.lock().unwrap().touch(s)) else {
                    return Reply {
                        status: 400,
                        session: None,
                        body: Some(json!({ "jsonrpc": "2.0", "id": id, "error": {
                            "code": -32002,
                            "message": "no session: send `initialize` first and use the Mcp-Session-Id header"
                        }})),
                    };
                };
                if method == "tools/list" {
                    result(id, json!({ "tools": self.list_tools(&client) }))
                } else {
                    result(id, self.call_tool(&client, params))
                }
            }
            _ => error(id, -32601, &format!("unknown method {method:?}")),
        }
    }

    fn initialize(&self, id: &Value, params: &Value) -> Reply {
        let asked = params["protocolVersion"].as_str().unwrap_or("");
        let version = VERSIONS.iter().find(|v| **v == asked).unwrap_or(&VERSIONS[0]);
        let text = |value: &Value| value.as_str().unwrap_or("").chars().take(256).collect::<String>();
        let client = Session {
            name: clean_name(&text(&params["clientInfo"]["name"])),
            version: clean_text(&text(&params["clientInfo"]["version"]), 64),
        };
        let (client_name, client_version) = (client.name.clone(), client.version.clone());
        let Some(session) = self.sessions.lock().unwrap().start(client) else {
            return error(id, -32000, "limit_exceeded: too many clients are connected");
        };
        let asked = match self.clients.lock().unwrap().request(&client_name, &client_version, self.permissions().ask_on_connect) {
            Some((_, asked)) => asked,
            None => {
                // Too many clients wait for an answer: do not let a program
                // fill the list of the person with names.
                self.sessions.lock().unwrap().end(&session);
                self.log("a client was refused: too many clients are waiting for approval");
                return error(id, -32000, "limit_exceeded: too many clients are waiting for approval");
            }
        };
        if asked {
            // The user answers in the app; a system notification is sent when
            // the app is in the background.
            self.log("a new client waits for approval");
            self.host.emit(CLIENT_PENDING_EVENT, json!({ "name": client_name, "version": client_version }));
            self.host.emit(CLIENTS_EVENT, Value::Null);
        }
        let mut reply = result(
            id,
            json!({
                "protocolVersion": version,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "sprixel", "version": env!("CARGO_PKG_VERSION") },
                "instructions": super::tools::INSTRUCTIONS,
            }),
        );
        reply.session = Some(session);
        reply
    }

    fn standing(&self, client: &Session) -> Standing {
        self.clients.lock().unwrap().standing(&client.name, &client.version)
    }

    /// The tools a client sees, with their schemas.
    fn list_tools(&self, client: &Session) -> Vec<Value> {
        // A client that is not allowed sees no tool at all.
        if self.standing(client) != Standing::Allowed {
            return Vec::new();
        }
        self.tools
            .iter()
            .map(|tool| {
                json!({ "name": tool.name, "description": tool.description, "inputSchema": tool.schema })
            })
            .collect()
    }

    /// Runs `tools/call`. A failure of the tool is a normal result with
    /// `isError`, so the AI reads the exact error and fixes it.
    fn call_tool(&self, client: &Session, params: &Value) -> Value {
        let outcome = match params["name"].as_str() {
            Some(name) => self.run_tool(client, name, &params["arguments"]),
            None => Err(ToolError::new("bad_arguments", "`name` is the name of the tool to call")),
        };
        tool_result(outcome)
    }

    /// Finds a tool and runs it. The checks before it runs (approval,
    /// permissions) belong here.
    pub fn run_tool(&self, client: &Session, name: &str, arguments: &Value) -> ToolResult {
        // Looked up on every call, so removing or rejecting a client applies
        // to its very next call.
        match self.standing(client) {
            Standing::Allowed => {}
            Standing::Pending => {
                return Err(ToolError::new(
                    "awaiting_approval",
                    "the user has not allowed this client yet; ask them to press Allow in the Sprixel app, then try again",
                ));
            }
            Standing::Rejected => {
                return Err(ToolError::new("client_rejected", "the user rejected this client"));
            }
            Standing::Unknown => {
                return Err(ToolError::new(
                    "not_approved",
                    "this client is not allowed; connect again to ask the user for access",
                ));
            }
        }
        let tool = self.tools.iter().find(|t| t.name == name).ok_or_else(|| {
            let names: Vec<&str> = self.tools.iter().map(|t| t.name).collect();
            ToolError::new("unknown_tool", format!("there is no tool {name:?}; the tools are {}", names.join(", ")))
        })?;
        let ctx = super::tool::Ctx { core: self, client: &client.name };
        let outcome = tool.need_check(&self.permissions()).and_then(|()| {
            // A stopped client cannot write until the person lets it continue.
            if tool.need != super::Need::Nothing && self.host.agents().is_stopped(&client.name) {
                return Err(tools::write::stopped());
            }
            // A bug in a tool must not take the connection down: it becomes an
            // error for the client.
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (tool.run)(&ctx, arguments)))
                .unwrap_or_else(|_| {
                    self.log("a tool failed unexpectedly");
                    Err(ToolError::new("internal_error", "the tool failed unexpectedly; the project was not changed by this call"))
                })
        });
        // Reading tools are not actions: they are not logged.
        if tool.need != super::Need::Nothing {
            self.record(&client.name, tool.name, arguments, &outcome);
        }
        outcome
    }
}

/// A tool result as MCP wants it: a list of content and the `isError` flag.
fn tool_result(outcome: ToolResult) -> Value {
    match outcome {
        Ok(Output::Json(value)) => text_result(&value, false),
        Ok(Output::Image { png, info }) => json!({
            "content": [
                { "type": "image", "data": base64::encode(&png), "mimeType": "image/png" },
                { "type": "text", "text": info.to_string() },
            ],
            "isError": false,
        }),
        Err(error) => text_result(&error.to_json(), true),
    }
}

fn text_result(value: &Value, is_error: bool) -> Value {
    json!({ "content": [{ "type": "text", "text": value.to_string() }], "isError": is_error })
}
