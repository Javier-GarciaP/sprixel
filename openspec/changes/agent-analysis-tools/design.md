# Design

## Context

Every pixel tool writes layer 0 of one frame as one atomic history entry
(`mcp/tools/write.rs`). A frame is a `Vec<u8>` of palette keys (one ASCII byte
per pixel, `.` is transparent), row by row. Version 1 shows one layer.

## Goals / Non-Goals

**Goals:**
- Exact, predictable rules an agent can rely on and a test can check.
- Deterministic output: the same frame and arguments give the same result.

**Non-Goals:**
- Smart art decisions (anti-aliasing, dithering, hue shifting).
- Several layers, selections or undo rules different from other write tools.

## Decisions

- **Pure module.** `src-tauri/src/analysis/` has functions from
  `(pixels, width, height, options)` to a list of changes or a report. No Tauri
  or document types except `PixelChange`. The tools in `mcp/tools/analysis.rs`
  read arguments, check keys and frames, call the module and use `write()`.
- **Opaque** means any key except `.`. Neighbors are the 4 pixels up, down,
  left and right (connectivity 4), or those plus the 4 diagonals (connectivity 8).
  The canvas edge is not a pixel: outside the canvas there are no neighbors.
- **Exterior.** The exterior is the set of transparent pixels that can be
  reached from a transparent pixel on the canvas border by moving up, down,
  left or right only through transparent pixels. A transparent pixel that is
  not in the exterior is a hole (it is enclosed). One function computes it and
  `auto_outline` and `analyze_sprite` both use it.
- **Order.** Changed pixels are listed row by row, top to bottom and left to
  right, so the replay in the editor is stable.
- **Only real changes.** A pixel that already has the wanted key is not part
  of the write. If nothing changes, nothing is written (`changed` false).

### `auto_outline`

Arguments: `name`, `folder`, `frame` (default 1), `key` (required, a palette
key, not `.`), `connectivity` (4 or 8, default 8), `outside_only` (default
false), `inner` (default false), `preview`, `base_revision`.

- Default (`inner` false): every transparent pixel that has an opaque neighbor
  (by `connectivity`) gets `key`. This outlines holes too.
- `outside_only` true: only transparent pixels in the exterior are drawn, so
  holes are not outlined.
- `inner` true: nothing is drawn on transparent pixels. Instead every opaque
  pixel that has a transparent neighbor (by `connectivity`) gets `key`, so the
  outline is inside the shape. With `outside_only` it counts only neighbors in
  the exterior. The canvas edge does not make a pixel an edge pixel.
- Outline pixels that would fall outside the canvas cannot be drawn. They are
  skipped and counted in `clipped`. Sprite size never changes.
- The rule looks at the frame as it was before the call. Calling the tool twice
  draws a second ring; call it once, when the shape is done.
- Answer: `pixels_changed`, `clipped`, and the usual `revision`, `changed`,
  `sprite`, preview.

### `auto_shade`

Arguments: `name`, `folder`, `frame` (default 1), `light` (required: `top-left`,
`top`, `top-right`, `left`, `right`, `bottom-left`, `bottom`, `bottom-right`),
`base` (required: the key of the pixels to shade), `keys` (required: the ramp,
from dark to light, a list of keys or a text like `"DBL"`; 2 to 8 different
keys; it MUST contain `base`), `region` (optional box `{x, y, width, height}`),
`preview`, `base_revision`.

Let `n` be the number of ramp keys and `i` the position of `base` in `keys`
(from 0). There are `s = i` darker keys and `m = n - 1 - i` lighter keys.
`d` is the step toward the light: top-left = (-1,-1), top = (0,-1), top-right
= (1,-1), left = (-1,0), right = (1,0), bottom-left = (-1,1), bottom = (0,1),
bottom-right = (1,1).

For every pixel `p` whose key is `base` (measured on the frame before the call):

