# Proposal

## Why

Colors are the base of pixel art, and in Sprixel each color also has a
one-letter key that the text grid and the AI use. Palettes must be easy to edit
without breaking sprites.

## What Changes

- Build the palette panel in the editor with keys, a "Show keys" switch and
  an Edit button.
- Build the palette editor dialog: list of colors, name, key, hex and RGB
  sliders, add and remove colors.
- Apply color changes to every sprite that uses the palette.
- Create palettes (empty, from a preset) and import them from `.gpl` and `.hex`
  files.
- Warn when changing a key would break existing text grids.

## Capabilities

### New Capabilities
- `palette-management`: palettes, their colors and keys, editing, creating and importing.

### Modified Capabilities

## Impact

- New feature folder `src/features/palette-editor/`.
- Rewriting a key touches every sprite file that uses the palette
  (see `project-storage`).
- The text grid and MCP tools rely on the keys defined here.
