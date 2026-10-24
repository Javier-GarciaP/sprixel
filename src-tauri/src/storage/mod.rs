//! Project files on disk: format, saving, trash and file watching.

// Nothing outside the tests uses this module yet (IPC and MCP come later).
#![allow(dead_code, unused_imports)]

mod atomic;
mod autosave;
mod error;
mod files;
mod format;
mod listing;
mod load;
mod migrate;
mod session;
mod trash;
mod watch;

pub use atomic::{DiskWriter, FileWriter, write_atomic};
pub use autosave::{Autosave, DEFAULT_DEBOUNCE};
pub use error::StorageError;
pub use files::{
    palette_path, read_manifest, read_palette, read_sprite, sprite_path, write_manifest,
    write_palette, write_sprite,
};
pub use session::{ConflictChoice, ExternalChange, SaveStatus, Session};
pub use listing::{Listing, SpriteInfo, list_project};
pub use load::Damaged;
pub use migrate::{FileKind, Migration, Migrator, Step, migrate_project};
pub use format::{
    FORMAT_VERSION, Manifest, manifest_from_text, manifest_to_text, palette_from_text,
    palette_to_text, sprite_from_text, sprite_to_text,
};

#[cfg(test)]
mod format_tests;

#[cfg(test)]
mod migrate_tests;

#[cfg(test)]
mod atomic_tests;

#[cfg(test)]
mod session_tests;
#[cfg(test)]
mod autosave_tests;
#[cfg(test)]
mod save_error_tests;
#[cfg(test)]
mod trash_tests;
#[cfg(test)]
mod watch_tests;
#[cfg(test)]
mod open_tests;
#[cfg(test)]
mod listing_tests;
#[cfg(test)]
mod docs_tests;
#[cfg(test)]
mod move_tests;
