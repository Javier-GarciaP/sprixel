# Design

## Context

See proposal.md, `docs/ai-and-mcp.md` and `docs/architecture.md`. The core owns
the document and all changes are operations with authors, so the server maps
tool calls to operations.

## Goals / Non-Goals

**Goals:**
- A safe local server that works with Claude Code and OpenCode through the
  `sprixel` command, and with any MCP client.
- Tools that are exact, atomic and explain their errors.
- Clear control for the person: approve, restrict, stop, undo.

**Non-Goals:**
- Access from other computers, user accounts or a hosted service.
- A headless mode.

## Decisions

- **HTTP on loopback with a bearer token** (the transport the app can serve
  directly). The `sprixel mcp` command turns stdio into this. Alternative:
  only stdio (the app would need to be a child process of the AI tool, which
  breaks "watch the live app").
- **Token file** `~/.sprixel/token`, created on first run with mode `0600` (or
  a user-only ACL on Windows). Rotated by deleting it.
- **Origin and Host checks** on every request. No CORS headers are ever sent.
- **Client identity is a label.** Approval is remembered by the client name and
  the declared version; the token is what protects, and this is documented.
- **MCP implementation:** evaluate the official Rust SDK against a small
  JSON-RPC layer; choose the smaller one that supports tool schemas and image
  results. The dependency must be approved.
- **Permissions in one place** (a policy checked before a tool runs).
- **Writes through the core** as grouped operations, so undo, revisions and
  attribution come for free.
- **Cancellation** with a flag checked between steps of long tools; a tool
  call is applied entirely or not at all.
- **Connection state** from a request heartbeat and the stdio bridge closing.

## Risks / Trade-offs

- [A local process with the token can do everything allowed] → permissions,
  approval, Origin checks, project-only paths and the activity log.
- [A label can be faked by another local program] → documented limit; the
  token file is the real boundary.
- [Large `render_preview` images] → limit scale and size.
- [Rate limits annoy fast agents] → generous defaults, configurable later.

## Open Questions

- Exact numeric limits (requests per second, clients) can be tuned after the
  first real sessions.
