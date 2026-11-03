# Proposal

## Why

An agent changes a sprite in one instant and the screen jumps to the result.
The person wants to see it work like a human, as in OpenPencil: a cursor that
moves and pixels that appear. Also, the first designs need method to look
good: a small palette, shading, outlines and a review loop.

## What Changes

- The core emits the pixels of every agent write as an ordered list (event
  `agent-ops`). The editor replays them with a moving cursor that carries the
  agent name, and the frame stays busy until the replay ends.
- The replay has a time limit, can be skipped with Stop, and respects the
  reduced motion setting.
- The editor follows the agent to the sprite and frame it edits, without
  stealing focus from what the person is doing.
- A tool `get_guide` serves pixel art guidance (design, animation, palette). The
  server instructions and the Claude Code skill use the same text and a worked
  example.
- A scripted demo checks the full experience with a real agent.

## Capabilities

### New Capabilities
- `agent-live-view`: replay of agent writes with a cursor in the editor.
- `agent-design-guide`: guidance for good pixel art served to agents.

### Modified Capabilities

## Impact

- Rust: `src-tauri/src/mcp/tools/write.rs` (new event), `ipc/editor.rs` (replay
  done command), `tools/docs.rs`, new tool `get_guide`.
- UI: `src/features/canvas-editor` (replay hook, cursor overlay, editor screen).
- CLI: skill text in `cli/internal/clients/skill/SKILL.md`.
- Docs: `docs/ai-and-mcp.md`, `docs/demo.md`.
- Depends on `agent-usability-fixes` (the skill file and `open_project`).
- No new dependency.
