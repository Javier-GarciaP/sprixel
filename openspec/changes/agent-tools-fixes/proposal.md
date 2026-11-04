# Proposal

## Why

A real agent drew sprites with the tools and found problems: it could not try
an outline or a shade without writing it; `analyze_sprite` said "all good" for
things it did not check; `ground_kept` was false when only one foot was lifted;
a `.` in a small grid erased the sprite by accident; and a frame duration of
100 ms means two different things. The guides also did not say how a walk or an
attack must look.

## What Changes

- `dry_run` on `auto_outline`, `auto_shade`, `transform_region`, `copy_region`
  and `apply_grid`: nothing is written, the answer has the counts and a preview.
- Warnings: `auto_outline` with `inner` on thin parts; `auto_shade` on speckled
  shading.
- `analyze_sprite` returns `checked` and `not_checked`, and detects the outline
  key when it is not given.
- `ground_kept` follows the support (the `feet` anchor, or the lowest opaque row).
- `apply_grid` in region form does not erase with `.` by default. Every write
  reports `erased`.
- `get_sprite` shows `null` for a frame that follows the animation speed, and
  `set_frame_duration` accepts `null` to go back to it.
- Guides, skill and instructions teach all of this, and walk and attack rules.

## Capabilities

### New Capabilities
- `agent-tools-fixes`: dry runs, honest analysis, support-based ground check,
  safe region writes, frame duration at the tool boundary, guide rules.

### Modified Capabilities

## Impact

- Rust: `src-tauri/src/mcp/tools/{write,analysis,region,draw,gamedata,read,docs}.rs`,
  `src-tauri/src/analysis/*` (new functions only; old signatures stay).
- Docs: `docs/ai-and-mcp.md`, `docs/text-grid-format.md`, `CHANGELOG.md`.
- Skill: `cli/internal/clients/skill/SKILL.md`.
- No change to the project file format or to the export JSON.
