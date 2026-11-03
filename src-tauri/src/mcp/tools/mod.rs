//! The tools an AI can call. Each file has the tools of one group.

use super::Tool;

/// Every tool of the server, in the order a client sees them.
pub fn all() -> Vec<Tool> {
    Vec::new()
}
