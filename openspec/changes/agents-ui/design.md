# Design

## Context

See proposal.md and the screens 28 to 33 in `docs/design/`. Events come from
the core: connection changes, approval requests, activity entries and busy
markers.

## Goals / Non-Goals

**Goals:**
- A person who never read the docs can connect an AI in under a minute.
- The state of every agent is clear at a glance.

**Non-Goals:**
- Chat with the agent inside the app. The AI is used from its own tool.
- Agents other than Claude Code and OpenCode in the interface.

## Decisions

- **The UI only renders core events.** No agent state is kept in the UI.
- **The commands in the dialog are generated** from the client id, so they stay
  in sync with the CLI (`sprixel mcp add <client>`).
- **The approval prompt is a global dialog** mounted by the app shell, so it
  appears on any tab.
- **System notification** through the Tauri notification support when the
  window is not focused.

## Risks / Trade-offs

- [The user closes the approval dialog by accident] → the pending request stays
  and can be answered from the Agents tab.
- [The CLI is not installed when the user opens the dialog] → the dialog shows
  how to install it with a link to the README.
