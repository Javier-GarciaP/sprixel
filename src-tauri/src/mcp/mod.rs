//! The MCP server: AI tools connect to the running app, read the project and
//! change it with operations. Plain Rust (no window), so the tests drive it
//! directly. See `docs/ai-and-mcp.md` and the `mcp-server-and-tools` spec.

// The app does not start the server yet.
#![allow(dead_code, unused_imports)]

mod base64;
mod host;
mod random;
mod rpc;
mod sessions;
mod tool;

#[cfg(test)]
mod testing;

pub use host::{Host, TauriHost};
pub use rpc::Reply;
pub use sessions::{MAX_SESSIONS, Session};
pub use tool::{Ctx, Need, Output, Tool, ToolError, ToolResult};

use std::sync::{Arc, Mutex};

/// Everything the server shares: the host, the tools and the sessions.
pub struct Core {
    pub(crate) host: Arc<dyn Host>,
    pub(crate) tools: Vec<Tool>,
    pub(crate) sessions: Mutex<sessions::Sessions>,
}

impl Core {
    pub fn new(host: Arc<dyn Host>, tools: Vec<Tool>) -> Core {
        Core { host, tools, sessions: Mutex::default() }
    }
}

#[cfg(test)]
mod protocol_tests;
