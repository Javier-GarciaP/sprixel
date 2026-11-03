# Design

## Context

`frame_durations` and the operation `SetFrameDuration` already exist. The
animation preview and the export already use the durations (a frame with the
default 100 ms in a tag uses `1000 / fps`).

## Goals / Non-Goals

**Goals:** engine-ready data; small, simple additions.

**Non-Goals:**
- **Marks of anchors on the editor canvas: out of scope.** The canvas paints
  from a typed array outside React; drawing marks and keeping them in sync
  would touch the canvas code, which is not easy. Agents see anchors in
  `get_sprite`. A later spec can add it.
- A UI to edit anchors or durations by hand (durations already have one).
- Changing the text grid format: anchors live in the sprite file only.

## Decisions

- **Anchor** = `{ name, x, y, frame }`. `frame` is `None` for the whole sprite
  or a frame number. A name is unique per `(name, frame)` ignoring case. `x`
  and `y` are pixel cells inside the sprite. An anchor of a frame wins over the
  anchor of the same name for the whole sprite.
- **Operations:** `SetAnchor { sprite, anchor, index }` (adds, or replaces the
  one with the same name and frame; its inverse is `RemoveAnchor` or the old
  `SetAnchor`) and `RemoveAnchor { sprite, name, frame }`.
- **Frame operations keep per-frame anchors right:** delete drops them and
  shifts the later ones (`RestoreFrame` gets the dropped anchors to undo);
  insert shifts; move remaps the frame numbers (a permutation, so the inverse
  needs nothing extra).
- **File:** `anchors` array in the sprite file, written only when not empty
  and read with a default of empty, so old files open and files without
  anchors do not change. `formatVersion` stays 1 (additive).
- **Export:** every frame has `anchors`: the effective anchors of that frame
  (frame ones override sprite ones), sorted by name, with coordinates
  multiplied by the export scale. The field is always present (maybe `[]`).
  The JSON version stays 1: a reader that ignores unknown fields is not
  broken.
- **`check_tile`:** compares column 1 with the last column and row 1 with the
  last row, on one frame (default 1), all layers flattened. A pair matches
  when both pixels are transparent, or both have the same key. It lists the
  mismatching pairs (at most 50, plus the real count).
- **`render_tiled`:** the frame repeated 3x3 on a gray checkerboard, scale
  chosen so the image is at most 1024 pixels on a side (same cap as
  `render_preview`); optional `scale`.

## Risks

- Frame numbers inside anchors must follow frame operations; tests cover
  delete, undo, insert and move.
