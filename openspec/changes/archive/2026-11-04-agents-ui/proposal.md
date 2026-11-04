# Proposal

## Why

The server and the tools are invisible. People need to connect an AI easily,
see what it does and keep control, without reading documentation.

## What Changes

- Build the Agents tab: Claude Code and OpenCode cards, the activity list and
  the permissions panel.
- Build the "Connect an AI" dialog with the `sprixel mcp add` commands, a
  waiting state and a help message when it takes too long.
- Build the "Approve a connection" prompt and the waiting state in the top bar.
- Show the agent status in the top bar and the "is drawing" tag with Stop in
  the editor.
- Build the lost connection state.

## Capabilities

### New Capabilities
- `agents-ui`: the screens that connect, show, restrict, stop and undo AI agents.

### Modified Capabilities

## Impact

- New feature folder `src/features/agents/`.
- Uses the events and commands of `mcp-server-and-tools`.
- Changes the commands shown in the designs: the dialog shows
  `sprixel mcp add claude`, not a command with a token. See
  `docs/design/README.md`.
