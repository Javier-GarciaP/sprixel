# Spec Delta

## Purpose

Defines the images an agent can ask for to see its work.

## ADDED Requirements

### Requirement: Preview background
`render_preview` and the image returned by `apply_grid`, `set_pixels` and `draw_shape` SHALL accept `background` with the values `checker`, `light`, `dark` and `transparent`. The default MUST be `checker`, a neutral gray checkerboard. With `transparent` the transparent pixels MUST keep alpha 0. Any other value MUST fail with `bad_argument` and change nothing.

#### Scenario: White sprite is visible
- **WHEN** an agent previews a sprite with white pixels and no `background`
- **THEN** the white pixels differ from the background pixels around them

#### Scenario: Transparent background
- **WHEN** an agent previews with `background` `transparent`
- **THEN** the pixels that are `.` in the sprite have alpha 0

#### Scenario: Unknown background
- **WHEN** an agent sends `background` `purple`
- **THEN** the call fails with `bad_argument` and a write tool changes nothing

### Requirement: Frames sheet
The tool `render_frames` SHALL return one PNG with the frames of a sprite (all, or those from `from` to `to`) in a grid of `columns`, at `scale`, over the chosen background. The image MUST be at most 1024 pixels on a side; otherwise the call fails with `too_big` and a `max_scale`. It is a read tool and changes nothing.

#### Scenario: All frames
- **WHEN** an agent calls `render_frames` for a sprite of 4 frames with `columns` 2
- **THEN** the image has 2 columns and 2 rows of frames, in order

#### Scenario: Range
- **WHEN** an agent sends `from` 2 and `to` 3
- **THEN** only those two frames are in the image

#### Scenario: Too big
- **WHEN** the grid would be wider than 1024 pixels at the given scale
- **THEN** the call fails with `too_big`

#### Scenario: Bad frame
- **WHEN** `to` is larger than the number of frames
- **THEN** the call fails with `bad_frame`

### Requirement: Frame labels
With `labels` true, `render_frames` SHALL draw the number of each frame above it, with a built-in 3x5 pixel font.

#### Scenario: Labels
- **WHEN** an agent sends `labels` true
- **THEN** the image has an extra band above each frame with its number, and with `labels` false it has none

### Requirement: Onion image
The tool `render_onion` SHALL return one PNG with the listed frames drawn over each other in the given order. The last frame MUST be opaque and untinted. Each earlier frame MUST be tinted and translucent.

#### Scenario: Movement
- **WHEN** a pixel is at x 1 in frame 1 and at x 2 in frame 2, and an agent sends `frames` [1, 2]
- **THEN** the pixel at x 2 has its real color and the pixel at x 1 is tinted and not fully opaque

#### Scenario: Bad frame list
- **WHEN** `frames` is empty or has a frame that does not exist
- **THEN** the call fails with `bad_argument` or `bad_frame`

### Requirement: Images from the shell
`sprixel tool` SHALL save the image of `render_frames` and `render_onion` as a PNG file and print its path, as it does for `render_preview`. Two different images MUST NOT use the same file.

#### Scenario: Frames from the shell
- **WHEN** a shell agent runs `sprixel tool render_frames name=goblin`
- **THEN** a PNG file is saved and `image saved: <path>` is printed
