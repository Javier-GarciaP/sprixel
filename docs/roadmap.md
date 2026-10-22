# Roadmap

This is the plan to reach **version 1.0.0**, the first stable release. Every
item is a spec in [`openspec/changes/`](../openspec/changes/). The code is
written spec by spec, in the order below.

> **Status:** planning. All specs are written. No code has been started.

## The goal of version 1

You can:

1. create a project and organize sprites in folders;
2. draw and animate sprites of any size up to 512x512;
3. connect Claude Code or OpenCode with one command and watch them draw;
4. export a sprite sheet with JSON, PNG files or a GIF;
5. install it with one command on Linux, Windows and macOS;
6. use it in Spanish or English, in light or dark.

## Specs

| # | Spec | What it delivers | Needs |
| --- | --- | --- | --- |
| 1 | [`project-foundation`](../openspec/changes/project-foundation/) | The Tauri + React + Go skeleton, folders, tooling and builds on 3 systems | |
| 2 | [`project-format-and-storage`](../openspec/changes/project-format-and-storage/) | The document model, the project folder, autosave, trash and migrations | 1 |
| 3 | [`app-shell-and-settings`](../openspec/changes/app-shell-and-settings/) | Window layout, tabs, light and dark theme, Spanish and English, settings and shortcuts | 1 |
| 4 | [`project-browser`](../openspec/changes/project-browser/) | Folders and sprites: create, rename, duplicate, move, delete, multi-select | 2, 3 |
| 5 | [`pixel-canvas-editor`](../openspec/changes/pixel-canvas-editor/) | The canvas, tools, zoom, undo and redo, onion skin | 2, 3 |
| 6 | [`frames-and-animations`](../openspec/changes/frames-and-animations/) | Frames, animation tags and the live preview | 5 |
| 7 | [`palette-editor`](../openspec/changes/palette-editor/) | Palettes with one-letter keys, editing, creating and importing | 2, 5 |
| 8 | [`text-grid-format`](../openspec/changes/text-grid-format/) | The text grid parser and validator, and the Text view | 5, 7 |
| 9 | [`export-spritesheet`](../openspec/changes/export-spritesheet/) | Sprite sheet + JSON, PNG files and GIF | 6 |
| 10 | [`mcp-server-and-tools`](../openspec/changes/mcp-server-and-tools/) | The MCP server, the tools, security, permissions, approval and AI undo | 2, 5, 6, 7, 8, 9 |
| 11 | [`agents-ui`](../openspec/changes/agents-ui/) | The Agents tab, connect dialog, activity, permissions and approval prompt | 3, 10 |
| 12 | [`cli-sprixel`](../openspec/changes/cli-sprixel/) | The `sprixel` command: bridge, setup, doctor and tools from the terminal | 10 |
| 13 | [`install-and-release`](../openspec/changes/install-and-release/) | Install scripts, release pipeline, installers, versioning and updates | 1, 12 |

## Milestones

| Milestone | Specs | Result |
| --- | --- | --- |
| **M1 · Foundation** | 1, 2, 3 | An empty app that opens and saves a project, with the theme and language working |
| **M2 · Draw** | 4, 5, 6, 7 | A real pixel art editor: projects, drawing, frames, animations and palettes |
| **M3 · Text and export** | 8, 9 | The text view and exporting to a game |
| **M4 · AI** | 10, 11, 12 | An AI can draw in the live app, with permissions and undo |
| **M5 · Release** | 13 | Install with one command and publish `1.0.0` |

Suggested versions: `0.1.0` at the end of M1, `0.2.0` at M2, `0.3.0` at M3,
`0.4.0` at M4, and `1.0.0` after M5 when everything is tested.

## How a spec is built

1. Read the spec (`proposal.md`, `design.md`, `specs/`, `tasks.md`).
2. Create `feature/<spec-name>` from `dev`.
3. Do the tasks one by one, with tests, one commit per task.
4. Open a pull request into `dev`.
5. Archive the spec when it is merged.

See [CONTRIBUTING.md](../CONTRIBUTING.md).

## Not in version 1

These were considered and are left for later:

- Layers in the interface (the file format already supports them).
- Tilemaps and level or scene editing.
- Importing PNG images.
- Export formats made for one engine (for example Godot `.tres`).
- Codex and other agents with one-command setup (any MCP client can still
  connect).
- A headless mode (running without a window).
- Cloud sync, accounts or collaboration.
- Languages other than Spanish and English.
