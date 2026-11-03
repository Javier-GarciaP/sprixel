# Spec Delta

## Purpose

Defines the data a game engine needs from a sprite and the tools an agent
uses to set and check it.

## ADDED Requirements

### Requirement: Frame durations
The tool `set_frame_duration` SHALL set the duration of one frame in milliseconds, from 10 to 10000, as one undoable history entry. `get_sprite` SHALL return the duration of every frame.

#### Scenario: Set a duration
- **WHEN** an agent calls `set_frame_duration` with frame 2 and 250
- **THEN** frame 2 lasts 250 ms, `get_sprite` shows it and the person can undo it

#### Scenario: Out of range
- **WHEN** the duration is 5 or 20000, or the frame does not exist
- **THEN** the call fails with `bad_argument` or `bad_frame` and nothing changes

### Requirement: Anchor points
A sprite SHALL be able to have named anchor points with `x`, `y` inside the sprite and an optional frame. `set_anchor` adds or moves one, `remove_anchor` removes one, both undoable. `get_sprite` SHALL return the anchors. Anchors of a frame MUST follow the frame when frames are deleted, inserted, moved or restored by undo. Project files without anchors MUST open, and files of sprites without anchors MUST NOT change.

#### Scenario: Set and read
- **WHEN** an agent sets the anchor `feet` at 8,15
- **THEN** `get_sprite` lists it and undo removes it

#### Scenario: Outside the sprite
- **WHEN** x or y is outside the sprite, or the frame does not exist
- **THEN** the call fails with `bad_argument` or `bad_frame`

#### Scenario: Remove a missing anchor
- **WHEN** `remove_anchor` names an anchor that does not exist
- **THEN** it fails with `not_found`

#### Scenario: Frames change
- **WHEN** a frame with an anchor is deleted and the deletion is undone
- **THEN** the anchor is back on that frame

#### Scenario: Old file
- **WHEN** a sprite file has no `anchors` field
- **THEN** it opens with no anchors

### Requirement: Engine export
The sheet JSON SHALL give every frame its `duration` and its `anchors` (the effective anchors, scaled), next to the existing animations. `docs/export-format.md` SHALL explain how to map it to Godot `SpriteFrames` and to Unity, with an example, and a test MUST keep the documented sample equal to the real JSON.

#### Scenario: Anchors in the export
- **WHEN** a sprite with anchors is exported
- **THEN** each frame of the JSON lists its anchors, a frame anchor replacing a sprite anchor of the same name

### Requirement: Seamless tiles
`check_tile` SHALL report whether a frame repeats without seams by comparing the first and last column and the first and last row, and list the mismatching pixels. `render_tiled` SHALL return an image of the frame repeated 3x3. Both only read.

#### Scenario: Seamless
- **WHEN** opposite edges are equal
- **THEN** `check_tile` returns `seamless: true` and no mismatches

#### Scenario: Seam
- **WHEN** a pixel of the left edge differs from the one on the right edge of the same row
- **THEN** `seamless` is false and the pair is listed

#### Scenario: Tiled image
- **WHEN** `render_tiled` is called for a 16x16 sprite
- **THEN** the image shows the sprite 3 times wide and 3 times high
