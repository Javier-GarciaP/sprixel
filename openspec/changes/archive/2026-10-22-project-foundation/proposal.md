# Proposal

## Why

Nothing exists yet. Every other spec needs a working skeleton that builds on
Linux, Windows and macOS, and the biggest technical risk (drawing speed on a
large canvas inside the system web view) must be measured before the interface
is built on top of it.

## What Changes

- Create the app skeleton: Tauri 2 (Rust core) with a React, TypeScript, Vite
  and Tailwind interface, using the folder layout from `docs/architecture.md`.
- Create the Go module for the `sprixel` command with a `version` command only.
- Add one place for the version number, shared by the app and the command.
- Add the shared tooling: formatting, linting (with import boundaries between
  features), unit test runners for the three languages.
- Add a continuous integration workflow that lints, tests and builds on the
  three systems for every pull request.
- Build a small drawing benchmark for a 512x512 sprite and record the result.
- Document the commands to run, test and build.

## Capabilities

### New Capabilities
- `app-platform`: the app runs on Linux, Windows and macOS, builds the same
  way everywhere, has one version number, is checked automatically and meets a
  drawing speed budget.

### Modified Capabilities

## Impact

- Creates `src/`, `src-tauri/`, `cli/`, `scripts/` and `.github/workflows/`.
- Adds the first dependencies (Tauri, React, Vite, Tailwind, Vitest, ESLint,
  Prettier). Each one must be approved before it is added.
- Requires a Rust toolchain and the system web view libraries on the developer
  machine.
- Updates `AGENTS.md` with the real commands.
