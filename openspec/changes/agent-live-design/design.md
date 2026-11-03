# Design

## Context

Today one call is one atomic write (`mcp/tools/write.rs`) and the editor
reloads the whole frame only when the busy mark ends. There is no per-pixel
event. The canvas keeps a typed array of palette codes and repaints with
dirty rectangles (`render/renderer.ts`, `PixelCanvas.tsx`).

## Goals / Non-Goals

**Goals:**
- The person sees the agent work, with a cursor, in a short time.
- Designs from agents are good by default.

**Non-Goals:**
- Slowing down the agent. The write is applied at once; only the view replays.
- Letting the person edit during the replay.

## Decisions

- **The UI replays; the core does not wait.** The write is stored first, then
  the event is emitted. No extra state in the core except the busy mark.
- **Busy ends when the UI says so.** A new command `agent_playback_done`
  clears it. A core timer (10 s) clears it too, in case the UI is closed.
- **Pure timeline.** A function turns the pixel list into steps (pixels per
  frame of animation) with a cap of 8 s. It is unit tested without a DOM.
- **Cursor is an overlay** above the canvas, placed with the zoom and offset of
  the view, tagged with the agent name and color.
- **Follow rules protect the person.** No tab or sprite change while a stroke
  is in progress.
- **One text, three places.** The guide text lives in `tools/docs.rs`; the
  skill file is generated from it by a test that fails when they differ.

## Risks / Trade-offs

- Many calls in a row queue replays. A queue plays them in order and drops to
  final state if more than 3 are waiting.
- Large events cost memory. Pixels are sent as compact arrays.
