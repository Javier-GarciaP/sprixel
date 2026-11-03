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
The answer has the new `revision`. Check the result with render_preview.";
