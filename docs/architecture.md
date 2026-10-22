# Architecture

> **Status: foundation built.** The app skeleton (Tauri 2 + Vite + React +
> TypeScript + Tailwind), the Go CLI, the tooling, CI and the drawing
> benchmark exist. The editor features, the Rust core modules and the MCP
> server are still planned and follow the version 1 specs in
> [`openspec/`](../openspec/). If a spec and this document disagree, the spec
> wins.

## Overview

Sprixel has three parts:

| Part | Written in | What it does |
| --- | --- | --- |
| **Sprixel app** | Tauri 2: Rust core + React and TypeScript UI | The desktop editor. The Rust core owns all the data. The UI is a view of it. |
| **`sprixel` command** | Go | Installs and checks things, registers the MCP in AI tools, and bridges AI tools to the app. |
| **Project folder** | Plain text files | Where your sprites, palettes and settings are saved. |

```
 Claude Code / OpenCode
        │  MCP over stdio
        ▼
 sprixel mcp  (Go)  ──┐                          ┌────────────────────────────────────┐
                      │  HTTP + token             │  Sprixel app (Tauri)                │
 sprixel <command> ───┴─────────────────────────▶ │                                    │
                         127.0.0.1 only            │  Rust core                          │
                                                   │   ├─ document + operations         │
 Any MCP client ───────── HTTP + token ──────────▶ │   ├─ history (undo / redo)         │
 (optional, direct)                                │   ├─ MCP server                    │
                                                   │   └─ project files, export         │
                                                   │        ▲ commands / events ▼       │
                                                   │  React UI (view only)              │
                                                   └──────────────┬─────────────────────┘
                                                                  ▼
                                                        Project folder (text files)
```

The AI never edits files directly. It sends operations to the running app, so
you see the canvas change live and every AI action can be stopped or undone.

## Key decisions

1. **One owner of the data.** The Rust core holds the document (sprites,
   frames, palette) and its history. The React UI never keeps a second copy as
   the truth. This avoids two sources of truth when an AI and a person edit at
   the same time.
2. **Every change is an operation.** Drawing a stroke, adding a frame and an AI
   tool call all become an `Operation` with an author (`user`, `claude-code`,
   `opencode`...), applied by the core. The core stores the inverse of each
   operation, so undo and redo work the same for everyone, and AI actions are
   labeled.
3. **Fast drawing.** While you draw a stroke, the UI paints on one `<canvas>`
   using a typed array of palette indexes, outside of React state. When the
   stroke ends, the UI sends one operation with only the changed pixels.
4. **The app is the MCP server.** The server lives inside the app, on
   `127.0.0.1` only. The `sprixel` command is a thin client and a stdio bridge,
   like a remote control. See [AI and MCP](ai-and-mcp.md).
5. **Plain files.** Projects are folders of text files, with one-letter palette
   keys for pixels, so they are readable, diff well in Git and can be read by
   an AI.
6. **Animation is a tag.** A sprite has frames. An animation is a named range
   of frames with a speed and a repeat mode.
7. **One layer in version 1.** The file format already stores layers, but the
   interface shows only one.

## The project on disk

A project is a folder. Example:

```
Dungeon Kit/
├── sprixel.json                  project manifest (name, formatVersion, active palette)
├── palettes/
│   └── dungeon16.palette.json    colors with name, one-letter key and hex
├── sprites/
│   ├── Personajes/
│   │   ├── goblin.sprite.json
│   │   └── heroe.sprite.json
│   └── Enemigos/
│       └── slime.sprite.json
├── assets/sprites/               default export folder
└── .sprixel/
    └── trash/                    deleted items (kept so Undo can bring them back)
```

A sprite file (shortened):

```json
{
  "formatVersion": 1,
  "width": 8,
  "height": 8,
  "palette": "dungeon16",
  "frameDurations": [83, 83, 83, 83],
  "layers": [
    {
      "name": "Layer 1",
      "frames": [
        ["....RRRR", "...RRRRR", "...NNNN.", "..NPNPPN", "..NPPPPN", "..NNNNNN", ".NAAAAN.", ".NA..AN."]
      ]
    }
  ],
  "tags": [{ "name": "walk", "from": 1, "to": 4, "fps": 12, "mode": "loop" }]
}
```

- Folders in `sprites/` are the folders you see in the app.
- Files are saved automatically and **atomically** (write a temporary file,
  then rename it), so a crash never leaves a broken file.
- If a file changes on disk while the app is open (for example after
  `git pull`), the app reloads it.
- Each file has a `formatVersion`. Opening an older project runs a migration.
- Deleted items go to `.sprixel/trash/` first.

The full format is defined in the `project-format-and-storage` spec.

## The `sprixel` command

Written in Go, as one small program for Linux, Windows and macOS.

| Command | What it does |
| --- | --- |
| `sprixel start [path]` | Starts the app, optionally opening a project |
| `sprixel status` | Shows whether the app is running |
| `sprixel mcp` | MCP server over stdio, used by AI tools. Starts the app if needed |
| `sprixel mcp add <client>` | Registers Sprixel in Claude Code or OpenCode |
| `sprixel mcp remove <client>` | Removes it |
| `sprixel tools` / `sprixel tool <name> [key=value...]` | Lists and calls tools from the terminal |
| `sprixel doctor` | Checks the install, the app, the token and the AI registrations |
| `sprixel update` | Updates to the latest release |
| `sprixel uninstall` | Removes Sprixel |
| `sprixel version` | Shows the version |

