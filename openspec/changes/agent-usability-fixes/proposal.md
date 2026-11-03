# Proposal

## Why

The first test with a real agent (Claude Code) drew a chicken, but it exposed
problems: the `sprixel tool` command never closes its session, so the app
reaches its limit of 16 clients; the bridge runs tool calls in parallel, so
steps arrive out of order; the agent cannot open a project; the approval
dialog seems stuck; Claude did not know how to use the app; and `sprixel start`
opened an old build without saying so.

## What Changes

- Close the session when `sprixel tool`, `tools` and `doctor` finish.
- Run the tool calls of the stdio bridge one after another, in order.
- Close the approval dialog after any answer, and show the error when the
  answer fails.
- Add the tools `list_projects` and `open_project`, with a new permission
  "Open projects" that is off by default.
- `sprixel mcp add claude` also installs a skill that teaches Claude Code how to
  use Sprixel; remove and uninstall delete it.
- `sprixel start` prints the app path and version and warns when the version
  differs from the CLI; `SPRIXEL_APP` is documented.

## Capabilities

### New Capabilities
- `agent-connection`: sessions, call order, approval answers and the launcher.
- `agent-projects`: list and open projects from an agent, with its permission.
- `agent-skill`: the Claude Code skill installed by the CLI.

### Modified Capabilities

## Impact

- Go: `cli/internal/{appclient,bridge,clients,command,launcher,doctor}`.
- Rust: `src-tauri/src/mcp/` (new `tools/projects.rs`, `policy.rs`, `host.rs`,
  `mod.rs`, `testing.rs`).
- UI: `src/features/agents` (approval dialog, permissions panel), `src/app/App.tsx`,
  i18n files.
- Docs: `docs/ai-and-mcp.md`, `docs/roadmap.md`.
- No new dependency.
