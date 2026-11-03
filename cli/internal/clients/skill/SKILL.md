---
name: sprixel
description: Draw pixel art sprites in the Sprixel app. Use when the user asks you to draw, edit or animate a game sprite, or to work in a Sprixel project.
---

# Sprixel

Sprixel is a pixel art editor for game sprites. You draw on the person's real
canvas through the `sprixel` MCP tools, and they watch while you work.

## Connect

1. The Sprixel app must be running. If a tool fails because the app is closed,
   run `sprixel start` in the shell and try again.
2. The first time, the person must press **Allow** for your client in the
   **Agents** tab of the app. Until then tools fail with `awaiting_approval`:
   ask them to allow you, then retry.
3. If the `sprixel` tools are not available at all, run `sprixel doctor` and
   follow its advice.

## Workflow

Follow these steps in this order:

1. `get_project`: read the project. If it says no project is open, call `list_projects` and then `open_project`.
2. `get_guide` with the topic `design` (and `animation` for a sprite that moves). Follow it.
3. `get_palette`: read the colors. Draw with the keys of the palette only; a `legend` line in `apply_grid` adds a new color. If a legend line gives a used key another color, the pixels move to a free key and the answer says so.
4. `create_sprite` for a new sprite (`own_palette: true` gives it a palette of its own), or `get_sprite` to read an existing one.
5. Draw: `apply_grid` for a whole frame or a region, `set_pixels` and `draw_shape` for details and fixes.
6. LOOK at the image every write returns (or call `render_preview`). If something looks wrong, fix it and look again. Do not say you are done before you have seen the result and it looks right.

If you only have a shell (no MCP tools), use `sprixel tool <name> key=value ...`
for every step. Every write (`apply_grid`, `set_pixels`, `draw_shape`) and
`sprixel tool render_preview name=goblin` save the image as a PNG file and print
`image saved: <path>`. OPEN that file with your Read tool, look at it, and fix
what looks wrong before you say you are done. Send `preview=false` to skip the
image of a write.

Small sprites (up to about 32x32) are drawn most reliably.

## Design rules

Design guide: how to make a sprite that looks good

1. Small palette with ramps. Use 4 to 16 colors. For each material make a ramp of 2 or 3 tones (dark, base, light). Reuse the same colors everywhere. Add a color only when you need it.
2. One light direction. Choose it first (for example from the top left) and keep it in the whole sprite: light tones on the side that faces the light, dark tones on the other side.
3. Silhouette first. Block the whole shape with flat colors before any detail. If the silhouette is not clear, shading will not fix it.
4. Flat fills. Fill big areas with one tone. No noise, no dithering, unless the person asks for it.
5. Two or three tones of shading per material: base, shadow and highlight. Put them in small clean shapes, not in scattered pixels.
6. Selective outline. Outline the outside of the shape with a dark tone of the material (not black everywhere). Inside the shape use no lines, or lines lighter than the outer outline.
7. No stray pixels. Every pixel must belong to a shape. Remove single pixels that touch nothing, and keep lines clean, with steady steps.
8. Look at the result. Every pixel write (`apply_grid`, `set_pixels`, `draw_shape`) returns an image of the edited frame. LOOK at it; if you have no image, call `render_preview`. If something looks wrong, fix it and look again. Do not say you are done before you have seen the image and it looks right.
9. Big sprites (more than 32x32): work region by region. Write one part at a time (head, body, legs) with `apply_grid` and `x` and `y`, look at the image after each part, then go on.

For moving sprites call `get_guide` with `animation`; for colors, with `palette`.

## Worked example

Worked example: a 16x16 slime with an idle frame and a jump frame.
1. `create_sprite` with name slime, width 16 and height 16.
2. `add_frame` with name slime (now it has 2 frames).
3. `apply_grid` with name slime and this grid. The `legend` adds the colors; frame 1 is the idle pose and frame 2 is the jump:

legend
  K outline #1E3A2B
  D shade #3F9B4F
  B base #6FD36B
  L light #B8F2A6
  W eye #FFFFFF
frame 1
................
................
................
................
................
.....KKKKKK.....
....KLLLBBBK....
...KLLBBBBBBK...
..KLLBBBBBBBBK..
..KLBBWKBBWKBK..
.KBBBBWKBBWKBBK.
.KBBBBBBBBBBBBK.
.KBBBBBBBBBBDDK.
.KBBBBBBBBDDDDK.
..KDDDDDDDDDDK..
...KKKKKKKKKK...
frame 2
................
................
................
......KKKK......
.....KLLBBK.....
....KLLBBBBK....
....KLBBBBBK....
...KLBWKBWKBK...
...KBBWKBWKBK...
...KBBBBBBBBK...
...KBBBBBBDDK...
....KBBBBDDK....
....KDDDDDDK....
.....KKKKKK.....
................
................

4. LOOK at the image that came back. Fix what is wrong with `set_pixels` (a few pixels) or `apply_grid` with `x` and `y` (a region) and look again.
5. `create_tag` with sprite slime, name jump, from 1, to 2, fps 4: the two frames become an animation.

## Rules

- `x` and `y` count from 0, at the top left.
- Frames count from 1.
- A color is a one-character palette key. `.` is always transparent.
- Every write returns the new `revision`. Pass it back as `base_revision` and
  the call fails with `stale_revision` if the project changed meanwhile.
- Each call is applied whole or not at all.
- The person can stop you and can undo your last action.

## When a call fails

Every error has a `code` and a `message` that says how to fix it. Read it.

- `awaiting_approval`, `client_rejected`, `not_approved`: the person has not
  allowed you. Ask them to press Allow in the Agents tab.
- `permission_denied`: a permission is off. Tell the person which one (the
  message names it) and ask them to turn it on in the Agents tab. Do not try to
  work around it. `open_project` needs "Open projects", which is off by default.
- `stale_revision`: the project changed. Read it again, then redo your change.
- `stopped_by_user`: the person pressed Stop. Stop working and ask what to do.
- `newer_changes`: someone changed the project after your last action, so it
  cannot be undone by you. Read the project again.
- `not_found`, `ambiguous_name`, `name_exists`, `invalid_name`: a name problem.
  Use `list_sprites` to see the real names.
- `not_a_project`: the folder is not a Sprixel project. Use `list_projects`.
- `row_length`, `row_count`, `unknown_key`, `no_frame`, `bad_frame`,
  `region_out`, `too_big`: the grid or drawing is not valid. `errors` lists
  every problem with its `line`, `row` and `column`. Fix them all and send the
  grid again.
