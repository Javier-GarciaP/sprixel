# Design

## Context

See proposal.md and the release steps in `CONTRIBUTING.md`. Artifacts are
built by GitHub Actions on runners for each system. The first release will be
unsigned.

## Goals / Non-Goals

**Goals:**
- A release made only by pushing a tag after the release pull request.
- Installers that are small, checked and reversible.

**Non-Goals:**
- Code signing and notarization in version 1.
- Package managers (Homebrew, winget, apt) in version 1.
- Automatic update in the background.

## Decisions

- **Install scripts live in the repository** and are published as release
  assets, so `releases/latest/download/install.sh` always matches the latest
  release.
- **Tauri bundles** per system (AppImage on Linux, `.dmg` or `.app` archive on
  macOS, installer on Windows) plus the Go command built with `go build` for
  each target.
- **One version file at the repository root** read by the bump script and by
  the build.
- **Checksums with SHA-256** in `checksums.txt`, verified by the scripts.
- **Notes from the changelog**: the pipeline extracts the section of the tag.
- **Smoke test** on a clean runner per system.

## Risks / Trade-offs

- [Unsigned apps show warnings (Gatekeeper, SmartScreen)] → document the steps
  in the README and plan signing after version 1.0.0.
- [`curl | sh` is a risk] → scripts are short, readable and verify checksums;
  show the MIT license and the source URL before installing.
- [The pipeline is slow on three systems] → cache dependencies and run jobs in
  parallel.
- [A broken release] → releases can be marked as pre-release and a fixed
  `PATCH` version is published as described in `CONTRIBUTING.md`.
