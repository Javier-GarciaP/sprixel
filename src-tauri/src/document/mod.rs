//! The document model: project, folders, sprites, palettes, frames, layers
//! and animation tags, with their rules. Plain Rust, no Tauri types.

// Nothing outside the tests uses this module yet (IPC and MCP come later).
#![allow(dead_code, unused_imports)]

mod animation;
mod error;
mod external;
mod grid;
mod import;
mod model;
mod names;
mod operation;
mod palette;
mod presets;
mod ramp;
mod resize;
mod stroke;

pub use animation::{create_animation, delete_animation, duplicate_animation, rename_animation};
pub use error::DocError;
pub use names::{copy_name, same_name, validate_name};
pub use operation::{Applied, AppliedGroup, Operation, PixelBatch, PixelChange, SpriteRef, Target};
pub use import::{ImportFormat, palette_from_file};
pub use presets::{PRESETS, Preset};
pub use ramp::ramp;
pub use resize::{ResizeAnchor, move_anchors, resize_pixels};
pub use palette::{ColorEdit, check_colors};
pub use grid::{Grid, GridError, GridOptions, GridPlan, Remap, write_grid, write_grid_part};
pub use stroke::ColorChange;
pub use model::{
    Anchor, Color, DEFAULT_FRAME_MS, Folder, Frame, Layer, MAX_COLORS, MAX_PARTS, Part, frame_duration_ms, MAX_SIZE, Palette, Project, Sprite, TRANSPARENT, Tag,
    TagMode,
};

#[cfg(test)]
mod model_tests;

#[cfg(test)]
mod names_tests;

#[cfg(test)]
mod operation_tests;

#[cfg(test)]
mod frame_tests;

#[cfg(test)]
mod animation_tests;

#[cfg(test)]
mod part_tests;

#[cfg(test)]
mod stroke_tests;

#[cfg(test)]
mod palette_tests;

#[cfg(test)]
mod palette_ops_tests;

#[cfg(test)]
mod presets_tests;

#[cfg(test)]
mod import_tests;

#[cfg(test)]
mod grid_tests;

#[cfg(test)]
mod anchor_tests;

#[cfg(test)]
mod ramp_tests;

#[cfg(test)]
mod resize_tests;

#[cfg(test)]
mod resize_op_tests;
