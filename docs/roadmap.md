# Roadmap

This is the plan to reach **version 1.0.0**, the first stable release. Every
item is a spec in [`openspec/changes/`](../openspec/changes/). The code is
written spec by spec.

> **Status:** specs 1 to 3 (milestone M1) are built and archived. The current
> goal is **milestone M2, the demo**. Specs 4 to 13 are written; the demo
> builds most of them (see below) and version 1 finishes the rest.

## The goal of version 1

You can:

1. create a project and organize sprites in folders;
2. draw and animate sprites of any size up to 512x512;
3. connect Claude Code or OpenCode with one command and watch them draw;
4. export a sprite sheet with JSON, PNG files or a GIF;
5. install it with one command on Linux, Windows and macOS;
6. use it in Spanish or English, in light or dark.

## The demo (milestone M2)

The first public goal is a **demo where an AI draws and animates sprites in the
real editor while you watch**, and where a person without an AI can draw with
any color. Sprites for role playing and farming games (16x16 and 32x32, with
walk animations) are the target.

The demo is built from the specs below, in this order. Each spec has a **demo
cut** (the tasks done for the demo) and an **after the demo** group (the tasks
left for version 1, which stay unchecked in the spec). A spec with tasks left
is archived when version 1 is done.

| Order | Spec | In the demo | After the demo |
| --- | --- | --- | --- |
| 1 | `project-browser` | tree, sprite cards, details, New sprite and New folder, rename, duplicate, delete with Undo, open in the editor | search, move, folder delete, multi-select, context menu |
| 2 | `pixel-canvas-editor` | all: canvas, zoom, tools, onion skin, Select and Move, agent marker | |
| 3 | `frames-and-animations` | all | |
| 4 | `palette-editor` | all: palette that grows with the colors you use, presets, import `.hex` and `.gpl` | |
| 5 | `text-grid-format` | all, including the Text view | |
| 6 | `mcp-server-and-tools` | the server, security, approval, permissions, every tool except `export`, activity, Stop and AI undo | lost connection detection |
| 7 | `agents-ui` | all | lost connection state |
| 8 | `cli-sprixel` | the bridge, `mcp add`, `doctor`, `status`, `tools` and `tool` | tests on Windows and macOS |
| 9 | `export-spritesheet` | sprite sheet, JSON, PNG files, the Export tab and the `export` tool | GIF, trim and pivot, progress |
| 10 | demo polish | `docs/demo.md`, an example project, an end to end test | |

