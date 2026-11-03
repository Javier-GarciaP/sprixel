//! Helpers for the tests of the server: a host with plain values, and a way
//! to talk JSON-RPC to a core.

use super::{Core, Host, Tool};
use crate::ipc::editor::EditorState;
use crate::ipc::project::ProjectState;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

/// A host with no window. It records what the app would send to the UI.
pub struct TestHost {
    pub project: ProjectState,
    pub agents: EditorState,
    pub events: Mutex<Vec<(String, Value)>>,
    /// The data folder of the server (token, runtime.json...).
    pub dir: tempfile::TempDir,
}

impl Host for TestHost {
    fn project(&self) -> &ProjectState {
        &self.project
    }

    fn agents(&self) -> &EditorState {
        &self.agents
    }

    fn emit(&self, event: &str, payload: Value) {
        self.events.lock().unwrap().push((event.to_string(), payload));
    }
}

impl Default for TestHost {
    fn default() -> Self {
        TestHost {
            project: ProjectState::default(),
            agents: EditorState::default(),
            events: Mutex::default(),
            dir: tempfile::tempdir().unwrap(),
        }
    }
}

pub fn core_with(tools: Vec<Tool>) -> (Arc<TestHost>, Core) {
    let host = Arc::new(TestHost::default());
    let core = Core::new(host.clone(), tools, host.dir.path().join(".sprixel")).unwrap();
    (host, core)
}

/// Sends one JSON-RPC request and returns the reply body.
pub fn rpc(core: &Core, session: Option<&str>, method: &str, params: Value) -> Value {
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    core.handle(session, body.to_string().as_bytes()).body.expect("a reply body")
}

/// Starts a session as a client with this name and returns its id.
pub fn connect(core: &Core, name: &str) -> String {
    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": { "name": name, "version": "1.0" } } });
    core.handle(None, body.to_string().as_bytes()).session.expect("a session id")
}

/// Like `connect`, and the user allows the client.
pub fn connect_allowed(core: &Core, name: &str) -> String {
    let session = connect(core, name);
    assert!(core.answer_client(name, "1.0", true).unwrap());
    session
}
