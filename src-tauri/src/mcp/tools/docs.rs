//! The long texts that tools show to the AI. They are the instructions an AI
//! has, so they explain the text grid format and the palette keys.

/// The description of `apply_grid`.
pub const APPLY_GRID: &str = "\
Writes a text grid into a sprite. One character is one pixel and each character is a palette key; `.` is always transparent. \
Read the palette first (get_palette) and use only its keys.\n\
\n\
Format (keywords are English):\n\
palette: main            (optional)\n\
legend                   (optional; `<key> <name> [#RRGGBB]`)\n\
  Q teal #1FB5A8         a legend line with a #hex ADDS the color Q if the palette has no Q, in the same step\n\
frame 1                  (frame numbers start at 1)\n\
....RRRR                 one row per line, no spaces\n\
...RRRRR\n\
\n\
Rules: all rows have the same length (the width) and their number is the height. Without `x` and `y` the grid is a whole frame and must be \
exactly the size of the sprite. With `x` and `y` it is a region written at that top left position, and it may be smaller. \
A `.` erases the pixel unless `skip_transparent` is true. Keys are case sensitive. If you only have rows, pass `frame`.\n\
\n\
It is atomic: if anything is wrong nothing changes, and `errors` lists every problem with its `line`, `row` and `column` \
(codes: row_length, row_count, unknown_key, no_frame, bad_frame, too_big, region_out, palette_full, bad_legend). \
The answer has the new `revision`; pass `base_revision` (the revision you last read) to fail with `stale_revision` instead of overwriting newer changes. The person can undo the call. Check the result with render_preview.";

/// What the server tells every client when it connects (`instructions` in the
/// answer of `initialize`). Short: a client may put it in every prompt.
pub const INSTRUCTIONS: &str = "\
Sprixel is a pixel art editor for game sprites. You draw on the person's real canvas while they watch.\n\
\n\
Workflow: get_project, then get_palette (use only its keys), then create_sprite or get_sprite, then draw with apply_grid \
(best for whole frames and regions), set_pixels and draw_shape (for details), then render_preview to SEE the result and fix it.\n\
\n\
Pixels: x and y count from 0 at the top left; frames count from 1; a color is a one-character palette key; `.` is always transparent. \
A text grid is one character per pixel, one row per line; the exact format is in the description of apply_grid. \
Small sprites (up to about 32x32) are drawn most reliably.\n\
\n\
Every write returns the new `revision`. Pass it back as `base_revision` and the call fails with `stale_revision` if the project changed. \
Every error has a `code` and a `message` that says how to fix it, and each call is applied whole or not at all. \
If a call fails with `permission_denied`, ask the person to change the permission in the Agents tab. \
The person can stop you (`stopped_by_user`) and undo your last action.";
