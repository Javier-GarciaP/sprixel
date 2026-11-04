# Spec Delta

## Purpose

Defines the screens through which a person connects AI agents, watches what
they do, chooses what they may do, and stops or undoes them.

## ADDED Requirements

### Requirement: Agents tab
The Agents tab SHALL show one card for Claude Code and one for OpenCode with their state (connected, not connected, connection lost), the actions "Connect", "View command" and "Disconnect", the activity list and the permissions panel when at least one agent is connected.

#### Scenario: Two agents connected
- **WHEN** both agents are connected
- **THEN** both cards show "Connected" with "View command" and "Disconnect", and the permissions panel is visible

#### Scenario: Nobody connected
- **WHEN** no agent has connected
- **THEN** both cards offer "Connect", the activity area explains that it will show what agents draw, and the permissions panel is hidden

### Requirement: Connect dialog
Pressing "Connect" SHALL open a dialog with a tab per client that shows the command to copy (`sprixel mcp add claude` or `sprixel mcp add opencode`), a Copy button that does not cover the text, the next steps, and a waiting state that turns into "Connected" when the agent connects.

#### Scenario: Copy the command
- **WHEN** the user presses Copy on the Claude Code tab
- **THEN** `sprixel mcp add claude` is on the clipboard and the dialog keeps waiting for the connection

#### Scenario: It takes too long
- **WHEN** no connection happens after 60 seconds
- **THEN** the dialog shows a help message that suggests running `sprixel doctor`

### Requirement: Dialog size
The Connect dialog SHALL keep the same position and size when switching tabs.

#### Scenario: Switch tabs
- **WHEN** the user switches from Claude Code to OpenCode
- **THEN** the dialog does not move or change size

### Requirement: Approval prompt
When a new client connects, the app SHALL show a prompt from any tab with the client name, the kind of connection, the permissions it would have and the buttons Allow and Reject, and the top bar MUST show "Waiting for approval" until the user answers.

#### Scenario: Allow
- **WHEN** the user presses Allow
- **THEN** the card shows "Connected" and the top bar shows the agents as connected

### Requirement: Top bar indicator
The top bar SHALL show the state of the agents as "No agents", the names of the connected agents, "Waiting for approval" or "<agent> disconnected", with a matching color.

#### Scenario: Lost connection
- **WHEN** a connected agent is lost
- **THEN** the indicator turns red with "Claude Code disconnected" and its card shows "Connection lost" with "Reconnect"

### Requirement: Activity list
The activity list SHALL show for each action the thumbnail of the sprite, the description, the client and the time, mark the action in progress, offer "Stop" on it, and offer "Undo" only on the latest finished action.

#### Scenario: In progress
- **WHEN** an agent is drawing frame 2
- **THEN** the first row is highlighted with "Stop"

### Requirement: Permissions panel
The permissions panel SHALL show switches for Create sprites, Edit pixels, Delete sprites, Export files and "Ask me when a new agent connects", apply a change at once and say that AI changes can be undone except exported files.

#### Scenario: Turn delete on
- **WHEN** the user turns on "Delete sprites"
- **THEN** the next `delete_sprite` call from an agent is allowed

### Requirement: Editor tag
When an agent is drawing in the open sprite, the editor SHALL show an amber tag with the agent name and the frame, and a Stop button, and the tag MUST disappear when the work ends.

#### Scenario: Stop from the editor
- **WHEN** the user presses Stop in the tag
- **THEN** the agent stops and the tag disappears
