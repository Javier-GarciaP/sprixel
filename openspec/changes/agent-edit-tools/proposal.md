# Proposal

## Why

A real agent drew sprites and found that editing and reading big sprites is
expensive: to flip a leg or move a head it had to rewrite whole grids, it could
not draw "only on the skin color", it had to draw both halves of a symmetric
sprite, `get_sprite` returned the whole sprite, and it could not use a
reference picture.

## What Changes

- New tool `transform_region`: flip, mirror, move (with `wrap`) and rotate a
  region of one frame.
- New tool `copy_region`: copy a region inside a sprite, or to another sprite or
  frame, with an offset and `skip_transparent`.
- New argument `only_over` (a palette key) on `draw_shape`, `set_pixels` and the
  region form of `apply_grid`: only pixels that now have that key change.
- New argument `symmetry` (`horizontal`, `vertical`, `both`, with an optional
  `axis`) on `set_pixels`, `draw_shape` and `apply_grid`.
- `get_sprite` gets `region` and `include_palette`.
- New tool `import_reference`: reads a PNG from the folder `references/` of the
  project and writes it into a frame, reduced to the palette.

## Capabilities

### New Capabilities
- `agent-edit-tools`: region tools, `only_over`, symmetry, partial reads and
  reference images.

### Modified Capabilities

## Impact

- Rust: `src-tauri/src/mcp/tools/` (new `region.rs`, `reference.rs`, `edit.rs`
  helpers; changes in `draw.rs`, `read.rs`, `mod.rs`), `src-tauri/src/mcp/activity.rs`,
  `src-tauri/src/document/grid.rs` (partial grid).
- Docs: `docs/ai-and-mcp.md`, `docs/text-grid-format.md`, the guide text and
  the Claude Code skill, `CHANGELOG.md`.
- No new dependency (the `image` crate with png is already used).
- No change in the project file format.
