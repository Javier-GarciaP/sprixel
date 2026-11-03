# Proposal

## Why

A real agent drew sprites and could not see its work well. A white sprite was
invisible on the transparent background of the preview. It could look at only
one frame at a time, so it could not judge an animation, and it could not see
how pixels move between two frames.

## What Changes

- `render_preview` and the image that `apply_grid`, `set_pixels` and
  `draw_shape` return get a `background` option: `checker` (default, the
  neutral gray checkerboard of the editor), `light`, `dark` or `transparent`.
- New read tool `render_frames`: all frames (or a range) in one image, in a
  grid, with optional frame numbers.
- New read tool `render_onion`: several frames drawn over each other, earlier
  frames tinted and translucent, to see how pixels move.
- `sprixel tool` saves the images of the new tools as files, like it does for
  `render_preview`.
- The server guide, the Claude Code skill and the docs mention the new tools.

## Capabilities

### New Capabilities
- `agent-previews`: the images an agent can ask for to see its work.

### Modified Capabilities
None. The old behavior of `render_preview` changes only in its default
background (see the design).

## Impact

- Rust: `src-tauri/src/render/` (a new `preview.rs`) and `mcp/tools/read.rs`,
  `write.rs`, `draw.rs`.
- Go: `cli/internal/command/preview.go` (file names of the new images).
- Docs: `docs/ai-and-mcp.md`, the guide text, the skill, `CHANGELOG.md`.
- No new dependency.
