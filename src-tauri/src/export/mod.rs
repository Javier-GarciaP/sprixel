//! Exports sprites to files a game engine can read: a sprite sheet with a
//! JSON file and single PNG files. Plain Rust, no Tauri types. The app and the
//! MCP `export` tool share it.

// Nothing outside the tests uses this module until the export is wired.
#![allow(dead_code, unused_imports)]

mod error;
mod plan;
mod png_files;
mod sheet;
mod write;

pub use error::ExportError;
pub use plan::{DEFAULT_NAMING, ExportFile, Format, Options, Scope};
pub use png_files::build_png_files;
pub use sheet::build;
pub use write::{DEFAULT_OUTPUT, Written, resolve_output, write_files};

#[cfg(test)]
mod sheet_tests;

#[cfg(test)]
mod png_files_tests;

#[cfg(test)]
mod write_tests;
