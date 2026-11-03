//! The MCP server: AI tools connect to the running app, read the project and
//! change it with operations. Plain Rust (no window), so the tests drive it
//! directly. See `docs/ai-and-mcp.md` and the `mcp-server-and-tools` spec.

// The app does not start the server yet.
#![allow(dead_code, unused_imports)]

mod base64;
mod clients;
mod host;
mod http;
mod limits;
mod policy;
mod random;
mod rpc;
mod server;
mod sessions;
mod token;
mod tool;
mod tools;

#[cfg(test)]
mod testing;

pub use clients::{ClientInfo, Standing};
pub use policy::Permissions;
pub use host::{Host, TauriHost};
pub use rpc::Reply;
pub use server::Server;
pub use sessions::{MAX_SESSIONS, Session};
pub use tool::{Ctx, Need, Output, Tool, ToolError, ToolResult};

use std::path::PathBuf;
use std::collections::VecDeque;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

/// Everything the server shares: the host, the tools and the sessions.
pub struct Core {
    pub(crate) host: Arc<dyn Host>,
    /// The Sprixel data folder (`~/.sprixel`): token, `runtime.json`, clients.
    pub(crate) data_dir: PathBuf,
    /// The secret every request must carry. Never logged.
    pub(crate) token: String,
    pub(crate) tools: Vec<Tool>,
    pub(crate) sessions: Mutex<sessions::Sessions>,
    pub(crate) clients: Mutex<clients::Clients>,
    pub(crate) permissions: Mutex<Permissions>,
    pub(crate) limits: limits::Limits,
    pub(crate) rate: Mutex<limits::RateLimiter>,
    /// Connections being served now.
    pub(crate) connections: AtomicUsize,
    /// The last log lines (see `log`).
    pub(crate) logs: Mutex<VecDeque<String>>,
}

impl Core {
    /// Opens the data folder: reads the token or makes a new one.
    pub fn new(host: Arc<dyn Host>, tools: Vec<Tool>, data_dir: PathBuf) -> std::io::Result<Core> {
        let token = token::load_or_create(&data_dir)?;
        Ok(Core {
            host,
            data_dir: data_dir.clone(),
            token,
            tools,
            sessions: Mutex::default(),
            clients: Mutex::new(clients::Clients::load(&data_dir)),
            permissions: Mutex::new(Permissions::load(&data_dir)),
            limits: limits::Limits::default(),
            rate: Mutex::default(),
            connections: AtomicUsize::new(0),
            logs: Mutex::default(),
        })
    }
}

/// Starts the server for the app: opens the data folder `~/.sprixel`, makes
/// the core with every tool, and listens. The core is shared with the commands
/// of the UI (approval, permissions, activity).
pub fn start(app: tauri::AppHandle) -> std::io::Result<(Arc<Core>, Server)> {
    let home = tauri::Manager::path(&app)
        .home_dir()
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    let host = Arc::new(TauriHost { app });
    let core = Arc::new(Core::new(host, tools::all(), home.join(".sprixel"))?);
    let server = Server::start(core.clone())?;
    Ok((core, server))
}

/// The event sent when a client waits for the user to allow or reject it.
pub const CLIENT_PENDING_EVENT: &str = "mcp://client-pending";
/// The event sent when the list of clients changes.
pub const CLIENTS_EVENT: &str = "mcp://clients-changed";
/// The event sent when the permissions change.
pub const PERMISSIONS_EVENT: &str = "mcp://permissions-changed";

impl Core {
    pub fn permissions(&self) -> Permissions {
        *self.permissions.lock().unwrap()
    }

    /// Changes the permissions. They apply to the next call of every client,
    /// with no reconnecting.
    pub fn set_permissions(&self, permissions: Permissions) -> std::io::Result<()> {
        permissions.save(&self.data_dir)?;
        *self.permissions.lock().unwrap() = permissions;
        self.host.emit(PERMISSIONS_EVENT, serde_json::to_value(permissions).unwrap_or_default());
        Ok(())
    }

    /// The clients and their state, for the Agents tab.
    pub fn clients(&self) -> Vec<ClientInfo> {
        self.clients.lock().unwrap().list()
    }

    /// The user allows or rejects a client that waits. False if none waits.
    pub fn answer_client(&self, name: &str, version: &str, allow: bool) -> std::io::Result<bool> {
        let done = self.clients.lock().unwrap().answer(name, version, allow)?;
        if done {
            self.log(&format!("client {}", if allow { "allowed" } else { "rejected" }));
            self.host.emit(CLIENTS_EVENT, serde_json::Value::Null);
        }
        Ok(done)
    }

    /// The user removes a client. Its sessions end, so its next call fails
    /// and it has to ask again.
    pub fn remove_client(&self, name: &str, version: &str) -> std::io::Result<bool> {
        let removed = self.clients.lock().unwrap().remove(name, version)?;
        self.sessions.lock().unwrap().end_client(name, version);
        if removed {
            self.log("client removed");
            self.host.emit(CLIENTS_EVENT, serde_json::Value::Null);
        }
        Ok(removed)
    }

    /// Writes a line to the log (the terminal, and the last 200 lines in
    /// memory). A line says what happened and why. It must never have the
    /// token, nor anything a client sent.
    pub fn log(&self, line: &str) {
        eprintln!("[mcp] {line}");
        let mut logs = self.logs.lock().unwrap();
        if logs.len() >= 200 {
            logs.pop_front();
        }
        logs.push_back(line.to_string());
    }
}

#[cfg(test)]
mod protocol_tests;

#[cfg(test)]
mod server_tests;

#[cfg(test)]
mod http_tests;

#[cfg(test)]
mod approval_tests;

#[cfg(test)]
mod policy_tests;
