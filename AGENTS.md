# AGENTS.md

Guide for AI coding agents working in this repository. Read it before you
change anything.

## What this project is

**Sprixel** is a desktop pixel art editor for game sprites that an AI agent can
use. It has three parts: the **app** (Tauri 2: Rust core + React, TypeScript,
Vite and Tailwind UI), the **`sprixel` command** (Go) and **project folders**
(plain text files).

The project is proprietary (see `LICENSE`). Everything is written in English
(code, comments, commits, docs). The product UI is in Spanish and English.

## Status and commands

Specs 1 to 3 are built and archived: `project-foundation` (the app skeleton,
the Go CLI in `cli/`, the tooling, CI and the drawing benchmark),
`project-format-and-storage` (the document model, operations, undo and redo,
the project files, autosave and trash, in the Rust core) and
`app-shell-and-settings` (the window layout, themes, languages, shortcuts,
settings, and creating and opening projects). Only the `settings` feature
folder has code; the other feature folders are still empty placeholders. The
next specs follow `docs/roadmap.md`.

Commands run from the repository root. The package manager is **npm**; run
`npm ci` once first to install the JS dependencies.

```sh
npm run tauri dev      # the full app (opens the window; the first run
                       # compiles Rust and takes minutes)
npm run dev            # UI only in a browser at http://localhost:1420
                       # (no Rust core)
npm test               # Vitest (UI and scripts) + cargo test (Rust core)
                       # + go test (CLI); the exit code reflects the result
npm run lint           # ESLint, includes the import-boundary rules
npm run format         # Prettier write
npm run format:check   # Prettier check
npm run build          # type-check and build the UI only
npm run build:cli      # builds the sprixel command to bin/sprixel
                       # (bin/sprixel.exe on Windows), version from package.json
npm run build:release  # Tauri bundle for the current system (Linux: .deb,
                       # .rpm and .AppImage in src-tauri/target/release/bundle/)
                       # plus bin/sprixel
npm run benchmark      # 512x512 drawing benchmark in the real app window;
                       # prints the median latency; exit code 1 if above 16 ms
```

Single tests:

```sh
npx vitest run <path>                            # one UI or scripts test
cargo test --manifest-path src-tauri/Cargo.toml  # Rust core
go test -C cli ./...                             # CLI
```

**Version:** the single source is the `version` field of `package.json`.
`tauri.conf.json` reads it (`"../package.json"`), the Go command gets it at
build time with `-ldflags` (`scripts/build-cli.mjs`) and `Cargo.toml` must
have the same value (a test checks it).

Other commands:

```sh
openspec list                 # specs in progress
openspec validate --all --strict
op start --file docs/design/sprixel.op   # open the design in OpenPencil
```

## How to work here

- **Specs first.** Work on one spec at a time, in the order of
  `docs/roadmap.md`. Read its `proposal.md`, `design.md`, `specs/` and
  `tasks.md`, then do `tasks.md` one task at a time, one commit per task.
- Use the OpenSpec commands: `/opsx:propose`, `/opsx:apply`, `/opsx:archive`
  (Claude Code) or `/opsx-propose` and so on (OpenCode).
- Branches: work from `dev` in `feature/*` branches and open pull requests
  into `dev`. `main` only receives release pull requests. See
  `CONTRIBUTING.md` for commits, versions and releases.
- Never add a dependency without asking. Each one named in a spec is approved
  at the moment its task starts.
- If code and a spec disagree, stop and ask. The specs win over `docs/`, and
  `docs/` wins over the design images.

## Architecture (what is not obvious from the code)

- **The Rust core owns the data.** The React UI is a view. Never keep a second
  copy of the document in React state as the truth.
- **Every change is an operation** with an author (`user` or an AI client name),
  applied by the core, grouped into history entries (a stroke or an AI call is
  one entry). Undo, redo, revisions and AI attribution all depend on this.
- **Drawing is fast on purpose.** The canvas paints a typed array of palette
  indexes outside React state and sends one operation per stroke.
- **The app is the MCP server** (loopback only, token, Origin checks, client
  approval, permissions enforced in the core). The `sprixel` command is a thin
  client and a stdio bridge; it reads the token from `~/.sprixel/`.
- **Animation is a tag** on a sprite (name, from, to, fps, mode), not a
  separate file. Version 1 shows one layer; the file format stores layers.
- **Projects are folders of text files.** Pixels are rows of one-character
  palette keys. Files are written atomically. Deleting moves files to
  `.sprixel/trash/`.
- The folder layout and import rules are in `docs/architecture.md`. A feature
  never imports from another feature.

## Domain conventions

- **Palette keys:** one printable ASCII character, not a space and not `.`;
  case sensitive. `.` is always transparent. Maximum 93 colors.
- **Text grid keywords are English** (`palette`, `legend`, `frame`), even in the
  Spanish UI. The format is in `docs/text-grid-format.md`.
- **Sprite size** is chosen when the sprite is created (1 to 512 per side) and
  never changed in the editor.
- **All user-facing text goes through the translation files** (Spanish and
  English). Never hard-code UI text.
- **Names** of sprites and folders are unique per folder, ignoring case.
- **Colors and fonts** come from the design tokens in `docs/design/README.md`.
  The pixel canvas background is a neutral gray checkerboard in both themes.
- **Deleting one sprite does not ask**: it goes to the trash with an Undo
  message. Folders and groups ask for confirmation.

## Overrides of the global rules

- The very first commits of this repository were made directly on `main` and
  `dev` because the remote was empty. From then on, nothing is pushed directly
  to `main` or `dev`.
