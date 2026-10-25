# Proposal

## Why

A sprite becomes an animation with frames and named animations (walk, idle,
attack). People need to add and arrange frames, name ranges of them and see
them play.

## What Changes

- Build the frames column: thumbnails, select, add, duplicate, delete and
  reorder.
- Build animation tags (name, from-to frames, speed, repeat mode) with the New
  animation dialog and the tag selector above the frames.
- Build the live preview with play, pause and speed.
- Show and edit animations in the sprite details of the Project tab.

## Capabilities

### New Capabilities
- `frames-and-animations`: frames, animation tags and the live preview.

### Modified Capabilities

## Impact

- New feature folder `src/features/frames-animations/`.
- Uses document operations for frames and tags from `project-format-and-storage`.
- The export spec reads tags and durations defined here.
