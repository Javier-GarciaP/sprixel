# Spec Delta

## Purpose

Defines the MCP server that lets AI tools control the running app, and the
protections that keep it safe on a personal computer.

## ADDED Requirements

### Requirement: Local only
The server SHALL start with the app, listen only on `127.0.0.1` on a free port chosen at start, stop with the app, and write the port and process id to `runtime.json` in the Sprixel data folder.

#### Scenario: Not reachable from the network
- **WHEN** another computer in the network tries to connect to the server port
- **THEN** the connection is refused because the server is not listening on the network

### Requirement: Token
Every request SHALL carry a bearer token, the token MUST be random with at least 128 bits, stored in a file that only the current user can read, and a request without a valid token MUST be answered with an authorization error and no data.

#### Scenario: Missing token
- **WHEN** a request without a token is sent
- **THEN** the server answers "unauthorized" and does not run any tool

### Requirement: Browser protection
The server SHALL reject any request that has an `Origin` header and any request whose `Host` header is not the local address and port, so that web pages cannot control the app.

#### Scenario: Request from a web page
- **WHEN** a web page sends a request to the server, with an `Origin` header
- **THEN** the request is rejected even if it carries a valid token

### Requirement: Approval of new clients
The first time an AI client connects, the app SHALL ask the user to allow or reject it, wait for the answer, and remember allowed clients; a rejected or unanswered client MUST NOT run any tool, and the user MUST be able to remove an allowed client.

#### Scenario: New client
- **WHEN** a client named "Claude Code" connects for the first time
- **THEN** the app shows "Claude Code wants to connect" with Allow and Reject, and tools stay blocked until the user answers

#### Scenario: Answer later
- **WHEN** the app is in the background
- **THEN** a system notification asks the user to open the app and answer

### Requirement: Permissions
The server SHALL enforce, in the app core, the permissions Create sprites, Edit pixels, Delete sprites and Export files, apply them to every client, and answer a denied tool call with the error `permission_denied` and a hint about the setting.

#### Scenario: Delete is off
- **WHEN** an agent calls `delete_sprite` and the delete permission is off
- **THEN** the sprite is not deleted and the error `permission_denied` is returned

### Requirement: Limits
The server SHALL limit the size of a request (4 MB), the number of requests per second per client, and the number of clients connected at the same time, and answer with the error `limit_exceeded` when a limit is passed.

#### Scenario: Huge request
- **WHEN** a request larger than 4 MB is sent
- **THEN** it is rejected with `limit_exceeded` and the app keeps working

### Requirement: Private logs
Logs and error messages MUST NOT contain the token.

#### Scenario: A failed request
- **WHEN** a request fails authorization
- **THEN** the log has the reason but not the token that was sent

### Requirement: Standard protocol
The server SHALL implement the Model Context Protocol so that any MCP client can list the tools and call them, including their input descriptions.

#### Scenario: List tools
- **WHEN** a client asks for the list of tools
- **THEN** it receives every tool with its name, description and input schema
