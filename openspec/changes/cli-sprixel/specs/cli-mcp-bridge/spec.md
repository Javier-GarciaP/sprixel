# Spec Delta

## Purpose

Defines how AI tools reach the app through the `sprixel mcp` command, so that
they can use it with one setup command and no manual token.

## ADDED Requirements

### Requirement: stdio bridge
`sprixel mcp` SHALL speak MCP over stdin and stdout, forward every message to the running app and every answer back, and send its own logs only to stderr so that the MCP stream is never corrupted.

#### Scenario: List tools through the bridge
- **WHEN** an AI tool starts `sprixel mcp` and asks for the tools
- **THEN** it receives the same tool list as the app server

### Requirement: Token from the file
The bridge SHALL read the port and the token from the shared data folder and use them on every request, and MUST NOT print the token.

#### Scenario: No token argument
- **WHEN** the bridge is started with no arguments
- **THEN** it connects to the app using the token file

### Requirement: Start the app
If the app is not running, the bridge SHALL start it, wait up to 20 seconds for the server, and then continue; if it cannot start the app or no screen is available, it MUST report a clear MCP error that says why.

#### Scenario: App not running
- **WHEN** an AI tool starts the bridge and the app is closed
- **THEN** the app starts and the first tool call works

#### Scenario: No desktop session
- **WHEN** the app cannot start because there is no display
- **THEN** the error says that Sprixel needs a desktop session

### Requirement: Close with the client
The bridge SHALL exit when its input closes, and MUST NOT stop the app.

#### Scenario: AI tool closes
- **WHEN** the AI tool closes the bridge input
- **THEN** the bridge exits and the app keeps running

### Requirement: Reconnect
If the app restarts while the bridge is running, the bridge SHALL reread the runtime file and reconnect, and tools that were running MUST fail with a clear error instead of hanging.

#### Scenario: App restarts
- **WHEN** the app is restarted during a session
- **THEN** the next tool call succeeds after reconnecting

### Requirement: Client identity
The bridge SHALL pass the name and version of the AI client to the app, so the app can show and remember it.

#### Scenario: Client name
- **WHEN** Claude Code starts the bridge
- **THEN** the app sees the client as "Claude Code"
