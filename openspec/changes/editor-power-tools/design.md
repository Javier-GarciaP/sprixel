# Design

## Context

The editor sends strokes as `editor_apply` (author `user`, one history entry).
The agent tools build `Operation::SetPixels` from pure functions
(`mcp/tools/region.rs`, `analysis/`, `document/ramp.rs`, `render/`).

## Goals / Non-Goals

**Goals:**
- A person gets each agent tool with the same result.
- Every action is one history entry; undo and redo work.

**Non-Goals:**
- Layers, free rotation, persistent selections.
- Changing what the pure functions do (another spec improves them).

## Decisions

- **Thin IPC layer.** A new `ipc/power.rs` has plain functions (no window) and
  one `#[tauri::command]` each. They call the same pure functions as the MCP
  tools, then `session.apply("user", ops)`. Small helpers (`cut`, `changes`,
  `add_ramp`) become `pub(crate)`; their behavior does not change. Tests check
  that a command and the MCP tool give the same pixels.
- **Busy frame.** Like `editor_apply`, a write on a frame an agent is writing
  fails with `frame_busy`.
- **Preview is read-only.** Outline and Shade preview commands return the list
  of changed pixels `(x, y, key)` and write nothing. The canvas paints them as
  a translucent overlay. Apply calls the write command.
- **Symmetry lives in the UI.** The `DrawingController` already collects one
  stroke into one `onCommit`. In symmetry mode each drawn point is also drawn
  at its mirror points before the commit, so the stroke is still one
  `editor_apply` and one history entry. The axis is the center of the sprite.
- **Copy and paste.** The clipboard is a small value held by the editor
  (`palette`, size, keys). It is not the document. Paste goes through the core,
  which checks the palette name and the size, and uses `paste`.
- **Review.** One read command `edit_review` returns the same data as
  `analyze_sprite`. Clicking an item sets a highlight in the canvas scene.
- **Palette.** Generate ramp and Remove unused colors use `add_ramp` and
  `used_keys` and the existing `palette_edit_operations`, so they are one
  entry. The dialog shows what will happen before it asks to confirm.
- **Durations and anchors.** They are existing operations
  (`SetFrameDuration`, `SetAnchor`, `RemoveAnchor`). New commands wrap them.
  Frame info already knows the tags; it gets durations and anchors.
- **Preview tools are view settings.** Background, tile preview and onion
  frames count are UI state (not in the document). The tile check is a
  read command that uses the same rule as `check_tile`.
- **Shortcuts.** Each new key is a command in `app/commands/registry.ts` with a
  default, so the Settings page lists it and it can be changed.

## Risks / Trade-offs

- The editor and the agent can touch the same frame: the busy check covers it.
- Many new buttons can crowd the bar: the selection bar only shows when
  something is selected, and the dialogs open on demand.
