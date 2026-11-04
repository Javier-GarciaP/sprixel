# Spec Delta

## Purpose

Defines the fixes to the agent tools found by a real agent: dry runs, honest
analysis, a ground check that follows the support, safe region writes and
frame durations without magic values.

## ADDED Requirements

### Requirement: Dry run
`auto_outline`, `auto_shade`, `transform_region`, `copy_region` and `apply_grid` SHALL accept `dry_run` (boolean, default false). With `dry_run` true nothing is written: the revision, the history, the replay and the UI do not change. The answer SHALL have `dry_run: true`, `changed: false`, the same counts as a real call and, unless `preview` is false, an image of what the frame would look like.

#### Scenario: Outline without writing
- **WHEN** an agent calls `auto_outline` with `dry_run` true on a 2x2 block
- **THEN** the answer says `pixels_changed` 12 with an image, and the pixels, the revision and the history are the same as before

#### Scenario: Dry run of a grid
- **WHEN** an agent calls `apply_grid` with `dry_run` true and a legend that adds a color
- **THEN** nothing is written and the color is not added

#### Scenario: Stale revision
- **WHEN** a dry run has an old `base_revision`
- **THEN** it fails with `stale_revision`, like a real call

### Requirement: Outline warns about thin parts
`auto_outline` with `inner` true SHALL return `thin_regions` (bounding boxes) and a `warnings` entry for every group of shape pixels that is not part of any 3x3 square of the shape and would turn entirely into outline.

#### Scenario: Hat brim
- **WHEN** `inner` outlines a shape that has a brim 2 rows high
- **THEN** the answer has a warning and the box of the brim

#### Scenario: Thick shape
- **WHEN** `inner` outlines a 5x5 square
- **THEN** there are no warnings

### Requirement: Shade warns about speckles
`auto_shade` SHALL return `speckled_pixels`, the number of changed pixels with no side neighbor of the same new key, and a `warnings` entry when it is more than 0.

#### Scenario: Two isolated pixels
- **WHEN** a 2x2 block is shaded with the light at the top left
- **THEN** `speckled_pixels` is 2 and there is a warning

### Requirement: Analysis says what it checked
`analyze_sprite` SHALL return `checked` and `not_checked` lists and MUST NOT report a result for something in `not_checked`. Without `outline_key` it SHALL use the most common key of the silhouette edge, if it is more than half of the edge pixels, and report `outline_key_used`; otherwise `outline` is in `not_checked`.

#### Scenario: Detected outline
- **WHEN** the edge of a sprite is drawn with `K` and no `outline_key` is given
- **THEN** `outline_key_used` is `K`, `outline` is in `checked` and `outline_gaps` is reported

#### Scenario: No clear outline
- **WHEN** the edge has two keys in equal numbers
- **THEN** `outline` is in `not_checked` and there is no `outline_gaps`

#### Scenario: One frame
- **WHEN** the sprite has one frame
- **THEN** `frame_changes` and `ground` are in `not_checked`

### Requirement: Ground follows the support
In `changes`, `ground_kept` SHALL compare the `y` of the anchor `feet` of the two frames when the sprite has one, and the lowest opaque row of the two frames otherwise, and `ground_rule` SHALL say which. The old fields stay, and the old measure is `bottom_rows_equal`.

#### Scenario: Lifted foot
- **WHEN** one foot goes up and the other stays on the ground row
- **THEN** `ground_kept` is true and `bottom_rows_equal` is false

#### Scenario: Support moves
- **WHEN** the `feet` anchor is at y 7 in frame 1 and y 6 in frame 2
- **THEN** `ground_kept` is false and `ground_rule` is `feet_anchor`

### Requirement: Region grids do not erase by default
`apply_grid` with `x` or `y` SHALL use `skip_transparent` true unless the call says false. A grid without `x` and `y` keeps erasing. Every write answer that erased opaque pixels SHALL have `erased` (count), and a region `apply_grid` with `skip_transparent` false that erased SHALL also have a warning.

#### Scenario: Region keeps pixels
- **WHEN** a region grid with a `.` over an opaque pixel is applied
- **THEN** the pixel stays and `erased` is not in the answer

#### Scenario: Explicit erase
- **WHEN** the same call has `skip_transparent` false
- **THEN** the pixel is erased, `erased` is 1 and `warnings` says so

### Requirement: Frame durations without magic numbers
`get_sprite` SHALL show `null` in `frame_durations` for a frame that follows the animation speed and the number of milliseconds for a frame with its own time. `set_frame_duration` SHALL accept `milliseconds: null` to go back to the animation speed.

#### Scenario: Default and own
- **WHEN** frame 2 has 150 ms and frame 1 has the default
- **THEN** `frame_durations` is `[null, 150]`

#### Scenario: Back to the animation speed
- **WHEN** `set_frame_duration` gets `milliseconds` null for frame 2
- **THEN** `frame_durations` is `[null, null]`

### Requirement: Guides teach the rules
The `animation` and `walk-and-attack-cycles` guides SHALL require a body bob, opposite arms, 6 or 8 frames for a walk, a fixed support foot (the `feet` anchor), ground contact in actions, and say that `ground_kept` follows the support foot. The `tools` guide SHALL have one example call per tool. The instructions and the skill SHALL explain `dry_run`, `checked`/`not_checked` and `erased`, and the skill SHALL equal the server text.

#### Scenario: Every tool has an example
- **WHEN** the tools guide is read
- **THEN** every tool has an example call with valid JSON arguments

#### Scenario: Skill and server agree
- **WHEN** the tests compare the skill with the server texts
- **THEN** they are the same
