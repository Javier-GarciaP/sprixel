//! Exports sprites to files a game engine can read: a sprite sheet with a
//! JSON file and single PNG files. Plain Rust, no Tauri types. The app and the
//! MCP `export` tool share it.

// Nothing outside the tests uses this module until the export is wired.
#![allow(dead_code, unused_imports)]

mod error;
mod plan;
mod sheet;

pub use error::ExportError;
pub use plan::{ExportFile, Format, Options, Scope};
pub use sheet::build;

#[cfg(test)]
mod sheet_tests;
