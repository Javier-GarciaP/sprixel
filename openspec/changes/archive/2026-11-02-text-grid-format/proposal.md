# Proposal

## Why

The text grid is the way an AI draws, and it also lets a person copy, paste and
fix a sprite as text. It needs one exact definition and one validator that
gives errors both a person and an AI can act on.

## What Changes

- Implement the text grid parser, writer and validator exactly as in
  `docs/text-grid-format.md`.
- Give precise errors with codes, line and row.
- Support writing a region at an `x` and `y` position.
- Build the Text view in the editor: text editor, live preview, Apply, Copy
  and Paste, and error display.

## Capabilities

### New Capabilities
- `text-grid`: the text format, parsing, writing, validation and errors.
- `text-view`: the editor view that shows and edits a sprite as a text grid.

### Modified Capabilities

## Impact

- New Rust module for the grid (shared by the app and the MCP tools) and a UI
  feature `src/features/text-view/`.
- The MCP tools `get_sprite` and `apply_grid` use this module.
- Defines the error codes the AI sees.
