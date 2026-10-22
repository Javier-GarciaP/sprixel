# Proposal

## Why

Sprixel must install with one command and be released in a repeatable way, so
that every version is built, tested, versioned and published without manual
steps.

## What Changes

- Write `install.sh` (Linux and macOS) and `install.ps1` (Windows) that install
  the app and the `sprixel` command from GitHub Releases.
- Add `sprixel update` and `sprixel uninstall`.
- Add the version bump script that updates every version file.
- Add the release pipeline: on a version tag, build the app bundles and the
  command for every system, create checksums and publish a GitHub Release with
  the changelog notes.
- Document the release steps that match `CONTRIBUTING.md`.

## Capabilities

### New Capabilities
- `installation`: installing, updating and uninstalling Sprixel from the terminal.
- `release-management`: version numbers, tags, the release pipeline and its artifacts.

### Modified Capabilities

## Impact

- New `scripts/` files and `.github/workflows/release.yml`.
- Needs the app and the command to build on every system (`project-foundation`,
  `cli-sprixel`).
- Code signing and notarization are not part of version 1; the first run may
  show system warnings and the docs must say so.
