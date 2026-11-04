# Design

## Context

A sprite has one frame list and tags `{name, from, to, fps, mode}` (Aseprite model). Operations `AddTag`,
`SetTag`, `RemoveTag` and `MoveFrame` exist in the core. The editor has `AnimationDialog`, `AnimationSelect`
(a filter "All") and `FramesColumn`. The MCP only has `create_tag`.

## Goals / Non-Goals

**Goals:** a simple pattern known from other tools; one call creates an animation without sending pixels;
the frames strip shows each animation as its own group; agents get a clear recipe and rules.

**Non-Goals:** changing the file model to separate frame lists per animation; GIF; export changes (tags
already export to the sprixel JSON and the Aseprite `frameTags`).

## Decisions

- **Tags stay.** No migration. An animation is a tag; its frames are the range from..to.
- **No overlap.** New animations never overlap another; existing files that overlap are shown in the first
  group that owns the frame and a warning is shown. `create_tag` keeps working and rejects an overlap with
  `animation_overlap` for new tags.
- **Base = frame 1** unless `base_frame` says otherwise. The group "Base" is the frames before the first
  animation.
- **`create_animation` copies the base** n times at the end of the sprite (or at `at`), tags them and sets the
  fps and mode, in ONE history entry. Names are unique per sprite ignoring case.
- **`delete_animation` has `with_frames`** (default false: only the tag goes). With frames, the frames go and
  later tags move (the core already shrinks tags when frames are deleted).
- **`frames` in region tools:** a list or ranges (`[8,9,10]` or `"8-13"`), one history entry, the same
  region on every frame, atomic.
- **Parts (phase 2):** a sprite keeps an optional list of named rectangles; tools `set_part`, `remove_part`;
  `transform_region` and `copy_region` accept `part` instead of `region`. Undo, frame size checks.
- **UI uses the same operations.** The grouped strip is a view over tags; every action is one Undo step.
- **Rules for agents** live in the guide, the server instructions and the skill (a test keeps them equal):
  base first, one tag per animation, derive frames from the base, never generate pixels outside the app
  without asking.

## Risks / Trade-offs

- Old files with overlapping tags: shown with a warning, never rewritten.
- Grouping many frames needs virtual lists if a sprite has hundreds of frames: collapse groups by default
  above 40 frames.