`install-and-release` is not part of the demo. The demo runs from the source
code (see the [README](../README.md#try-it-from-the-source-code)).

### The color model

Palettes are not required to draw. A person can use **any color** and the
sprite's palette **grows by itself** with each new color. A sprite always has a
palette: if there is none, the app creates one called `main` the first time.
For an AI, the palette is its language: every pixel is a one-letter key, so the
AI can read and edit a sprite that a person drew by hand. In the text format, a
`legend` line with a `#hex` for a key that does not exist adds that color. See
the `palette-editor` and `drawing-tools` specs.

## Specs

| # | Spec | What it delivers | Needs |
| --- | --- | --- | --- |
| 1 | [`project-foundation`](../openspec/changes/archive/2026-10-22-project-foundation/) | The Tauri + React + Go skeleton, folders, tooling and builds on 3 systems | |
| 2 | [`project-format-and-storage`](../openspec/changes/archive/2026-10-22-project-format-and-storage/) | The document model, the project folder, autosave, trash and migrations | 1 |
| 3 | [`app-shell-and-settings`](../openspec/changes/archive/2026-10-22-app-shell-and-settings/) | Window layout, tabs, light and dark theme, Spanish and English, settings and shortcuts | 1 |
| 4 | [`project-browser`](../openspec/changes/archive/2026-10-24-project-browser/) | Folders and sprites: create, rename, duplicate, move, delete, multi-select | 2, 3 |
| 5 | [`pixel-canvas-editor`](../openspec/changes/archive/2026-10-24-pixel-canvas-editor/) | The canvas, tools, zoom, undo and redo, onion skin | 2, 3 |
| 6 | [`frames-and-animations`](../openspec/changes/archive/2026-10-24-frames-and-animations/) | Frames, animation tags and the live preview | 5 |
| 7 | [`palette-editor`](../openspec/changes/palette-editor/) | Palettes with one-letter keys, editing, creating and importing | 2, 5 |
| 8 | [`text-grid-format`](../openspec/changes/text-grid-format/) | The text grid parser and validator, and the Text view | 5, 7 |
| 9 | [`export-spritesheet`](../openspec/changes/export-spritesheet/) | Sprite sheet + JSON, PNG files and GIF | 6 |
| 10 | [`mcp-server-and-tools`](../openspec/changes/mcp-server-and-tools/) | The MCP server, the tools, security, permissions, approval and AI undo | 2, 5, 6, 7, 8, 9 |
| 11 | [`agents-ui`](../openspec/changes/agents-ui/) | The Agents tab, connect dialog, activity, permissions and approval prompt | 3, 10 |
| 12 | [`cli-sprixel`](../openspec/changes/cli-sprixel/) | The `sprixel` command: bridge, setup, doctor and tools from the terminal | 10 |
| 13 | [`install-and-release`](../openspec/changes/install-and-release/) | Install scripts, release pipeline, installers, versioning and updates | 1, 12 |

The numbers are the names of the specs. In the demo, `export-spritesheet` (9)
is built after the AI path (10 to 12); its `export` tool is added then.

## Milestones

| Milestone | Specs | Result |
| --- | --- | --- |
| **M1 · Foundation** (done) | 1, 2, 3 | An empty app that opens and saves a project, with the theme and language working |
| **M2 · Demo** | the demo cut of 4 to 12 | A real pixel art editor where an AI draws and animates sprites while you watch, and you can export them |
| **M3 · Version 1** | the rest of 4 to 12, and 13 | GIF export, search and multi-select, install with one command and publish `1.0.0` |

Suggested versions: `0.1.0` at the end of M1, `0.2.0` for the demo, and `1.0.0`
after M3 when everything is tested. The demo lives in `dev`; `main` only
receives a release pull request.

## How to continue

Where we are: M1 is done and the plan for M2 (the demo) is written. The demo
has not started: the next step is **phase 1, `project-browser`**.

### Build the demo (M2)

1. Do the phases in the table above, **one at a time and in order**. For each
   phase: create `feature/<spec-name>` from `dev`, do the tasks under "Demo" in
   its `tasks.md` (one commit per task, tests first), open a pull request into
   `dev`, wait for the three CI jobs, audit and merge.
2. Do not archive a spec that still has tasks under "After the demo". Archive
   it when version 1 is done.
3. Who writes what: a Claude agent for interface, design and security; OpenCode
   for pure code with an exact spec and tests (the Rust operations, the grid
   parser, the Go command, the renderer). The rules for agents, the checklist to
   audit them and the dependency rules are in [AGENTS.md](../AGENTS.md).
4. The tasks that need a decision first: the MCP dependencies are approved in
   the design of `mcp-server-and-tools` (`tiny_http`, `getrandom`, `png`); the
   two preset palettes ("Pocket RPG 16" and "Farm 32") are designed in phase 4.
5. Phase 10 (demo polish): write `docs/demo.md` with five prompts to try, add an
   example project, run an end to end test with a scripted MCP client and one
   real run with Claude Code, and update the `CHANGELOG.md`.

### From the demo to the first complete version (M3, `1.0.0`)

1. Finish the tasks under "After the demo" in each spec: search, move, folder
   delete, multi-select and the context menu (`project-browser`); lost
   connection (`mcp-server-and-tools`, `agents-ui`); GIF, trim, pivot and
   progress (`export-spritesheet`); and the tests on Windows and macOS
   (`cli-sprixel`). Then archive those specs.
2. Build `install-and-release` (spec 13): the version bump script, `install.sh`
   and `install.ps1`, `sprixel update` and `uninstall`, and the release pipeline.
   Release builds must ship `LICENSE` and `docs/third-party.md`.
3. Check by hand on Windows and macOS (the CI only builds and tests there),
   including the system menu bar on macOS, and the real window on each system.
4. Check the licenses again (see `docs/third-party.md`).
5. Bump the version (`0.2.0` for the demo, `1.0.0` for the first complete
   version), open the release pull request `dev` to `main`, tag it, and publish.

The **demo can be shown before `1.0.0`**: `main` is the branch GitHub shows, so
a release pull request from `dev` to `main` is what puts the new README in front
of people. That is the maintainer's decision.

## How a spec is built

1. Read the spec (`proposal.md`, `design.md`, `specs/`, `tasks.md`).
2. Create `feature/<spec-name>` from `dev`.
3. Do the tasks one by one, with tests, one commit per task.
4. Open a pull request into `dev`.
5. Archive the spec when it is merged and all its tasks are done.

See [CONTRIBUTING.md](../CONTRIBUTING.md).
