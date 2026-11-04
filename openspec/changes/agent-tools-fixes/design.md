# Design

## Context

See proposal.md. The UI work (`editor-power-tools`) calls the same pure
functions in `analysis` and `mcp/tools/region.rs`. So old public signatures stay;
new behavior comes as new functions and new parameters.

## Decisions

- **`dry_run` lives in `write()`.** The tool builds its plan as always. With
  `dry_run: true`, `write()` checks `base_revision` and the plan, then applies
  the operations to a COPY of the project, draws the preview from the copy, and
  returns. No history entry, no revision change, no busy marker, no replay
  event, no change event. The answer has `dry_run: true`, `changed: false`,
  the counts of the tool, `would_change` (true if the plan has operations) and
  the preview image (unless `preview` is false). The activity log records the
  call like any other call.
- **Thin parts of `inner`.** A shape pixel is *covered* if it belongs to some
  3x3 square made only of opaque pixels. Pixels that are not covered, are on the
  edge (they would become outline) and that `auto_outline` really changes are
  *thin*. Thin pixels are grouped by 8-neighbors and each group gives a
  bounding box. A hat brim 2 rows high is thin; the rim of a thick body is not,
  because those pixels are covered. `thin_regions` has the boxes and `warnings`
  has one sentence. New pure function `analysis::thin_regions`.
- **Speckled shading.** After the shade, a changed pixel is *isolated* if none
  of its 4 side neighbors has the same new key. `speckled_pixels` is the count;
  a warning is added when it is more than 0. New pure function
  `analysis::isolated_changes`.
- **`checked` / `not_checked`.** Names: `colors`, `loose_pixels`, `outline`,
  `frame_changes`, `ground`. With one frame, `frame_changes` and `ground` are
  not checked. Without `outline_key` the tool detects the key: the opaque pixels
  that have a side neighbor in the outside (the edge of the silhouette) are
  counted over all frames; the most common key wins if it is more than half of
  them. Otherwise (no shape, a tie, no clear winner) `outline` goes to
  `not_checked`. The key used is in `outline_key_used`, also when it was given.
- **`ground_kept` rule.** For each pair of frames: if the sprite has an anchor
  named `feet` (ignoring case; the anchor of the frame wins over the one of the
  whole sprite), `ground_kept` is true when both frames have the same `y` for it
  (`ground_rule: "feet_anchor"`). If it has no anchor, it is true when the lowest
  opaque row of both frames is the same (`ground_rule: "lowest_row"`); two empty
  frames are equal. A lifted foot does not change the lowest row while the other
  foot stays. The old measure (the bottom `ground_rows` rows are equal) stays as
  `bottom_rows_equal`. New pure function `analysis::lowest_row`.
- **Region writes keep pixels.** `apply_grid` with `x` or `y` uses
  `skip_transparent: true` unless the call says `false`. Whole-frame grids keep
  erasing. `write()` counts the opaque pixels that a plan sets to `.` and puts
  `erased` in the answer when it is more than 0. If the call was a region
  `apply_grid` with `skip_transparent: false` and `erased` > 0, `warnings` says
  that `.` erased pixels.
- **Frame duration.** The model stores one number per frame, and 100 ms
  (`DEFAULT_FRAME_MS`) means "no duration of its own". So an explicit 100 ms and
  the default cannot be told apart. At the tool boundary `get_sprite` shows
  `null` for 100 and the number otherwise; `set_frame_duration` accepts
  `milliseconds: null` and stores `DEFAULT_FRAME_MS`; when it gets exactly 100 it
  says in `note` that the frame follows the animation speed. We do not change the
  model. Follow-up: a real optional duration in the file format belongs to the
  spec `format-and-compat` (that folder does not exist in this branch, so it is
  in the final report).
- **Guides.** The `animation` and `walk-and-attack-cycles` texts get the new
  rules; the `tools` topic gets one example call per tool; INSTRUCTIONS and the
  skill get a short paragraph on `dry_run`, `checked`/`not_checked` and `erased`.
  A test keeps the skill equal to the server text.

## Risks

- Cloning the project for a dry run costs memory on big projects. It is one
  copy per call and it is dropped at once.
- The thin-part rule can flag one-pixel tips of round shapes. It only warns.
