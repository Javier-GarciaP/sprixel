# Design

## Context

See proposal.md. Frames and tags are part of the sprite in the document model.
The preview reuses the canvas renderer at a small size.

## Goals / Non-Goals

**Goals:**
- Frame and tag changes are normal operations (undoable, same for AI).
- A preview that follows what is being drawn.

**Non-Goals:**
- Layers per frame, cel offsets or timeline scrubbing.

## Decisions

- **Frame and tag changes are operations** (`add_frame`, `move_frame`,
  `set_tag`, ...), so undo and the MCP tools use the same path.
- **Preview in its own small canvas** with a timer driven by
  `requestAnimationFrame` and the durations from the core.
- **Tags may overlap.** A frame can be part of several animations (for example
  a shared idle frame).
- **Ranges are rewritten by the core** when frames are inserted, deleted or
  moved, so the UI never edits ranges by hand.

## Risks / Trade-offs

- [Reordering with overlapping tags is ambiguous] → define the rule: a tag
  keeps the same frames and its range is recomputed from the new positions; if
  they are no longer contiguous the tag is shown with a warning.
- [Playback timing drifts] → use elapsed time, not a fixed interval.
