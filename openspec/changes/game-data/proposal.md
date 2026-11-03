# Proposal

## Why

A real agent drew sprites and asked for what a game engine needs: frame
durations it can set, anchor points (feet, hand), an export that an engine
can import, and a way to check that a tile repeats without seams.

## What Changes

- Tool `set_frame_duration`; `get_sprite` returns the duration of each frame.
- Anchor points: a named point per sprite, optionally per frame. New
  operations (with undo), file field, tools `set_anchor` and `remove_anchor`,
  returned by `get_sprite`.
- The export JSON gets `frames[].anchors`. `docs/export-format.md` explains the
  mapping to Godot (`SpriteFrames`) and Unity, with an example.
- Tools `check_tile` (do opposite edges match?) and `render_tiled` (3x3 image).

## Capabilities

### New Capabilities
- `game-data`: durations, anchors, engine export and seamless tiles.

## Impact

Rust core (document, storage, export, MCP tools), docs, skill text. No new
dependency. Old project files open unchanged.
