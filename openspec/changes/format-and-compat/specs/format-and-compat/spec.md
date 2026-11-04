# Spec Delta

## Purpose

Makes the project format and the export safe to depend on for 1.0.0, and
lets a sprite change its canvas size.

## ADDED Requirements

### Requirement: Format versions
Every project file SHALL carry an integer `formatVersion`; a file without it is version 1. Older files MUST open, including files written before anchors and frame durations existed. Opening a project saved by a newer Sprixel MUST be refused with a clear message and nothing MUST be written. Migrations run when a project opens and are written on the next save. `docs/project-format.md` SHALL list the versions and the rules for changing the format.

#### Scenario: File without version
- **WHEN** a project file has no `formatVersion`
- **THEN** it opens as version 1

#### Scenario: Old sprite file
- **WHEN** a sprite file has no `anchors` and no `frameDurations`
- **THEN** it opens with no anchors and 100 ms per frame, and no data is lost

#### Scenario: Newer project
- **WHEN** a project has a `formatVersion` higher than the app knows
- **THEN** opening fails with an error that asks to update the app, and no file is changed

#### Scenario: Migration chain
- **WHEN** a project is older than the current version by several versions
- **THEN** one step per version runs in order, the files are written atomically and a backup exists

### Requirement: Frame numbers and pivot in the export
The Sprixel JSON SHALL give every frame `index` (position in `frames`, from 0) and `frame_number` (number inside its sprite, from 1). `pivot` SHALL be the anchor named `feet` of the frame, else its first anchor, else `0, 0`.

#### Scenario: Frame numbers
- **WHEN** a sprite with 3 frames is exported
- **THEN** its frames have `index` 0 to 2 and `frame_number` 1 to 3

#### Scenario: Feet pivot
- **WHEN** a frame has an anchor `feet` at 3,7 and the scale is 2
- **THEN** its `pivot` is 6,14; with no `feet` the first anchor is used; with no anchor it is 0,0

### Requirement: Aseprite JSON option
The `export` tool and the Export screen SHALL accept `json_format` `sprixel` (default) or `aseprite`. `aseprite` writes an Aseprite array-form sheet: `frames` with `frame`, `duration` and `sourceSize`, and `meta` with `frameTags`, `size`, `image` and `scale`. The documented samples MUST equal the real output.

#### Scenario: Aseprite JSON
- **WHEN** `json_format` is `aseprite`
- **THEN** the JSON has `frames` with `frame`, `duration` and `sourceSize`, and `meta.frameTags` with the animations

#### Scenario: Bad json_format
- **WHEN** `json_format` is something else
- **THEN** the call fails with `bad_argument`

### Requirement: Change the canvas size
The operation `ResizeSprite` SHALL crop or extend every frame of every layer to a new width and height (1 to 512) with transparent pixels, keeping the old content at one of nine anchors. It MUST be one undoable history entry. The size is chosen when the sprite is created and changed only by this explicit action.

#### Scenario: Extend
- **WHEN** a 4x4 sprite is resized to 8x8 with anchor `center`
- **THEN** the old pixels are at 2,2 and the rest is transparent

#### Scenario: Crop and undo
- **WHEN** a sprite is cropped and the change is undone
- **THEN** the old size, pixels and anchors are back exactly

#### Scenario: Limits
- **WHEN** a side is 0 or more than 512, or the size does not change
- **THEN** the call fails with `bad_argument` and nothing changes

### Requirement: Anchors follow a resize
Sprite anchors SHALL move with the content when the canvas changes. Anchors that end outside the new canvas MUST be removed and reported as warnings.

#### Scenario: Anchors
- **WHEN** an anchor ends outside the new canvas
- **THEN** it is removed and the answer lists it as a warning; the others moved by the same offset

### Requirement: Resize tool and dialog
The tool `resize_sprite` (permission Edit, optional `preview`) and a dialog "Change canvas" in the sprite details panel SHALL do the resize with the same options.

#### Scenario: Tool
- **WHEN** an agent calls `resize_sprite` with a sprite, a size and an anchor
- **THEN** the canvas changes as one history entry and the person can undo it

#### Scenario: Dialog
- **WHEN** the person opens "Change canvas", types a size, picks an anchor and confirms
- **THEN** the sprite changes size

### Requirement: Game engine guide
`docs/game-engines.md` SHALL explain, step by step, how to go from a drawing to Godot 4 and Unity, with the export options and where the feet anchor goes. Every option and file name in backticks MUST exist in the export code, and a test MUST check it.

#### Scenario: Options exist
- **WHEN** the tests run
- **THEN** every export option named in the page is a real option
