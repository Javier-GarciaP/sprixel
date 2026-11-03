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
    /// The recent projects list the app would keep.
    pub recent: Mutex<Vec<crate::ipc::preferences::RecentProject>>,
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

    fn recent_projects(&self) -> Vec<crate::ipc::preferences::RecentProject> {
        self.recent.lock().unwrap().clone()
    }

    fn open_project(
        &self,
        path: &std::path::Path,
    ) -> Result<crate::ipc::project::ProjectInfo, crate::ipc::project::ProjectError> {
        let info = crate::ipc::project::open_project(&self.project, path)?;
        let mut data = crate::ipc::preferences::Data::default();
        data.recent_projects = self.recent.lock().unwrap().clone();
        crate::ipc::preferences::add_recent(
            &mut data,
            crate::ipc::preferences::RecentProject { path: info.path.clone(), name: info.name.clone() },
        );
        *self.recent.lock().unwrap() = data.recent_projects;
        Ok(info)
    }
}

impl Default for TestHost {
    fn default() -> Self {
        TestHost {
            project: ProjectState::default(),
            agents: EditorState::default(),
            events: Mutex::default(),
            recent: Mutex::default(),
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

/// A core with every real tool, and a client "Claude Code" that is allowed.
pub fn setup() -> (Arc<TestHost>, Core, String) {
    let (host, core) = core_with(super::tools::all());
    let session = connect_allowed(&core, "Claude Code");
    (host, core, session)
}

/// Opens a project in the host with: the palette `main` (R red, G green), the
/// folder `Characters` and in it the 4x4 sprite `goblin` with 2 frames and
/// the animation `walk` (frames 1 to 2), and the 8x8 sprite `slime` in the
/// root. The setup is not an action of the user, so the history is empty.
pub fn open_demo_project(host: &TestHost) {
    use crate::document::{Color, Operation, Palette, PixelChange, SpriteRef, Tag, TagMode};
    let state = &host.project;
    crate::ipc::project::create_project(state, host.dir.path(), "Kit").unwrap();
    crate::ipc::project::edit(state, |session| {
        let goblin = SpriteRef::new(&["Characters"], "goblin");
        let color = |key: char, name: &str, hex: &str| Color { key, name: name.to_string(), hex: hex.to_string() };
        session
            .apply(
                "user",
                vec![
                    Operation::CreatePalette {
                        palette: Palette {
                            name: "main".to_string(),
                            colors: vec![color('R', "red", "#FF0000"), color('G', "green", "#00FF00")],
                        },
                        index: None,
                    },
                    Operation::CreateFolder { parent: vec![], name: "Characters".to_string() },
                    Operation::CreateSprite {
                        folder: vec!["Characters".to_string()],
                        name: "goblin".to_string(),
                        width: 4,
                        height: 4,
                        palette: "main".to_string(),
                    },
                    Operation::CreateSprite {
                        folder: vec![],
                        name: "slime".to_string(),
                        width: 8,
                        height: 8,
                        palette: "main".to_string(),
                    },
                    Operation::SetPixels {
                        sprite: goblin.clone(),
                        layer: 0,
                        frame: 1,
                        pixels: vec![PixelChange { x: 0, y: 0, key: 'R' }, PixelChange { x: 3, y: 3, key: 'G' }],
                    },
                    Operation::AddFrame { sprite: goblin.clone(), duration_ms: 100 },
                    Operation::AddTag {
                        sprite: goblin,
                        tag: Tag { name: "walk".to_string(), from: 1, to: 2, fps: 12, mode: TagMode::Loop },
                        index: None,
                    },
                ],
            )
            .unwrap();
        session.history.clear();
    })
    .unwrap();
}

/// Calls a tool. `Ok` is the JSON the tool gave, `Err` the JSON of its error.
pub fn call(core: &Core, session: &str, tool: &str, arguments: Value) -> Result<Value, Value> {
    let reply = rpc(core, Some(session), "tools/call", json!({ "name": tool, "arguments": arguments }));
    let result = &reply["result"];
    let text = result["content"][0]["text"].as_str().unwrap_or("null");
    let value: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    if result["isError"] == true { Err(value) } else { Ok(value) }
}

/// The revision of the open project.
pub fn revision(host: &TestHost) -> u64 {
    crate::ipc::project::read(&host.project, |s| s.project.revision).unwrap()
}

/// The pixels of a frame of the sprite as text rows, with `/` between rows.
pub fn rows(host: &TestHost, folder: &[&str], name: &str, frame: usize) -> String {
    use crate::document::SpriteRef;
    crate::ipc::project::read(&host.project, |s| {
        let sprite = s.project.find_sprite(&SpriteRef::new(folder, name)).unwrap();
        let width = sprite.width as usize;
        sprite.layers[0].frames[frame - 1]
            .pixels
            .chunks(width)
            .map(|r| String::from_utf8_lossy(r).into_owned())
            .collect::<Vec<_>>()
            .join("/")
    })
    .unwrap()
}
