# Proposal

## Why

The main idea of Sprixel is that an AI can use the app. That needs a server the
AI tools can talk to, a set of tools that are exact and safe, and a way for the
person to see, stop and undo what the AI does.

## What Changes

- Add an MCP server inside the app, reachable only from the same computer,
  protected by a token, host checks and approval of new clients.
- Add the tools: read the project, create sprites, write text grids and
  pixels, draw shapes, manage frames, animations and palette colors, export,
  and undo the last AI action.
- Enforce permissions inside the core (create, edit, delete, export).
- Record the activity of every agent, show who is drawing, let the user stop an
  agent and undo its last action, and detect a lost connection.
- Handle concurrent edits with revisions and busy frames.

## Capabilities

### New Capabilities
- `mcp-server`: the server, its transport, authentication, approval, permissions and limits.
- `mcp-tools`: the catalog of tools and the exact behavior and errors of each.
- `agent-activity`: activity log, attribution, stopping, undoing, busy frames and connection state.

### Modified Capabilities

## Impact

- New Rust modules `mcp` (server, tools, auth, permissions, activity) in
  `src-tauri/`.
- Adds an MCP library or a small hand-written JSON-RPC layer (decided in the
  design) and a random token file in `~/.sprixel/`.
- The `cli-sprixel` change connects to this server; `agents-ui` shows it.
- Security review is needed before the first release.
