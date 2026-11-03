# Proposal

## Why

A real agent drew sprites and lost colors without knowing it: a grid with
`palette: skin` was drawn with the palette `main`, and a legend line for a key
that already existed kept the old color. Also, every sprite shared one palette,
so one sprite could not have its own colors.

## What Changes

- New tools: `create_palette`, `set_sprite_palette`, `remove_unused_colors` and
  `generate_ramp`. `create_sprite` gets `own_palette`.
- `apply_grid` never ignores a `palette:` line: a different palette gives
  `palette_mismatch`, an unknown one gives `palette_not_found`.
- A legend color that conflicts with a color of the palette gets a free key.
  The pixels are rewritten and the answer says `remapped` and `warnings`.
- New operation `SetSpritePalette` in the document model (needed to point a
  sprite to another palette, with undo).
- Color math for ramps in the Rust core.

## Capabilities

### New Capabilities
- `sprite-palettes`: own palettes, safe grid palettes and legends, clean-up and ramps.

### Modified Capabilities

## Impact

- Rust: `document/operation.rs`, new `document/ramp.rs`, `document/grid.rs`,
  new `mcp/tools/palettes.rs`, `mcp/tools/create.rs`, `mcp/tools/draw.rs`,
  `mcp/tools/docs.rs`, `mcp/activity.rs`.
- UI: activity names in `en.json` / `es.json`; the palette panel is checked.
- Docs: `docs/ai-and-mcp.md`, `docs/text-grid-format.md`, the guide text and
  `cli/internal/clients/skill/SKILL.md`.
- No new dependency. The file format does not change: a sprite already stores
  its palette name.
