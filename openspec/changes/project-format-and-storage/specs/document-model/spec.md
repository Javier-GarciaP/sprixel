# Spec Delta

## Purpose

Defines what a project contains, the rules that keep it valid, and how every
change is applied and reverted, whether it comes from a person or an AI.

## ADDED Requirements

### Requirement: Project structure
A project SHALL contain folders (which can hold other folders), sprites and palettes. Every sprite MUST belong to one folder or to the project root and MUST use exactly one palette of the same project.

#### Scenario: A sprite uses a palette
- **WHEN** a sprite is created in a project that has the palette "dungeon16"
- **THEN** the sprite references "dungeon16" and cannot reference a palette that does not exist

### Requirement: Sprite rules
A sprite SHALL have a width and height between 1 and 512 pixels, at least one frame and at least one layer. Every frame MUST have the same size as the sprite and every pixel MUST be transparent or the key of a palette color.

#### Scenario: Size limits
- **WHEN** a sprite of 600x20 is requested
- **THEN** the request is rejected with the error `too_big`

#### Scenario: A pixel with an unknown key
- **WHEN** an operation sets a pixel to a key that is not in the sprite's palette
- **THEN** the operation is rejected and the sprite does not change

### Requirement: Names
Sprite and folder names MUST be unique inside their folder, ignoring upper and lower case, MUST NOT be empty, and MUST NOT contain characters that are invalid in file names on Linux, Windows or macOS.

#### Scenario: Duplicate name
- **WHEN** a sprite named "Goblin" is created in a folder that has "goblin"
- **THEN** the operation fails with the error `name_exists`

### Requirement: Animation tags
A sprite SHALL have zero or more animation tags. A tag MUST have a name that is unique in the sprite, a start and end frame inside the sprite (start not after end), a speed from 1 to 60 fps and a mode of `loop`, `once` or `pingpong`.

#### Scenario: Range outside the frames
- **WHEN** a tag with frames 3 to 8 is created on a sprite with 4 frames
- **THEN** the operation is rejected with the error `bad_range`

#### Scenario: Frames are removed
- **WHEN** a frame inside a tag range is deleted
- **THEN** the tag range shrinks to match, and a tag with no frames left is removed

### Requirement: Operations and revisions
Every change SHALL be applied as an operation that records its author (`user` or the name of an AI client). The project MUST have a revision number that increases by one with every applied operation or group of operations.

#### Scenario: Revision grows
- **WHEN** two operations are applied one after the other
- **THEN** the revision increases by exactly 2 and each operation keeps its author

#### Scenario: A failed operation
- **WHEN** an operation is rejected
- **THEN** the project and its revision do not change

### Requirement: Grouped actions
An action that has several steps, such as one pencil stroke or one AI tool call, SHALL be recorded as one history entry, applied all together or not at all.

#### Scenario: Atomic AI call
- **WHEN** an AI tool call changes 40 pixels and the 41st change is invalid
- **THEN** none of the 41 changes are applied

### Requirement: Undo and redo
The history SHALL support undo and redo of the most recent entries, in order, for any author. Applying a new entry after an undo MUST clear the redo entries.

#### Scenario: Undo an AI action
- **WHEN** the latest history entry was made by an AI and the user presses undo
- **THEN** the project returns to the state before that action and the entry is available to redo

### Requirement: Undo the last AI action
The system SHALL offer to undo the most recent AI action, but only when it is the latest entry in the history. If the user has changed something after it, the request MUST fail with the error `newer_changes` and nothing is undone.

#### Scenario: A newer user change exists
- **WHEN** the user draws after an AI action and then requests "undo last AI action"
- **THEN** the request fails with `newer_changes`
