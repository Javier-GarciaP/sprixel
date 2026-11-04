# pixel-canvas Specification

## Purpose
Defines how the canvas shows a sprite and how a person moves around it, so that
pixels look exact at every zoom level and the editor stays smooth.

## Requirements

### Requirement: Exact pixels
The canvas SHALL draw every pixel as a sharp square at a whole-number zoom, with no smoothing or blur at any zoom level.

#### Scenario: Zoomed in
- **WHEN** a sprite is shown at 800%
- **THEN** each sprite pixel is an 8x8 block of screen pixels with sharp edges

### Requirement: Transparent background
Transparent pixels SHALL be shown over a neutral gray checkerboard that is the same in the light and dark themes, so that black and white pixels stay visible.

#### Scenario: Black outline in the dark theme
- **WHEN** a sprite with a black outline is shown in the dark theme
- **THEN** the outline is clearly visible against the checkerboard

### Requirement: Zoom
The user SHALL zoom with the `+` and `-` keys, the mouse wheel with Control and the buttons in the bottom bar, from 100% to 3200% in whole steps, and use "Fit" to fit the sprite in the area.

#### Scenario: Fit
- **WHEN** the user presses "Fit" on a 120x30 sprite
- **THEN** the sprite is scaled by the largest whole zoom that fits in the area

### Requirement: Panning
The user SHALL move the view by holding the Space key and dragging, or by dragging with the middle mouse button, and the zoom MUST keep the point under the pointer fixed.

#### Scenario: Zoom at the pointer
- **WHEN** the user zooms in with the wheel while pointing at a pixel
- **THEN** that pixel stays under the pointer

### Requirement: Pixel grid
When the pixel grid option is on and the zoom is 600% or more, the canvas SHALL draw thin grid lines between pixels.

#### Scenario: Grid at 800%
- **WHEN** the option is on and the zoom is 800%
- **THEN** grid lines are visible between pixels

### Requirement: Position and size
The editor SHALL show the size of the sprite next to its name (not as a menu, because the size is fixed when the sprite is created) and the pixel coordinates under the pointer in the bottom bar.

#### Scenario: Coordinates
- **WHEN** the pointer is over the pixel at column 3 and row 4
- **THEN** the bottom bar shows "x: 3 y: 4"

### Requirement: Onion skin
The editor SHALL be able to show the previous frame under the current one in blue at an opacity chosen between 10% and 80% (40% by default), with a switch in the right panel, and the previous frame MUST NOT be editable or exported.

#### Scenario: Drawing frame 2
- **WHEN** onion skin is on and the user is on frame 2
- **THEN** the pixels of frame 1 appear in blue below the pixels of frame 2

#### Scenario: First frame
- **WHEN** the user is on frame 1
- **THEN** no onion skin is shown

### Requirement: Editor layout
The editor SHALL show the tools on the left, the frames column, the canvas in the center with a bar (name, size, undo, redo, active tool, canvas and text switch) and a right panel with the preview, the palette, the onion skin options and the agent activity.

#### Scenario: Open a sprite
- **WHEN** a sprite is opened
- **THEN** all these areas are visible and the canvas is centered

### Requirement: Agent drawing marker
While an AI agent is writing to the sprite, the canvas SHALL show an amber border and a label with the agent name and a Stop button, and the frame being written MUST be marked as busy.

#### Scenario: Agent draws frame 2
- **WHEN** an agent starts writing to frame 2
- **THEN** the label "Claude Code is drawing frame 2" and Stop appear until the work ends

### Requirement: Speed
Painting on the canvas MUST meet the drawing speed budget of the platform spec for a sprite of 512x512.

#### Scenario: Large sprite
- **WHEN** the user draws a long stroke on a 512x512 sprite
- **THEN** the pixels follow the pointer without visible delay
