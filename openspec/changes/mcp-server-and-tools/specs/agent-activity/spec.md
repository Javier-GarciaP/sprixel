# Spec Delta

## Purpose

Defines how the person sees, controls and reverses what AI agents do, and how
the app behaves when the person and an agent work at the same time.

## ADDED Requirements

### Requirement: Activity log
The app SHALL record every action of every client with the time, the client name, a short description, the sprite and a result, keep the log for the session, and show it newest first.

#### Scenario: An agent creates a sprite
- **WHEN** an agent creates the sprite "slime"
- **THEN** the log shows "OpenCode - Created the sprite slime" with the time

### Requirement: Attribution
Every operation applied by an agent SHALL carry the agent name as its author, and the "Last edited by" information of a sprite MUST show it.

#### Scenario: Last edited by
- **WHEN** an agent edits a sprite last
- **THEN** the details of the sprite show "Edited by" with that agent

### Requirement: Drawing state
While an agent is running a write call the app SHALL mark the sprite and frame as busy and show who is drawing, and when the call ends the marker MUST disappear.

#### Scenario: Busy frame
- **WHEN** an agent writes to frame 2
- **THEN** frame 2 is marked busy and shows the agent name until the write ends

### Requirement: Stop
The user SHALL be able to stop an agent from the editor and from the Agents tab, which cancels the work in progress, rejects the agent's further write calls with the error `stopped_by_user` until the user lets it continue, and leaves the project valid.

#### Scenario: Stop while drawing
- **WHEN** the user presses Stop while an agent draws
- **THEN** the current call ends, later calls fail with `stopped_by_user` and nothing is half applied

### Requirement: Undo from the activity
The activity list SHALL offer Undo only on the most recent AI action while it is still the latest history entry, and older rows MUST NOT offer Undo.

#### Scenario: An older row
- **WHEN** the user looks at an action that is not the latest
- **THEN** its row has no Undo button

### Requirement: Concurrent edits
When a person and an agent change the same frame, the core SHALL apply operations one at a time, the person's strokes on a busy frame MUST wait until the agent's call ends, and an operation based on an old revision MUST fail with `stale_revision`.

#### Scenario: User draws on a busy frame
- **WHEN** the user draws on a frame that an agent is writing
- **THEN** the stroke is applied right after the agent's call ends

### Requirement: Connection state
The app SHALL show the state of each agent as connected, waiting for approval, not connected or connection lost, detect a lost connection within 10 seconds, and stop the busy marker of a lost agent.

#### Scenario: The agent process dies
- **WHEN** an agent process is killed while drawing
- **THEN** within 10 seconds its card shows "Connection lost" and the busy marker is removed

### Requirement: Permission changes
Changing a permission SHALL apply to the next call of every agent without reconnecting.

#### Scenario: Turn export off
- **WHEN** the user turns Export off while an agent is connected
- **THEN** the next `export` call fails with `permission_denied`
