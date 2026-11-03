//! The tools an AI can call. Each file has the tools of one group.

mod args;
mod read;

use super::Tool;

pub use read::OPEN_SPRITE_EVENT;

/// Every tool of the server, in the order a client sees them.
pub fn all() -> Vec<Tool> {
    let mut tools = Vec::new();
    tools.extend(read::tools());
    tools
}

#[cfg(test)]
mod read_tests;
