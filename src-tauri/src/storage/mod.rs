//! Project files on disk: format, saving, trash and file watching.

// Nothing outside the tests uses this module yet (IPC and MCP come later).
#![allow(dead_code, unused_imports)]

mod error;
mod files;
mod format;

pub use error::StorageError;
pub use files::{
    palette_path, read_manifest, read_palette, read_sprite, sprite_path, write_manifest,
    write_palette, write_sprite,
};
pub use format::{
    FORMAT_VERSION, Manifest, manifest_from_text, manifest_to_text, palette_from_text,
    palette_to_text, sprite_from_text, sprite_to_text,
};

#[cfg(test)]
mod format_tests;
