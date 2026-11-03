# Spec Delta

## Purpose

Defines the tools that let the server outline, shade and check a sprite, so an
agent does not do it by hand.

## ADDED Requirements

### Requirement: Automatic outline
The tool `auto_outline` SHALL draw a one pixel outline with a palette `key` around the opaque pixels of a frame, using 4 or 8 neighbors (default 8), as one atomic write that needs the permission "Edit pixels". The options `outside_only` and `inner` are described in the scenarios.

#### Scenario: Outline a square
- **WHEN** an agent calls `auto_outline` with key `K` on a frame with a 2x2 block of `B` in the middle of an 8x8 sprite
- **THEN** the 12 transparent pixels around the block (8 neighbors) get `K` and the block is not changed

#### Scenario: Four neighbors
- **WHEN** the same call uses `connectivity` 4
- **THEN** the four corner pixels are not drawn and 8 pixels get `K`

#### Scenario: Outside only
- **WHEN** the shape has a closed hole and the call uses `outside_only`
- **THEN** the hole is not outlined, and without `outside_only` it is

#### Scenario: Inner outline
- **WHEN** the call uses `inner`
- **THEN** the opaque pixels at the edge of the shape get `K` and no transparent pixel changes

#### Scenario: Shape at the edge
- **WHEN** the shape touches the border of the sprite
- **THEN** the outline pixels that would be outside are not drawn and `clipped` counts them

#### Scenario: Unknown key
- **WHEN** the key is not in the palette
- **THEN** the call fails with `unknown_key` and nothing changes

### Requirement: Automatic shading
The tool `auto_shade` SHALL recolor the pixels with the key `base` using the ramp `keys` (dark to light) and the direction `light`, by the exact rule of `design.md`, as one atomic write that needs the permission "Edit pixels". The ramp MUST contain `base`.

#### Scenario: Square lit from the top left
- **WHEN** an agent shades a 6x6 square of `B` with light `top-left`, keys `DBL` and base `B`
- **THEN** the top row and left column are `L`, the bottom row and right column are `D`, the two corners with equal distances stay `B`, and the inside stays `B`

#### Scenario: Longer ramp
- **WHEN** the ramp has two lighter keys and two darker keys
- **THEN** the lightest key is at the lit edge, the next one a pixel inside, and the same for the dark side

#### Scenario: Region
- **WHEN** a `region` covers half of the square
- **THEN** only the pixels inside the box change, with the same colors they would have without a box

#### Scenario: Other keys are kept
- **WHEN** the shape has an outline of another key around the base
- **THEN** the outline pixels are never changed

#### Scenario: Bad ramp
- **WHEN** `base` is not in `keys`, or a key is not in the palette
- **THEN** the call fails with `bad_argument` or `unknown_key` and nothing changes

### Requirement: Sprite analysis
The tool `analyze_sprite` SHALL be read-only, need no permission and return structured JSON as defined in `design.md`: colors, loose pixels, outline gaps and the comparison of consecutive frames. Lists of coordinates MUST have at most 50 items while `count` stays complete.

#### Scenario: Loose pixels
- **WHEN** a frame has a block and one separate pixel
- **THEN** the report lists that pixel as loose and does not list the block

#### Scenario: Outline gaps
- **WHEN** a shape has an outline with one missing pixel and the call gives `outline_key`
- **THEN** the report lists the shape pixel that is open to the outside there

#### Scenario: Hole is not a gap
- **WHEN** the shape has an enclosed hole with no outline
- **THEN** the hole is not reported

#### Scenario: Unused colors
- **WHEN** the palette has a key that no frame uses
- **THEN** it is in `colors.unused` and not in `colors.used`

#### Scenario: Two frames
- **WHEN** a sprite has two frames that differ in a few pixels
- **THEN** `changes` has one entry with the count, the bounding box and `ground_kept`

#### Scenario: Ground moved
- **WHEN** the bottom rows differ between two frames
- **THEN** `ground_kept` is false

#### Scenario: One frame
- **WHEN** the sprite has one frame
- **THEN** the answer has no `changes`

#### Scenario: Nothing is changed
- **WHEN** an agent calls `analyze_sprite`
- **THEN** the revision, the history and the activity log do not change

### Requirement: Documentation
The three tools SHALL be listed in `docs/ai-and-mcp.md`, `auto_outline` and `auto_shade` SHALL appear in the activity list with a text in English and Spanish, and the design guide and the Claude Code skill SHALL tell agents to use them.

#### Scenario: Docs match the tools
- **WHEN** the docs test runs
- **THEN** every tool of the server is in the page and the skill equals the server guide
