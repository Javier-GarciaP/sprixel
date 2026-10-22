# Design

## Context

See proposal.md. The developer machine (Fedora) has Node and Go but no Rust
toolchain. Tauri uses the system web view, which is WebKitGTK on Linux. This is
a greenfield repository and the layout is already defined in
`docs/architecture.md`.

## Goals / Non-Goals

**Goals:**
- A skeleton that builds and is checked on the three systems.
- An early, honest measurement of drawing speed.
- Rules that keep the folder structure from drifting.

**Non-Goals:**
- Any real feature (projects, drawing, MCP). Those are other specs.
- Code signing and installers. Those are in `install-and-release`.

## Decisions

- **Tauri 2 + React + Vite + TypeScript + Tailwind.** Chosen in the project
  plan. Alternative considered: Electron (bigger, ships its own Chromium, which
  would avoid WebKitGTK problems). Kept as a fallback if the benchmark fails.
- **npm as the package manager.** It is already installed, so no extra tool is
  added. Alternative: pnpm (faster, but one more tool).
- **Vitest, `cargo test` and `go test`.** One runner per language, each next to
  its code.
- **ESLint with import-boundary rules and Prettier.** Enforces the feature
  isolation rules in the specs instead of relying on review.
- **GitHub Actions matrix** (ubuntu, windows, macos) for lint, test and build.
- **Benchmark first.** A single `<canvas>` with an `ImageData` buffer of
  palette indexes, painted outside React state. The result decides whether the
  rendering approach in `docs/architecture.md` is kept.
- **Version source of truth.** A single file at the repository root read by the
  bump script; the app and the Go command get the version at build time.

## Risks / Trade-offs

- [WebKitGTK is slow with big canvases] → measure in the first tasks; fall back
  to a WebGL canvas, and as a last resort to Electron.
- [Rust is new to the developer machine] → install it with `rustup` and
  document the exact steps in the contributing docs.
- [Three operating systems triple the CI time] → cache dependencies and run the
  matrix in parallel.