- `a(p)` = how many steps `p + d`, `p + 2d`, ... stay on pixels with key `base`
  before the first pixel that is not `base` or is outside the canvas. So
  `a(p) = 0` means `p` is on the edge that faces the light.
- `b(p)` = the same, walking `-d`. `b(p) = 0` means `p` is on the edge that
  faces away from the light.
- Lit: `a(p) < m` and shadow: `b(p) < s`.
- If only lit applies, `p` gets `keys[n - 1 - a(p)]` (the lightest key at the
  edge, the next one a step inside, ...). If only shadow applies, `p` gets
  `keys[b(p)]` (the darkest key at the edge, ...).
- If both apply, the smaller distance wins. If the distances are equal, `p`
  keeps `base` (the corners that face sideways to the light stay neutral).
- Pixels with another key, and transparent pixels, are never touched.
- With `region`, only pixels inside the box are changed. Distances are still
  measured on the whole frame, so the box does not create new edges.
- Example: a 6x6 square of `B`, light `top-left`, keys `DBL`, base `B`: the top
  row and left column (except the top-right and bottom-left corners, which are
  a tie) become `L`; the bottom row and right column (except those corners)
  become `D`.
- Answer: `pixels_changed`, `lit` (pixels set lighter), `shadow` (pixels set
  darker), and the usual fields.

### `analyze_sprite`

Read-only (`Need::Nothing`, not in the activity log). Arguments: `name`,
`folder`, `outline_key` (optional palette key), `ground_rows` (default 2, from
1), no frame argument: all frames are checked. The answer:

```
{ "sprite": "Characters/goblin", "revision": 7, "width": 16, "height": 16, "frames": 2,
  "colors": { "count": 4, "used": ["K","B","D","L"], "unused": ["W"] },
  "frame_reports": [ { "frame": 1,
      "loose_pixels": { "count": 1, "at": [[3,4]] },
      "outline_gaps": { "count": 0, "at": [] } } ],
  "changes": [ { "from": 1, "to": 2, "changed_pixels": 12,
      "bbox": { "x": 3, "y": 4, "width": 5, "height": 6 }, "ground_kept": true } ] }
```

- `colors.used` are the opaque keys that appear in any frame, in palette order;
  `count` is their number; `unused` are the palette keys that appear in no
  frame of this sprite (palette order).
- A loose pixel is an opaque pixel with no opaque neighbor (connectivity 8).
- `outline_gaps` is only present when `outline_key` is given. A gap is an
  opaque pixel whose key is not `outline_key` and that has a neighbor (up,
  down, left or right) in the exterior: the shape is open to the outside there.
  Holes do not need an outline. The canvas edge does not count as outside.
- `at` lists `[x, y]` row by row and has at most 50 items; `count` is always the
  full number, so `count > at.length` means the list was cut.
- `changes` is present only when the sprite has more than one frame. Entry
  `k` compares frame `k` with frame `k + 1`. `changed_pixels` counts pixels
  with a different key; `bbox` is the smallest box that holds them (`null` when
  none). `ground_kept` is true when the bottom `ground_rows` rows (at most the
  height) are equal in both frames.
- Errors: `not_found`, `ambiguous_name`, `unknown_key`, `bad_argument`.

### Errors and permissions

`auto_outline` and `auto_shade` need the permission "Edit pixels" (`Need::Edit`),
go through `write()` (one history entry, `base_revision`, busy mark, replay
event, `preview`) and appear in the activity log. `analyze_sprite` needs no
permission. Errors use the codes of the other tools: `bad_argument`, `bad_frame`,
`unknown_key`, `not_found`, `ambiguous_name`, `out_of_range` (a `region` outside
the sprite).

## Risks / Trade-offs

- The ray rule of `auto_shade` is simple, not artistic. It gives a clean
  one-pixel band per ramp step; the agent can still fix single pixels.
- A large sprite (512x512) costs one pass per ramp step at most; no problem.
