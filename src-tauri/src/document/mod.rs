//! The document model: project, folders, sprites, palettes, frames, layers
//! and animation tags, with their rules. Plain Rust, no Tauri types.

// Nothing outside the tests uses this module yet (IPC and MCP come later).
#![allow(dead_code, unused_imports)]

mod error;
mod external;
mod model;
mod names;
mod operation;
mod stroke;

pub use error::DocError;
pub use names::{copy_name, same_name, validate_name};
pub use operation::{Applied, AppliedGroup, Operation, PixelChange, SpriteRef, Target};
pub use stroke::ColorChange;
pub use model::{
    Color, Folder, Frame, Layer, MAX_COLORS, MAX_SIZE, Palette, Project, Sprite, TRANSPARENT, Tag,
    TagMode,
};

#[cfg(test)]
mod model_tests;

#[cfg(test)]
mod names_tests;

#[cfg(test)]
mod operation_tests;

#[cfg(test)]
mod stroke_tests;
