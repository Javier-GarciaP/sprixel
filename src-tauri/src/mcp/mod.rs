//! The MCP server: AI tools connect to the running app, read the project and
//! change it with operations. Plain Rust (no window), so the tests drive it
//! directly. See `docs/ai-and-mcp.md` and the `mcp-server-and-tools` spec.

// The app does not start the server yet.
#![allow(dead_code, unused_imports)]

mod base64;
mod host;
mod http;
mod random;
mod rpc;
mod server;
mod sessions;
mod token;
mod tool;

#[cfg(test)]
mod testing;

pub use host::{Host, TauriHost};
pub use rpc::Reply;
pub use server::Server;
pub use sessions::{MAX_SESSIONS, Session};
pub use tool::{Ctx, Need, Output, Tool, ToolError, ToolResult};

use std::path::PathBuf;
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
}

impl Core {
    /// Opens the data folder: reads the token or makes a new one.
    pub fn new(host: Arc<dyn Host>, tools: Vec<Tool>, data_dir: PathBuf) -> std::io::Result<Core> {
        let token = token::load_or_create(&data_dir)?;
        Ok(Core { host, data_dir, token, tools, sessions: Mutex::default() })
    }
}

#[cfg(test)]
mod protocol_tests;

#[cfg(test)]
mod server_tests;
