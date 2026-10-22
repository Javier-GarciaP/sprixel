# Spec Delta

## Purpose

Defines the tools a person uses to draw, how colors are chosen, and how each
stroke becomes one undoable action.

## ADDED Requirements

### Requirement: Tool set
The editor SHALL provide these tools with these keys: Pencil (B), Eraser (E), Bucket (G), Eyedropper (I), Line (L), Rectangle (R), Ellipse (C), Select (M) and Move (V), with the active tool highlighted and its name and key shown when the pointer rests on it.

#### Scenario: Switch tool by key
- **WHEN** the user presses `E`
- **THEN** the Eraser becomes the active tool

### Requirement: Colors
The editor SHALL keep a main and a secondary color taken only from the palette, draw with the main color on the left button and the secondary on the right, and offer a button and a key to swap them.

#### Scenario: Right click
- **WHEN** the user draws with the right button using the Pencil
- **THEN** the pixels use the secondary color

### Requirement: Pencil and eraser
The Pencil SHALL set the touched pixels to the active color and the Eraser to transparent, with no gaps when the pointer moves fast.

#### Scenario: Fast stroke
- **WHEN** the pointer moves across 100 pixels in one frame
- **THEN** every pixel on the path is painted

### Requirement: Bucket
The Bucket SHALL replace the connected area of the same color under the pointer with the active color, not crossing other colors, and a click on an area already of that color MUST change nothing.

#### Scenario: Fill an outline
- **WHEN** the user fills the inside of a closed outline
- **THEN** only the connected pixels inside the outline change

### Requirement: Eyedropper
The Eyedropper SHALL set the main color to the color under the pointer, or leave it unchanged if the pixel is transparent.

#### Scenario: Pick a color
- **WHEN** the user clicks a red pixel with the Eyedropper
- **THEN** the main color becomes that red

### Requirement: Shapes
Line, Rectangle and Ellipse SHALL show a live preview while the pointer is dragged and apply on release. Holding Shift MUST constrain a line to 45-degree steps and a rectangle or ellipse to a square or circle, and shapes can be outlined or filled.

#### Scenario: Square with Shift
- **WHEN** the user drags a rectangle holding Shift
- **THEN** the result is a square

### Requirement: Select and move
The Select tool SHALL select a rectangular area, and the Move tool SHALL move the selected pixels (or the whole frame if there is no selection) without changing their colors, clipping what leaves the canvas.

#### Scenario: Move a selection
- **WHEN** the user selects an area and drags it 3 pixels to the right
- **THEN** the pixels move 3 pixels and the old place becomes transparent

### Requirement: One action per stroke
Each stroke, fill, shape or move SHALL be one entry in the history, so that one undo reverts the whole stroke, and undo and redo MUST be available with `Ctrl+Z` and `Ctrl+Shift+Z` and as buttons in the canvas bar.

#### Scenario: Undo a stroke
- **WHEN** the user draws a stroke of 30 pixels and presses undo
- **THEN** all 30 pixels return to their previous color in one step

### Requirement: Drawing is clipped
Painting outside the sprite bounds SHALL be ignored, and drawing MUST be blocked while the frame is marked as busy by an agent, with the stroke applied when the agent finishes.

#### Scenario: Pointer leaves the canvas
- **WHEN** a stroke continues outside the canvas
- **THEN** only the pixels inside the sprite are painted
