# Spec Delta

## Purpose

Defines the power tools a person has in the editor, the same ones an agent has.

## ADDED Requirements

### Requirement: Commands for the editor tools
The core SHALL have a command for each editor tool. Each one MUST use the same pure function as the matching MCP tool, apply with the author `user` as one history entry, and fail with `frame_busy` when an agent is writing to that frame.

#### Scenario: Same result as the agent
- **WHEN** a person flips a region with the command and an agent flips the same region with `transform_region`
- **THEN** both give the same pixels

#### Scenario: One entry
- **WHEN** a command changes pixels and the person presses Undo
- **THEN** one Undo restores all of them

#### Scenario: Busy frame
- **WHEN** an agent is writing to the frame
- **THEN** the command fails with `frame_busy` and nothing changes

### Requirement: Selection actions
The editor SHALL show a selection bar when a selection exists, with buttons and shortcuts to flip horizontally, flip vertically, rotate 90, 180 and 270, and mirror a half onto the other half. The arrow keys SHALL move the pixels inside the selection (the whole frame if nothing is selected) one pixel, with an option to wrap.

#### Scenario: Flip
- **WHEN** the person presses Flip horizontal with a selection
- **THEN** only the selected pixels are flipped

#### Scenario: Rotate a rectangle
- **WHEN** the selection is not square and the person rotates 90 degrees
- **THEN** a clear message is shown and nothing changes

#### Scenario: Arrow move
- **WHEN** the person presses the right arrow with a selection
- **THEN** the pixels inside the selection move one pixel right, pixels that leave the selection are lost unless wrap is on, and the selection stays where it is

### Requirement: Copy and paste
The editor SHALL copy the selected region and paste it into any frame or sprite that uses the same palette. When the palette differs it MUST show a clear message and change nothing.

#### Scenario: Paste in another frame
- **WHEN** a region is copied and pasted in frame 2
- **THEN** frame 2 has the pixels and frame 1 is unchanged

#### Scenario: Different palette
- **WHEN** the person pastes into a sprite with another palette
- **THEN** a message says the palettes differ and nothing changes

### Requirement: Symmetry mode
The editor SHALL have a symmetry mode with the values off, horizontal, vertical and both. The axis MUST be drawn on the canvas. The pencil, line, rectangle, ellipse and bucket SHALL draw mirrored. One stroke MUST be one history entry.

#### Scenario: Mirrored stroke
- **WHEN** the mode is horizontal and the person draws a pixel at x 1 on a 16 pixel wide sprite
- **THEN** pixels at x 1 and x 14 are drawn

#### Scenario: One entry
- **WHEN** a mirrored stroke is drawn and Undo is pressed
- **THEN** both sides are undone together

### Requirement: Outline and Shade with preview
The editor SHALL have an Outline dialog (key, 4 or 8 neighbors, outside only, inner) and a Shade dialog (light direction, base key, ramp, optional selection region). While a dialog is open the canvas MUST show the result as a preview without writing. Apply writes it as one entry; Cancel changes nothing.

#### Scenario: Preview
- **WHEN** the person changes an option in the Outline dialog
- **THEN** the canvas shows the new outline and the document is not changed

#### Scenario: Apply and undo
- **WHEN** the person applies a shade and presses Undo
- **THEN** the sprite is as it was before

### Requirement: Review panel
The editor SHALL have a Review panel with the loose pixels, the outline gaps (for a chosen outline key), the unused colors and the comparison of each frame with the next. Clicking a pixel item MUST highlight that pixel on the canvas.

#### Scenario: Loose pixel
- **WHEN** a frame has a pixel that touches no other pixel
- **THEN** the panel lists it, and clicking it highlights it

#### Scenario: Clean sprite
- **WHEN** nothing is wrong
- **THEN** the panel says so

### Requirement: Palette ramp and cleanup
The palette editor SHALL generate a ramp from a base color, a number of steps and a hue shift, and SHALL remove unused colors. Remove MUST list the colors first and ask for confirmation. Each action is one history entry.

#### Scenario: Generate ramp
- **WHEN** the person generates 5 steps from a base color
- **THEN** the palette gets the new tones (a tone that already exists is reused)

#### Scenario: Remove unused
- **WHEN** the person asks to remove unused colors
- **THEN** the dialog lists them and nothing is removed until confirmed

### Requirement: Frame durations and anchors
The frames strip SHALL let a person set the duration of a frame and SHALL show the speed of the animation when the frame has no duration of its own. The canvas SHALL show anchors as a small cross with its name, and a person SHALL create, drag, rename and delete them.

#### Scenario: Duration
- **WHEN** the person sets 200 ms on frame 2
- **THEN** frame 2 shows 200 and the other frames still show the animation speed

#### Scenario: Drag an anchor
- **WHEN** the person drags an anchor to another pixel
- **THEN** the anchor is saved at that pixel as one entry

### Requirement: Preview tools
The editor SHALL have a preview background (checker, light, dark), a 3x3 tile preview that warns when the edges do not match, and an onion skin that shows 1 to 3 previous frames.

#### Scenario: Seam warning
- **WHEN** the first and last columns differ
- **THEN** the tile preview shows a warning with the number of mismatches

#### Scenario: Three onion frames
- **WHEN** the onion skin is set to 3 on frame 4
- **THEN** frames 3, 2 and 1 are shown, older ones fainter
