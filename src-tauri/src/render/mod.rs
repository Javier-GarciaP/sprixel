//! Draws frames of a sprite as RGBA images. Plain Rust, no Tauri types. The
//! export and the MCP `export` tool share it.

// Nothing outside the tests uses this module until the export is wired.
#![allow(dead_code, unused_imports)]

mod frame;
mod pack;
mod preview;

pub use frame::{Image, render_frame};
pub use preview::{Background, checker_cell, flatten};
pub use pack::{FRAMES_PER_ROW, Layout, Placement, pack_rows};

#[cfg(test)]
mod frame_tests;

#[cfg(test)]
mod pack_tests;

#[cfg(test)]
mod preview_tests;
