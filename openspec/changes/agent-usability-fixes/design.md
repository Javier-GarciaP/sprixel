# Design

## Context

See proposal.md. Facts found in the code: the app already handles `DELETE` of
a session (`mcp/http.rs`); `SPRIXEL_APP` already exists in the launcher; the
bridge runs each message in a goroutine; "Sprixel CLI" is a separate client
with its own approval (key is name and version).

## Goals / Non-Goals

**Goals:**
- An agent can work a full session without hitting limits or disorder.
- The person is never left with a stuck dialog.
- Claude Code knows how to use the app from the first prompt.

**Non-Goals:**
- Auto-approving "Sprixel CLI". It stays a separate client.
- Design quality and live movement (spec `agent-live-design`).

## Decisions

- **Close with `DELETE`.** `appclient.Client.Close` sends `DELETE` with the
  session id; commands call it with `defer`.
- **One worker for `tools/call`.** A channel feeds one goroutine, so the order
  holds and the reader still sees cancellations and pings.
- **The UI refreshes the clients after any answer** and shows the error. The
  core emits the clients event also when the entry is gone.
- **`open_project` has its own permission** (`Need::OpenProject`), off by
  default, because it can open any folder. It reuses the app's open code and
  emits an event the shell handles, so tabs and recent list update.
- **The skill is embedded** in the CLI with `go:embed`. `mcp add claude` writes
  it even when already registered. This changes the old rule "the CLI does not
  edit Claude's files": it now writes only its own folder `skills/sprixel/`.
- **`sprixel start` reads the version** from `runtime.json` after start.
  Whether the app reads a project path argument is checked in task 6.2; if not,
  the command says the path is ignored.

## Risks / Trade-offs

- A slow tool blocks the next call in the bridge. Accepted: order matters more.
- `open_project` on any folder is powerful. It is off by default and logged.
