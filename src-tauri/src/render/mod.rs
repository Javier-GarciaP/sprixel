//! Draws frames of a sprite as RGBA images. Plain Rust, no Tauri types. The
//! export and the MCP `export` tool share it.

// Nothing outside the tests uses this module until the export is wired.
#![allow(dead_code, unused_imports)]

mod frame;

pub use frame::{Image, render_frame};

#[cfg(test)]
mod frame_tests;
