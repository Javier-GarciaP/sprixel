# Design

## Context

The storage module already has a migration framework (`Migrator`, `Step`,
`migrate_project`) with a backup in `.sprixel/backups/`, and files carry
`formatVersion` (camelCase). The app has no older version, so its list of
steps is empty. The anchors and the frame durations were added without a bump.

## Goals / Non-Goals

**Goals:** old files keep opening; a future format change has a clear path;
engines can import the export; a sprite can change its canvas.

**Non-Goals:** a format version 2 (nothing needs it yet); GIF export; changing
the Sprixel JSON `meta.version`; a UI to edit anchors.

## Decisions

- **Key name stays `formatVersion`.** It is what every file already has. A
  file with no `formatVersion` is version 1 (the first files). A version that
  is not a number is damaged.
- **Old files open without a bump.** A sprite file with no `frameDurations`
  gets 100 ms for every frame; with no `anchors` it has none. These are
  defaults, not steps. A step in `Migrator::app()` is only needed when a
  change cannot be a default. The rules are in `docs/project-format.md`.
- **Newer version:** refused with `NewerVersion` at open time, before any
  write; the migration code and `Session::open` check the manifest first.
- **Export `index` and `frame_number`:** `index` is the position in
  `frames` from 0 (the same numbers `animations[].from/to` use);
  `frame_number` is the frame number inside its sprite from 1 (what the
  editor and the tools show). The Sprixel layout stays version 1.
- **Pivot:** the anchor `feet` of the frame (frame anchors win, as in
  `anchors`), else the first anchor of the frame, else `0, 0`. Scaled like
  the anchors.
- **Aseprite JSON** (`json_format: "aseprite"`): the "array" form of the
  Aseprite sheet: `frames` is an array of `{ filename, frame {x,y,w,h},
  rotated, trimmed, spriteSourceSize, sourceSize, duration }` and `meta` has
  `app`, `version`, `image`, `format`, `size`, `scale` (a string, as Aseprite
  writes it) and `frameTags` (`name`, `from`, `to`, `direction`). Frames of
  several sprites are all in one list; a tag `name` is `sprite_tag` when the
  scope has more than one sprite. `from`/`to` count from 0. `pingpong` is
  `pingpong`, `once` is `forward` (the loop is the engine's choice).
  Default stays `sprixel`.
- **`ResizeSprite { sprite, width, height, anchor }`:** every frame of every
  layer is cropped or extended with transparent pixels. The anchor says where
  the old content stays (9 places). Sprite anchors move by the same offset;
  those that end outside are removed and listed as warnings (the answer of
  the tool). Tags, durations, and the palette do not change. The inverse is
  `RestoreCanvas`, which holds the old size, pixels and anchors, so undo is
  exact. Limits 1 to 512 per side; the same size is `bad_argument`.
- **Where:** the pixel math is in `document/resize.rs` (pure functions),
  the operation in `operation.rs`, the tool in `mcp/tools/resize.rs`.
- **UI:** a button "Change canvas" in the details panel of one sprite opens a
  dialog with width, height and a 3x3 anchor grid; it calls the same core
  function as the tool.

## Risks

- A big crop loses pixels: undo restores them (tested), and the dialog warns
  when a side gets smaller that pixels and anchors can be lost.
- Engine importers are not tested with the engines themselves (Linux CI
  only): the docs say so.
