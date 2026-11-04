# Proposal

## Why

An agent animated a 64x64 goat with 22 frames. All frames were stacked in one list, a Python script outside
the app made the poses, and the person saw no cursor. Animations are only tags over one frame list, with no
tool to rename, duplicate, move or delete them, and nothing in the guide tells an agent to start from a base
pose or not to generate pixels outside the app.

## What Changes

- One pattern, like Aseprite, Godot and Unity: a base frame and several named animations (tags), each with
  its own contiguous frames that never overlap another animation.
- The frames strip groups the frames by animation (collapsible header with name, fps, loop and play), with a
  "Base" group first and a "No animation" group last, and a "New animation..." action that copies the base.
- New agent tools: `create_animation`, `duplicate_animation`, `rename_animation`, `delete_animation`,
  `move_frame`; `transform_region` and `copy_region` accept `frames`; named `parts` (reusable rectangles)
  in a second phase.
- New guide topic `animations-from-base`, and a rule in the skill and the server instructions: never
  generate pixels outside the app (scripts, PIL) without the agreement of the person.

## Capabilities

### New Capabilities
- `animation-organization`: how animations are created, grouped, edited and taught to agents.

### Modified Capabilities

## Impact

- Rust: `src-tauri/src/document/operation.rs`, `src-tauri/src/mcp/tools/` (animation, region), `docs.rs`.
- UI: `src/features/frames-animations/` and i18n.
- Docs and skill: `docs/ai-and-mcp.md`, `cli/internal/clients/skill/SKILL.md`.
- The file format does not change in phase 1 (tags stay). Parts (phase 2) add an optional list: backward
  compatible, with a `format_version` entry in `docs/project-format.md`.
- No new dependency.
