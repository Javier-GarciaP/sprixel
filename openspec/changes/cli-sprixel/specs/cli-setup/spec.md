# Spec Delta

## Purpose

Defines the terminal commands that register Sprixel in AI tools, check that
everything works, start the app and call tools directly.

## ADDED Requirements

### Requirement: Register in an AI tool
`sprixel mcp add <client>` SHALL register `sprixel mcp` as an MCP server named "sprixel" for `claude` (Claude Code, user scope) or `opencode` (OpenCode configuration), be safe to run twice, and print what it changed and how to undo it.

#### Scenario: Add for Claude Code
- **WHEN** the user runs `sprixel mcp add claude`
- **THEN** Claude Code lists a "sprixel" MCP server that starts `sprixel mcp`

#### Scenario: Run twice
- **WHEN** the same command is run again
- **THEN** nothing is duplicated and the command says it was already registered

### Requirement: Remove and list
`sprixel mcp remove <client>` SHALL remove the registration it made, and `sprixel mcp list` MUST show for each supported client whether Sprixel is registered.

#### Scenario: Remove
- **WHEN** the user runs `sprixel mcp remove opencode`
- **THEN** the "sprixel" server is removed from the OpenCode configuration and other servers stay

### Requirement: Doctor
`sprixel doctor` SHALL check the installation, the version of the app and the command, whether the app runs, the token file and its permissions, the server answer, and the registration in each client, print one line per check with OK or a failure and how to fix it, and exit with a non-zero code if a check fails.

#### Scenario: Everything is fine
- **WHEN** all checks pass
- **THEN** the output lists them with OK and the exit code is 0

#### Scenario: Token permissions are too open
- **WHEN** the token file can be read by other users
- **THEN** the check fails and says how to fix it

### Requirement: Start and status
`sprixel start [path]` SHALL start the app (optionally opening a project) and return, and `sprixel status` MUST say whether the app is running, with its port and version.

#### Scenario: Status when closed
- **WHEN** the app is not running
- **THEN** `sprixel status` says so and exits with a non-zero code

### Requirement: Tools from the terminal
`sprixel tools` SHALL list the available tools, and `sprixel tool <name> key=value ...` SHALL call one and print the result, using the same approval and permissions as any AI client.

#### Scenario: Call a tool
- **WHEN** the user runs `sprixel tool list_sprites`
- **THEN** the sprites are printed

### Requirement: Output and codes
Every command SHALL support `--json` for machine-readable output, use exit code 0 for success and distinct non-zero codes for usage errors, app not running and tool errors, and show help with `--help`.

#### Scenario: JSON output
- **WHEN** a command is run with `--json`
- **THEN** the output is valid JSON only

### Requirement: Same behavior on every system
All commands SHALL work the same on Linux, Windows and macOS, using the system user folders and paths, and never require administrator rights.

#### Scenario: Windows
- **WHEN** `sprixel mcp add claude` is run in PowerShell on Windows
- **THEN** it registers the server with the correct Windows command
