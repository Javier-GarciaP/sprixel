//! The tools an AI can call. Each file has the tools of one group.

/// The sentence every write tool ends with. A macro, so `concat!` can use it
/// in a description.
macro_rules! write_note {
    () => {
        " Returns the new `revision`. Pass `base_revision` (the revision you last read) to fail with `stale_revision` instead of overwriting newer changes. The whole call is applied or nothing is, and the person can undo it."
    };
}

mod args;
mod create;
mod docs;
mod draw;
mod export;
mod projects;
mod read;
mod shapes;
mod structure;
mod undo;
pub(super) mod write;

use super::Tool;

pub use docs::INSTRUCTIONS;
pub use projects::PROJECT_OPENED_EVENT;
pub use read::OPEN_SPRITE_EVENT;

/// Every tool of the server, in the order a client sees them.
pub fn all() -> Vec<Tool> {
    let mut tools = Vec::new();
    tools.extend(read::tools());
    tools.extend(projects::tools());
    tools.extend(create::tools());
    tools.extend(draw::tools());
    tools.extend(structure::tools());
    tools.extend(export::tools());
    tools.extend(undo::tools());
    tools
}

#[cfg(test)]
mod read_tests;

#[cfg(test)]
mod projects_tests;

#[cfg(test)]
mod create_tests;

#[cfg(test)]
mod draw_tests;

#[cfg(test)]
mod structure_tests;

#[cfg(test)]
mod export_tests;

#[cfg(test)]
mod revision_tests;

#[cfg(test)]
mod docs_tests;

#[cfg(test)]
mod undo_tests;
