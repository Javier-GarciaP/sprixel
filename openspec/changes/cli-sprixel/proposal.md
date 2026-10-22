# Proposal

## Why

AI tools start MCP servers as commands, and people want to set everything up
from the terminal. The `sprixel` command connects those two worlds and gives
quick tools for checking and controlling the app.

## What Changes

- Build `sprixel mcp`: an MCP server over stdio that forwards to the running
  app and starts the app when needed.
- Build `sprixel mcp add|remove|list` for Claude Code and OpenCode.
- Build `sprixel doctor`, `sprixel start`, `sprixel status`, `sprixel tools`
  and `sprixel tool <name> key=value ...`.
- Share the local data folder, the runtime file and the token with the app.
- Work on Linux, Windows and macOS with clear exit codes and a `--json` output.

## Capabilities

### New Capabilities
- `cli-mcp-bridge`: the stdio bridge between AI tools and the app.
- `cli-setup`: the commands that register, check, start and call the app from the terminal.

### Modified Capabilities

## Impact

- Fills the Go module in `cli/` (packages `runtime`, `bridge`, `clients`,
  `doctor`).
- Depends on the server and token from `mcp-server-and-tools`.
- The install scripts and releases from `install-and-release` ship this command.
