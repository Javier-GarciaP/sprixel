//! The tools an AI can call. Each file has the tools of one group.

mod args;
mod create;
mod docs;
mod draw;
mod read;
mod shapes;
mod structure;
mod write;

use super::Tool;

pub use read::OPEN_SPRITE_EVENT;

/// Every tool of the server, in the order a client sees them.
pub fn all() -> Vec<Tool> {
    let mut tools = Vec::new();
    tools.extend(read::tools());
    tools.extend(create::tools());
    tools.extend(draw::tools());
    tools.extend(structure::tools());
    tools
}

#[cfg(test)]
mod read_tests;

#[cfg(test)]
mod create_tests;

#[cfg(test)]
mod draw_tests;

#[cfg(test)]
mod structure_tests;
