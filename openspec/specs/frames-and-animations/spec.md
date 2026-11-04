# frames-and-animations Specification

## Purpose
Defines how frames are managed, how a range of frames becomes a named animation,
and how the animation is previewed.

## Requirements

### Requirement: Frames column
The editor SHALL list the frames of the sprite in a column with a thumbnail and the number of each, highlight the current frame, and select a frame on click.

#### Scenario: Select a frame
- **WHEN** the user clicks the thumbnail of frame 3
- **THEN** frame 3 becomes the current frame and the canvas shows it

### Requirement: Add frames
The user SHALL add a frame with the "+" button, either empty or as a copy of the current frame, always placed right after the current frame.

#### Scenario: Add after the current frame
- **WHEN** frame 2 of 4 is current and the user adds a frame
- **THEN** the sprite has 5 frames and the new one is frame 3 and becomes current

### Requirement: Duplicate and delete frames
The user SHALL duplicate and delete frames from the frame menu, and the last remaining frame MUST NOT be deletable.

#### Scenario: Delete the only frame
- **WHEN** a sprite has one frame and the user tries to delete it
- **THEN** the action is disabled and the frame stays

### Requirement: Reorder frames
The user SHALL reorder frames by dragging their thumbnails, and animation ranges MUST follow the frames they contain.

#### Scenario: Move a frame
- **WHEN** the user drags frame 4 before frame 2
- **THEN** the order becomes 1, 4, 2, 3 and the pixel content of each frame is unchanged

### Requirement: Frame duration
Each frame SHALL have an optional duration in milliseconds, and when it is not set the speed of the animation in fps MUST decide it.

#### Scenario: Default duration
- **WHEN** an animation runs at 12 fps and its frames have no duration
- **THEN** each frame lasts about 83 ms

### Requirement: Animation tags
The user SHALL create, edit and delete animations from the "+" in the sprite details, with a name, a start and end frame, a speed (1 to 60 fps) and a repeat mode (repeat, once or ping-pong), with validation errors shown next to the field.

#### Scenario: Create an animation
- **WHEN** the user creates "walk" from frame 1 to 4 at 12 fps repeating
- **THEN** it appears in the sprite details as "walk - frames 1-4 - 12 fps - repeats"

#### Scenario: Invalid range
- **WHEN** the user sets the end frame below the start frame
- **THEN** the field shows an error and the create button is disabled

### Requirement: Animation selector
The selector above the frames column SHALL list the animations of the sprite plus "All", and choosing one MUST show only its frames in the column and in the preview.

#### Scenario: Filter by animation
- **WHEN** the user chooses "walk" (frames 1 to 4) on a sprite with 7 frames
- **THEN** the column shows frames 1 to 4 and the preview plays only them

### Requirement: Live preview
The right panel SHALL show a preview that plays the selected animation (or all frames) with a play and pause button and a speed control from 1 to 60 fps, following the repeat mode, and MUST update when the drawing changes.

#### Scenario: Play
- **WHEN** the user presses play on an animation set to ping-pong
- **THEN** the preview plays forward and then backward, repeatedly

#### Scenario: Edit while playing
- **WHEN** the user draws on a frame while the preview plays
- **THEN** the preview shows the new pixels on the next loop
