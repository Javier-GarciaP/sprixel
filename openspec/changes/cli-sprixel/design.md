# Design

## Context

See proposal.md and `docs/architecture.md`. The command is a separate Go
program. The app writes `runtime.json` and the `token` file; the command reads
them.

## Goals / Non-Goals

**Goals:**
- One small static binary per system.
- Setup that works in one command and can be undone.

**Non-Goals:**
- Running the app headless.
- Support for clients other than Claude Code and OpenCode in `mcp add`.

## Decisions

- **Go with the standard library first.** Use an argument parsing library only
  if it is clearly needed (decided and approved when added).
- **MCP bridge as a line-based JSON-RPC proxy** over stdio to the HTTP endpoint.
  The messages are not interpreted, except the initialize call that adds the
  client name.
- **Register Claude Code by calling its own CLI**
  (`claude mcp add sprixel --scope user -- sprixel mcp`) instead of editing its
  files. Register OpenCode by editing its JSON configuration safely (read,
  merge, write atomically). The exact config shape is verified against the
  OpenCode documentation in the first task.
- **Start the app** with a platform specific launcher and poll `runtime.json`.
- **Exit codes** are documented and tested.

## Risks / Trade-offs

- [AI tools change their config format] → keep each client in its own package
  with tests and fixtures, and make `doctor` detect a mismatch.
- [Starting the app from a background process fails on some systems] → clear
  error and `sprixel start` as a fallback.
- [Stdout pollution breaks MCP] → a test that fails if anything other than
  protocol messages is written to stdout.