The app and the command share a small local folder, `~/.sprixel/`
(`%USERPROFILE%\.sprixel` on Windows):

| File | Content |
| --- | --- |
| `runtime.json` | Port, process id and version of the running app |
| `token` | The access token (readable only by you) |
| `clients.json` | The AI clients you approved |

## Security model

The MCP server can change your files, so it is closed by default.

- It listens on **`127.0.0.1` only**, on a random free port.
- Every request needs the **token** (`Authorization: Bearer ...`). The token is
  random, stored in a file only you can read, and the `sprixel` command reads
  it for you.
- The server checks the `Host` header and **rejects any request with an
  `Origin` header**, so a web page cannot talk to it (no CSRF or DNS rebinding).
- A **new AI client must be approved** by you the first time.
- **Permissions are enforced in the core**, not only in the interface: create
  sprites, edit pixels, delete sprites, export files.
- Export and file paths are **limited to the project folder**.
- Requests have a size limit.

More detail in [AI and MCP](ai-and-mcp.md).

## Concurrency: you and the AI at the same time

- Operations are small and atomic. They are applied one at a time by the core.
- The frame an agent is writing to is marked as busy in the interface. Your
  strokes on it wait until the operation finishes.
- Write tools accept an optional `base_revision`. If the document changed since
  then, the tool fails with `stale_revision` and the AI must read again.
- The **Stop** button cancels the agent's work in progress.
- **Undo** reverts the last action, whoever made it. The tool
  `undo_last_ai_action` reverts the last AI action. Older actions are not
  undone one by one.

## Technology

| Area | Choice |
| --- | --- |
| Desktop shell | Tauri 2 |
| Core | Rust |
| UI | React, TypeScript, Vite, Tailwind CSS |
| CLI | Go |
| Docs and specs | Markdown, OpenSpec |
| Fonts | Space Grotesk, Inter, DM Mono |
| Platforms | Linux, Windows, macOS |

Tauri uses the system web view (WebKitGTK on Linux). Pixel canvas performance
on large sprites (512x512) is checked early in the `project-foundation` spec.

## Drawing performance

The benchmark (`npm run benchmark`) checks that drawing stays fast.

- **Method.** One 512x512 sprite held as a `Uint8Array` of palette indexes and
  one canvas. Only the changed area is repainted with `putImageData`. A stroke
  of 300 points with a round brush of radius 4, each point arriving at a random
  moment of the frame. Latency is measured until just after the next frame is
  painted.
- **Budget.** Median of 16 ms or less. The command exits with code 1 above it.

| Platform | Median | Result |
| --- | --- | --- |
| Linux (Fedora 44, WebKitGTK, 60 Hz) | 9.5 to 10.0 ms in 3 runs (p95 16 ms) | Meets the budget |
| Windows | pending | |
| macOS | pending | |

Windows and macOS numbers are recorded when the benchmark is run there.

## Repository layout

Following the project structure rules (organize by feature, not by file type).
Folders marked *(empty)* exist with only a `.gitkeep` file; their content
arrives with the specs that build them.

```
sprixel/
├── src/                        React UI (exists)
│   ├── app/                    main.tsx, App.tsx, index.css, benchmark/
│   ├── features/               8 feature folders (all empty for now)
│   │   ├── project-browser/    later: components/, hooks/, api/, types.ts,
│   │   ├── canvas-editor/      index.ts per feature
│   │   ├── frames-animations/
│   │   ├── palette-editor/
│   │   ├── text-view/
│   │   ├── export/
│   │   ├── agents/
│   │   └── settings/
│   ├── shared/                 components/, hooks/, lib/, types/ (empty)
│   └── assets/                 (empty)
├── src-tauri/                  Rust core (exists)
│   └── src/                    lib.rs, main.rs, plus the module folders:
│       ├── document/           model and operations (empty)
│       ├── history/            undo and redo (empty)
│       ├── storage/            project files, autosave, trash, file watching (empty)
│       ├── render/             PNG and GIF rendering (empty)
│       ├── export/             sprite sheet and JSON (empty)
│       ├── mcp/                server, tools, auth, permissions, activity (empty)
│       └── ipc/                Tauri commands and events (empty)
├── cli/                        Go module github.com/Javier-GarciaP/sprixel/cli
│   ├── cmd/sprixel/main.go     sprixel version (more commands come later)
│   └── internal/               command/, version/ (runtime, bridge, clients,
│                               doctor and update come with later specs)
├── scripts/                    build-cli.mjs, benchmark.mjs, version.test.js
│                               (install.sh and install.ps1 come with later specs)
├── docs/
├── openspec/
└── README.md  LICENSE  CONTRIBUTING.md  CHANGELOG.md  AGENTS.md
```

Rules:

- The alias `@/x` means `src/x` (TypeScript paths and the Vite alias).
- A feature never imports from another feature. It uses `shared/` or the other
  feature's `index.ts`, so a cross-feature import must be `@/features/<name>`.
  The ESLint rule `no-restricted-imports` enforces this.
- `shared/` never imports from `features/`.
- `cmd/sprixel/main.go` has almost no logic.
- Tests live next to the code they test (`*.test.ts` and `*.test.tsx`), plus
  `eslint.config.test.js` in the root and `scripts/version.test.js`.

CI (`.github/workflows/ci.yml`) runs `format:check`, `lint`, `test` and
`build:release` on Linux, Windows and macOS for pull requests into `dev` and
`main`. The package manager is npm.
