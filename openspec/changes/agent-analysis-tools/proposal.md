# Proposal

## Why

A real agent drew sprites by writing every pixel by hand: outlines, shading and
clean-up cost many calls and often had mistakes (loose pixels, holes in the
outline, a body that moves between frames). The server can do this work with
simple exact rules, and can check a sprite and say what is wrong.

## What Changes

- New tool `auto_outline`: draws a 1 pixel outline around the shape of a frame.
- New tool `auto_shade`: recolors the lit side and the shadow side of a region
  with a ramp of keys, by a fixed rule.
- New read-only tool `analyze_sprite`: reports loose pixels, gaps in the
  outline, unused palette colors, the color count and, for sprites with more
  than one frame, how each frame differs from the previous one.
- The logic is a pure Rust module (`analysis`) with exact unit tests. The tools
  are thin wrappers.
- Docs, the design guide and the Claude Code skill name the new tools.

## Capabilities

### New Capabilities
- `sprite-analysis-tools`: automatic outline, automatic shading and sprite analysis for agents.

### Modified Capabilities

## Impact

- Rust: new `src-tauri/src/analysis/`, new `src-tauri/src/mcp/tools/analysis.rs`,
  `tools/mod.rs`, `activity.rs`, `tools/docs.rs`.
- UI: two strings for the activity list (`agents.tool.auto_outline`,
  `agents.tool.auto_shade`) in `en.json` and `es.json`.
- CLI: skill text in `cli/internal/clients/skill/SKILL.md` (kept equal to the guide).
- Docs: `docs/ai-and-mcp.md`, `CHANGELOG.md`.
- No new dependency. No file format change.
