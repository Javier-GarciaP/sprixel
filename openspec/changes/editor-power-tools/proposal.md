# Proposal

## Why

An AI agent can already flip, mirror, move and copy regions, draw with
symmetry, outline and shade a sprite, review it, make color ramps, remove unused
colors, set frame durations and anchors, and check tiles. A person in the
editor cannot do any of this. People should have the same power tools.

## What Changes

- New IPC commands that call the same pure functions the MCP tools use, with the
  author `user`. Nothing is copied: the Rust core still owns the data.
- Selection bar: flip, rotate, move with the arrow keys (with wrap), mirror a
  half; shortcuts for each.
- Copy and paste of a region between frames and sprites (same palette).
- Symmetry mode for the pencil, line, rectangle, ellipse and bucket, with the
  axis drawn on the canvas.
- Outline and Shade dialogs with a live preview on the canvas.
- Review panel: loose pixels, outline gaps, unused colors, frame comparison.
- Palette editor: Generate ramp and Remove unused colors.
- Frames strip: duration per frame. Anchors: create, drag, rename, delete.
- Preview tools: background (checker, light, dark), 3x3 tile preview with seam
  warning, onion skin with 1 to 3 frames.
- A manual test page: `docs/editor-tools-test.md`.

## Capabilities

### New Capabilities
- `editor-power-tools`: the power tools for people in the editor.

### Modified Capabilities

## Impact

- Rust: new `src-tauri/src/ipc/power.rs` (and small `pub(crate)` changes in
  `mcp/tools/{region,palettes,gamedata}.rs`, behavior unchanged);
  `ipc/frames.rs`; `lib.rs` registers the commands.
- UI: `src/features/canvas-editor`, `frames-animations`, `palette-editor`,
  `app/commands/registry.ts`, `en.json` and `es.json`.
- Docs: `docs/editor-tools-test.md`, `CHANGELOG.md`.
- No new dependency. No change in the project file format.
