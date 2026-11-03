# Spec Delta

## Purpose

Defines the tools that let an agent edit and read big sprites cheaply.

## ADDED Requirements

### Requirement: Transform a region
The tool `transform_region` SHALL change a rectangle of one frame (the whole frame if none is given) with the operation `flip_horizontal`, `flip_vertical`, `mirror` (with a `direction`), `move` (`dx`, `dy`, `wrap`) or `rotate` (`degrees`). It MUST be one atomic history entry, need the Edit permission and return a preview.

#### Scenario: Flip
- **WHEN** an agent flips a 4x2 region horizontally
- **THEN** each row of the region is reversed and the pixels outside the region stay

#### Scenario: Mirror a half
- **WHEN** an agent mirrors `left_to_right` on an 8 pixel wide frame
- **THEN** columns 4 to 7 become columns 3 to 0

#### Scenario: Move without wrap
- **WHEN** a region is moved by `dx` 2 without `wrap`
- **THEN** pixels that leave the region disappear and the free pixels become `.`

#### Scenario: Move with wrap
- **WHEN** a region is moved by `dx` 2 with `wrap` true
- **THEN** pixels that leave one side come back on the other side

#### Scenario: Rotate
- **WHEN** a square region is rotated 90 degrees
- **THEN** it turns clockwise

#### Scenario: Rotate a rectangle
- **WHEN** a region that is not square is rotated 90 or 270 degrees
- **THEN** the call fails with `bad_argument` and nothing changes

#### Scenario: Region out of the sprite
- **WHEN** the region does not fit in the sprite
- **THEN** the call fails with `region_out` and nothing changes

### Requirement: Copy a region
The tool `copy_region` SHALL copy a rectangle of a frame to a position of the same or another frame or sprite (`to_sprite`, `to_folder`, `to_frame`, `to_x`, `to_y`). With `skip_transparent` the `.` pixels of the source do not erase the destination. Both sprites MUST use the same palette. It MUST be one atomic history entry and need the Edit permission.

#### Scenario: Copy to another frame
- **WHEN** a region is copied to frame 2 of the same sprite
- **THEN** frame 2 has those pixels at `to_x`, `to_y` and frame 1 is unchanged

#### Scenario: Skip transparent
- **WHEN** `skip_transparent` is true
- **THEN** destination pixels under a `.` of the source stay

#### Scenario: Different palettes
- **WHEN** the two sprites use different palettes
- **THEN** the call fails with `palette_mismatch`

### Requirement: Draw only over a color
`draw_shape`, `set_pixels` and `apply_grid` (with `x` and `y`) SHALL accept `only_over`, one palette key. Only pixels that have that key before the call MAY change.

#### Scenario: Shape over a color
- **WHEN** a filled rectangle is drawn with `only_over` R on a frame with red and green pixels
- **THEN** the red pixels inside the rectangle change and the green ones stay

#### Scenario: Unknown key
- **WHEN** `only_over` is not in the palette
- **THEN** the call fails with `unknown_key`

### Requirement: Symmetry
`set_pixels`, `draw_shape` and `apply_grid` SHALL accept `symmetry` with `horizontal`, `vertical` or `both`, and an optional `axis` with `x` and `y`. Each pixel is also written at its mirrored place. The default axis is the center of the sprite.

#### Scenario: Horizontal
- **WHEN** a pixel is set at x 1 on an 8 pixel wide sprite with `horizontal`
- **THEN** the pixels at x 1 and x 6 are set

#### Scenario: Both
- **WHEN** `both` is used for one pixel
- **THEN** four pixels are set

#### Scenario: Mirror outside
- **WHEN** a mirrored pixel would be outside the sprite
- **THEN** the call fails with `out_of_range` and nothing changes

### Requirement: Read a part of a sprite
`get_sprite` SHALL accept `region` (`x`, `y`, `width`, `height`) and `include_palette`. With a region the grid has only the rows and columns of the region. With `include_palette` false the grid has the line `palette: <name>` and no legend. The grid MUST stay a valid text grid. Without these arguments the answer MUST be the same as before.

#### Scenario: Region
- **WHEN** a 2x2 region of a frame is read
- **THEN** the grid has 2 rows of 2 characters

#### Scenario: No legend
- **WHEN** `include_palette` is false
- **THEN** the grid has no `legend` line and the answer still names the palette

#### Scenario: Region out
- **WHEN** the region does not fit in the sprite
- **THEN** the call fails with `region_out`

### Requirement: Import a reference image
The tool `import_reference` SHALL read a PNG file from the folder `references/` of the open project and write it into a frame with the colors of the sprite palette. Alpha under 128 MUST be transparent. `width` and `height` scale the image down with nearest neighbor. `max_colors` allows adding up to that many new colors to the palette. The file name MUST NOT contain `..`, be absolute or be a symbolic link, MUST end with `.png`, MUST be at most 2 MB and 512 pixels on a side.

#### Scenario: Import
- **WHEN** a 4x4 PNG with two palette colors and a transparent pixel is imported
- **THEN** the frame has the matching keys and `.` for the transparent pixel

#### Scenario: Nearest color
- **WHEN** the image has a color that is not in the palette
- **THEN** the pixel gets the key of the nearest palette color

#### Scenario: Add colors
- **WHEN** `max_colors` is 1 and the image has a new color
- **THEN** that color is added to the palette in the same history entry

#### Scenario: Scale down
- **WHEN** an 8x8 image is imported with `width` 4 and `height` 4
- **THEN** a 4x4 area of the frame is written

#### Scenario: Not allowed path
- **WHEN** the file is `../secret.png`, an absolute path, a symbolic link out of the folder, not a PNG, or too big
- **THEN** the call fails with an error code and nothing changes

### Requirement: Documented
The new tools MUST be listed in `docs/ai-and-mcp.md` and in the activity log, and the guide text MUST name them.

#### Scenario: Docs match
- **WHEN** the tests run
- **THEN** every tool of the server is in the docs and the skill equals the server guide
