# Proposal

## Why

The server now has many tools (palettes, previews, edit, analysis, game data),
but the guides still teach only the first workflow. A real agent drew big
sprites, faces, walk cycles and game assets without help: it did not know which
tool to use, how many frames a walk needs, or how to check the feet. The agent
needs richer guides, each with an example that really works.

## What Changes

- `get_guide` gets five new topics: `large-sprites`, `faces-and-animals`,
  `walk-and-attack-cycles`, `game-ready` and `tools` (a map of which tool to
  use for what). The topics `design`, `animation` and `palette` stay.
- Each new guide has a worked example. A Rust test runs every call of the
  example through the real tools and checks the result.
- A test checks that every tool, argument and error code a guide names exists.
- The server instructions, the Claude Code skill and `docs/ai-and-mcp.md`
  list the topics. A test keeps the list equal in all three.
- No new tool, no new dependency, no file format change.

## Capabilities

### New Capabilities
- `agent-guide-topics`: the five new guide topics, their worked examples and the checks that keep guides, instructions, skill and docs equal.

### Modified Capabilities

## Impact

- Rust: `src-tauri/src/mcp/tools/docs.rs` (texts and topic list),
  `guide_tests.rs`, new `guide_examples_tests.rs`.
- CLI: `cli/internal/clients/skill/SKILL.md` (kept equal to the server text).
- Docs: `docs/ai-and-mcp.md`, `CHANGELOG.md`.
