# Spec Delta

## Purpose

Defines how the editor shows an agent working, step by step.

## ADDED Requirements

### Requirement: Ordered ops event
After an agent write is applied, the core SHALL emit an event with the agent name, the sprite, the frame, the tool name and the changed pixels in drawing order (for a line or shape, along its path; for a grid, row by row; for `set_pixels`, as given).

#### Scenario: Line
- **WHEN** an agent draws a line
- **THEN** the event lists its pixels from the start point to the end point

### Requirement: Replay with a cursor
When the editor shows the sprite and frame of the event, it SHALL replay the pixels in order on the canvas with a cursor tagged with the agent name, and at the end the canvas MUST equal the stored frame.

#### Scenario: Grid
- **WHEN** an agent applies a grid on the open frame
- **THEN** the pixels appear row by row with the cursor moving, and the final canvas matches the file

#### Scenario: Other frame
- **WHEN** the event is for a sprite or frame that is not shown
- **THEN** nothing is replayed and the editor reloads normally when it opens that frame

### Requirement: Busy during the replay
The frame SHALL stay busy until the replay ends, holding the strokes of the person as it does today, and then the core MUST clear the busy mark.

#### Scenario: Person draws during replay
- **WHEN** the person draws on the frame while the replay runs
- **THEN** the stroke is held and sent after the replay

### Requirement: Time limit and Stop
A replay SHALL take at most 8 seconds whatever the number of pixels, and pressing Stop SHALL jump to the final state at once.

#### Scenario: Big write
- **WHEN** an agent writes 5000 pixels in one call
- **THEN** the replay ends within 8 seconds

#### Scenario: Stop
- **WHEN** the person presses Stop during a replay
- **THEN** the canvas shows the final state immediately and the frame is free

### Requirement: Reduced motion
With reduced motion on, the editor SHALL skip the replay and show the final state.

#### Scenario: Reduced motion
- **WHEN** reduced motion is on and an agent draws
- **THEN** the final state appears without cursor movement

### Requirement: Follow the agent
When an agent edits a sprite or frame that is not open, the editor SHALL show it only if the person is in the editor tab and has no stroke in progress; otherwise it MUST NOT change what the person sees.

#### Scenario: Person is busy
- **WHEN** the person is drawing and an agent edits another sprite
- **THEN** the editor stays on the person's sprite
