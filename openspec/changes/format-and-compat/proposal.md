# Proposal

## Why

Before 1.0.0 the project format and the export must be safe to depend on. A
real agent and the first game-engine tests showed four gaps: old files must
keep opening when the format grows, the export JSON has no frame numbers and
no real pivot and does not match what Godot and Unity importers read, a sprite
cannot change its canvas size, and the way from drawing to a game engine is
not written down.

## What Changes

- Format: a documented version table and rules for changes. A file with no
  `formatVersion` is version 1. Old files (before anchors and frame durations
  existed) open. A newer file is refused and nothing is written. Fixture
  projects of old versions test it.
- Export JSON: frames get `index` (0, position in `frames`) and
  `frame_number` (1, number in the sprite); `pivot` uses the `feet` anchor;
  new option `json_format: "aseprite"` (tool `export` and a selector in the
  Export screen).
- Resize canvas: operation `ResizeSprite` with an anchor, tool `resize_sprite`
  and a dialog "Change canvas" in the sprite details panel.
- New page `docs/game-engines.md`: from drawing to Godot 4 and Unity.

## Capabilities

### New Capabilities
- `format-and-compat`: format versions, engine-ready export and canvas resize.

## Impact

Rust core (storage, document, export, one new MCP tool), Export screen,
sprite details panel, docs, `AGENTS.md`. No new dependency. Old project files
open unchanged; the Sprixel JSON stays at version 1 (only fields are added).
