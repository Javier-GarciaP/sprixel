# Proposal

## Why

The editor canvas is the main thing people use. It must be fast, show pixels
exactly, and make mistakes cheap to fix.

## What Changes

- Build the editor layout: tools column, canvas area, right panel (preview,
  palette, onion skin, agent activity).
- Build the pixel canvas: exact scaling, zoom, panning, checkerboard, optional
  grid, coordinates and size label.
- Build the tools: pencil, eraser, bucket, eyedropper, line, rectangle,
  ellipse, select and move, with two colors, shortcuts and undo and redo.
- Build onion skin: the previous frame in blue under the drawing.
- Show the marker and the stop control when an AI agent is drawing.

## Capabilities

### New Capabilities
- `pixel-canvas`: the canvas, its view controls, onion skin and the editor layout.
- `drawing-tools`: the tools, colors, strokes and undo and redo from the user side.

### Modified Capabilities

## Impact

- New feature folder `src/features/canvas-editor/`.
- Drawing happens outside React state and sends one operation per stroke to
  the core (see `project-format-and-storage`).
- Relies on the speed budget from `project-foundation`.
