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

## Order of work

1. `get_project`: read the project first (name, revision, folders, palettes).
   If it says no project is open, call `list_projects`, pick the right one and
   call `open_project` with its absolute path.
2. `get_palette`: read the colors. Draw with the keys of the palette only.
3. `create_sprite` for a new sprite, or `get_sprite` to read an existing one.
4. Draw:
   - `apply_grid`: best for a whole frame or a region (a text grid, one
     character per pixel).
   - `set_pixels` and `draw_shape`: for details and fixes.
5. `render_preview`: look at the result. If something is wrong, fix it and look
   again. Do not say you are done before you have seen the preview.

Small sprites (up to about 32x32) are drawn most reliably.

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
