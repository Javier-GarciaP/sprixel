# Text grid format

> **Status: planned.** This is the version 1 format. The full rules, with
> tests, are in the `text-grid-format` spec.

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
| `legend` block | No | Lines of `<key> <name> [#hex]`. A line with a `#hex` for a key that is **not** in the palette adds that color when the grid is applied. For a key that already exists it is only informational: a grid never changes an existing color. |
| `frame <n>` + rows | Yes, at least one | The pixels of frame number `n`. Frames are numbered from 1. |

- Keywords (`palette`, `legend`, `frame`) are always **English**, even when the
  interface is in Spanish.
- Blank lines are ignored. A line that starts with `#` after optional spaces
  is a comment.
- When you write only part of a sprite (for example one frame, or a region),
  the rest is not touched.

### Writing a region

A grid can be written at a position inside a bigger sprite. The tool
`apply_grid` takes `x` and `y` (default `0`, `0`). The region must fit inside
the sprite. Use `.` for transparent pixels you want to **erase**; use the
option `skip_transparent` to leave existing pixels instead.

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

## Using the format

- **In the app:** the **Text** view of the editor shows and edits this text,
  with a live preview and the same errors. **Copy** and **Paste** work with it.
- **From an AI:** the tools `get_sprite` and `apply_grid` read and write it.
  See [AI and MCP](ai-and-mcp.md).
- **On disk:** the sprite files store each frame as an array of these row
  strings. See [Architecture](architecture.md#the-project-on-disk).

## Tips for writing a grid

- Draw the outline first, then fill the inside.
- Keep rows aligned (use a monospace font).
- Read the palette first and reuse the same keys everywhere.
- For big sprites, write one region at a time.
