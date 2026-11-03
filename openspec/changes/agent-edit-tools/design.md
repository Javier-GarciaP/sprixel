# Design

## Context

Every write tool builds a list of `Operation::SetPixels` and gives it to
`write.rs`, which gives one history entry, `base_revision`, the busy mark, the
replay event and the preview. Pixels are one byte per pixel (the palette key).

## Goals / Non-Goals

**Goals:**
- Common edits are one cheap call.
- Reading a part of a big sprite is cheap.

**Non-Goals:**
- Layers, selections that persist, or free rotation.
- Reading images outside the project folder.

## Decisions

- **No new operation.** Every new tool computes the final pixels of the region,
  compares them with the current ones and sends only the changed pixels as
  `SetPixels`. So history, undo, replay and the file format do not change.
- **Two region tools.** `transform_region` works on one frame in place
  (`flip_horizontal`, `flip_vertical`, `mirror`, `move`, `rotate`).
  `copy_region` has a source and a destination, which can be another sprite or
  frame, so it is a separate tool with a clearer schema.
- **Pure functions.** The math (flip, mirror, rotate, move, copy, symmetry,
  quantize) lives in plain functions on byte buffers, tested without a project.
- **`only_over` and `symmetry` are one helper.** `finish_pixels` takes the wanted
  pixels, adds mirrored points, keeps the first value for a repeated point,
  drops pixels whose current key is not `only_over`, and drops pixels that do not
  change. `set_pixels`, `draw_shape` and `apply_grid` all call it.
- **Symmetry axis.** `horizontal` mirrors left to right (x changes), `vertical`
  top to bottom (y changes). Default axis is the center of the sprite
  (`(width - 1) / 2`). `axis: {x, y}` accepts whole or half numbers. A mirrored
  point outside the sprite is `out_of_range` and nothing changes.
- **Partial grid.** `write_grid` gets a region and an option to leave out the
  legend. The text stays a valid text grid: the rows of a region can be sent
  back to `apply_grid` with `x` and `y`.
- **Reference images.** The file name is checked as a path inside
  `references/` (no `..`, no absolute path, no symlink, `.png`, 2 MB, 512 pixels
  a side, checked before decoding). The image is scaled with nearest neighbor,
  alpha under 128 is `.`, and each color goes to the nearest palette color in
  RGB. With `max_colors`, up to that many of the most used colors that are not
  near the palette are added first.

## Risks / Trade-offs

- A reference with many colors looks poor with a small palette. `max_colors`
  lets the agent choose.
- Tests for files need a temp project folder; they use generated PNGs.
