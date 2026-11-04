# Text grid format

> **Status: built.** The parser, the writer and the validator are in the Rust
> core, and the **Text** view of the editor uses them. The tools `get_sprite`
> and `apply_grid` (see [AI and MCP](ai-and-mcp.md)) will use the same code.
> The full rules, with tests, are in the `text-grid-format` spec. Every grid
> in this page is checked by a test.

The **text grid** is the way Sprixel writes a sprite as plain text. Each pixel
is **one character**, and each character is the **key** of a color in the
palette. It is easy for people to read and the best way for an AI to draw.

## A first example

```
palette: dungeon16

legend
  . transparent
  N black   #000000
  P skin    #FFCC99
  A blue    #2F6BFF
  R red     #F04545

frame 1
....RRRR
...RRRRR
...NNNN.
..NPNPPN
..NPPPPN
..NNNNNN
.NAAAAN.
.NA..AN.
```

This is an 8x8 sprite. The first row `....RRRR` means: four transparent
pixels, then four red ones.

## Rules

### Pixels and keys

- **One character = one pixel.**
- Each key is **one printable ASCII character** that is not a space and not
  `.`. Letters and digits are recommended (`A`-`Z`, `a`-`z`, `0`-`9`).
  Keys are **case sensitive**: `G` and `g` are different colors.
- **`.` is always transparent.** It cannot be used as a color key.
- Every key must exist in the palette, **or be defined by the grid's own
  `legend`** (see below). The palette defines the key, the name and the color.
  A palette can have up to 93 colors.
- New colors get the next free key in this order: `A`-`Z`, `a`-`z`, `0`-`9`,
  then the other printable characters. Color names are unique in a palette.
- If a person changes a key in the palette editor, the pixels of the sprites
  change with it, but a grid written earlier with the old key becomes invalid
  (`unknown_key`). Read the palette again before writing a grid.

### Rows and size

- All rows of a frame must have **the same length**. That length is the width.
- The number of rows is the height.
- The size must match the sprite: for example 16 rows of 16 characters for a
  16x16 sprite.
- Maximum size is 512x512.

### Blocks

A text grid has these parts, in this order:

| Part | Required | Meaning |
| --- | --- | --- |
| `palette: <name>` | No | The palette the keys belong to. If missing, the project palette is used. |
| `legend` block | No | Lines of `<key> <name> [#hex]`. The name can have spaces. A line with a `#hex` for a key that is **not** in the palette adds that color when the grid is applied, in the same undo step as the pixels. A grid never changes an existing color. If the key exists with the **same** `#hex`, the line is only informational. If it exists with a **different** `#hex`, the line and every pixel of that key move to another key: the key that already has that hex, or else a free key, which is added as a new color. The old color stays as it was. The `apply_grid` tool then answers with `remapped` (for example `{"P": "Z"}`) and `warnings`. A legend line without a `#hex` does not define a key. `.` can be listed (`. transparent`) but never has a color. |
| `frame <n>` + rows | Yes, at least one | The pixels of frame number `n`. Frames are numbered from 1. |

- Keywords (`palette`, `legend`, `frame`) are always **English**, even when the
  interface is in Spanish.
- Blank lines are ignored. A line that starts with `#` after optional spaces
  is a comment. `#` is also a valid color key, so inside a frame only `#`
  followed by a space is a comment: a row that starts with `#` and then a
  color (`#..R`) is a row.
- A line that starts with the word `frame` and a space is a frame line. A row
  can never have spaces, so a row like `frame` (five keys) is still a row.
- When you write only part of a sprite (for example one frame, or a region),
  the rest is not touched.
- The keys are always checked against the palette of the **sprite**. The
  `apply_grid` tool never ignores the `palette:` line: another existing palette
  is `palette_mismatch` and an unknown one is `palette_not_found`. The app
  ignores that line.
- Frames can come in any order. A frame number that is not a number, is `0`
  or is written twice is `bad_frame`.

A grid can add a color with its legend. Here `Q` is new, so it is added to
the palette, and the frame uses it:

```
legend
  Q teal #1FB5A8

frame 1
Q.
.Q
```

### Writing a region

A grid can be written at a position inside a bigger sprite. The tool
`apply_grid` takes `x` and `y` (default `0`, `0`). The region must fit inside
the sprite. Use `.` for transparent pixels you want to **erase** when you write a whole
frame. When you write a region (the AI tool `apply_grid` with `x` and `y`), a
`.` **keeps** the pixel that is there by default (`skip_transparent` is true);
send `skip_transparent: false` to erase with it. Every AI write that erased
opaque pixels reports how many in `erased`.

Without `x` and `y` the grid is a **whole frame**: it must have the size of
the sprite. With `x` and `y` (even `0` and `0`) it is a region, so it can be
smaller; the size of the region is the number of rows and the length of the
first row. This grid, written at `x` 3 and `y` 5, fills a 2x2 area:

```
frame 1
RR
RR
```

The **Text** view always writes whole frames.

## Errors

When a grid is not valid, nothing is applied and the error says exactly what
is wrong, with the line number and the row number.

| Code | Message example |
| --- | --- |
| `row_length` | `line 15 (row 7 of frame 1): 7 columns, expected 8` |
| `row_count` | `frame 1 has 7 rows, expected 8` |
| `unknown_key` | `line 12 (row 4, column 3): key "X" is not in palette dungeon16` |
| `no_frame` | `no "frame" block found` |
| `bad_frame` | `frame 5 does not exist; the sprite has 4 frames` |
| `too_big` | `grid is 600x40; maximum is 512x512` |
| `region_out` | `region at (10, 2) of size 8x8 does not fit in 16x8` |
| `palette_full` | `line 2: the legend adds the color "Q" but palette main already has 93 colors` |
| `bad_legend` | `line 3: "#1FB5" is not a color, expected #RRGGBB` |

Every error has a `code`, a `message` and, when they apply, the `line`, the
`row` and the `column` (from 1). **All** the errors are reported together, so
an AI can fix them in one try, and when there is any error **nothing** is
applied: not the pixels and not the legend colors. Applying a valid grid is
one undo step, and a grid that changes nothing makes no undo step.

## Using the format

- **In the app:** the **Text** view of the editor shows and edits this text of
  the current frame, with a live preview and the same errors. The line with a
  problem is marked, the preview keeps the last valid state and **Apply** is
  off until the text is valid. **Copy** and **Paste** work with the whole
  text. The error messages are always in English, like the keywords.
- **From an AI:** the tools `get_sprite` and `apply_grid` read and write it.
  See [AI and MCP](ai-and-mcp.md).
- **On disk:** the sprite files store each frame as an array of these row
  strings. See [Architecture](architecture.md#the-project-on-disk).

## Tips for writing a grid

- Draw the outline first, then fill the inside.
- Keep rows aligned (use a monospace font).
- Read the palette first and reuse the same keys everywhere.
- For big sprites, write one region at a time.

## A part of a sprite

`get_sprite` can return only a rectangle (`region`) and can leave out the
legend (`include_palette` false). The text is still a grid: it has the line
`palette: <name>`, optionally a legend, and `frame N` with the rows of the
region only. To write it back, send the rows to `apply_grid` with `x` and `y`
set to the top left corner of the region.
