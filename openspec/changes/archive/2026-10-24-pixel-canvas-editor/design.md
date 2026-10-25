# Design

## Context

See proposal.md. Drawing speed is the main technical risk and is measured in
`project-foundation`. The core owns the document; the canvas keeps only a
working copy of the visible frame.

## Goals / Non-Goals

**Goals:**
- Pixel exact rendering and drawing that feels instant.
- Strokes that are one undo step and one operation for the core.

**Non-Goals:**
- Layers, symmetry, brushes of other sizes, gradients or selections other than
  rectangles. They can be added later.

## Decisions

- **One `<canvas>` and a typed array of palette indexes** for the visible
  frame. Alternatives: DOM cells (too slow for large sprites), WebGL (kept as
  the fallback if the benchmark fails).
- **Outside React state.** A small drawing controller handles pointer events
  and paints directly; React only renders the panels.
- **Strokes become one operation** containing only the changed pixels, sent to
  the core on pointer release. While drawing, the preview is local.
- **Nearest-neighbor scaling by drawing at integer zoom.** No CSS smoothing.
- **Onion skin is drawn as a second layer** in the canvas renderer with a blue
  tint and the chosen opacity; it is never part of the document.
- **Busy frames.** The core sends events when an agent starts and ends a write;
  the controller queues the user stroke until then.

## Risks / Trade-offs

- [The local copy and the core diverge] → the core is the truth: after each
  operation the controller refreshes from the core event.
- [Large flood fills freeze the UI] → use an iterative fill and test on 512x512.
- [Keyboard shortcuts clash with typing] → shortcuts are off while a text field
  has the focus.
