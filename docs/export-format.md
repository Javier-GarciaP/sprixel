# Export format

Sprixel exports a sprite sheet (one PNG) with a JSON file that describes it,
or one PNG file per frame. This page defines both. The JSON layout is a public
format: it has a version (`meta.version`, now `1`) and a change that can break
a reader will raise it.

The idea follows the sprite sheet JSON of Aseprite (frames with a rectangle
and a duration, and animations), so existing importers can be adapted. The
exact layout is the one below. No engine is claimed to read it as it is.

## Sheet + JSON

Two files are written, with the same base name. The base name is the name of
the exported folder, of the only sprite, or of the project, in lower case (for
example `personajes.png` and `personajes.json`).

### The PNG

- The frames are packed in rows, left to right, at most 8 frames in a row.
  Every frame goes at the place the JSON says. A row is as tall as its tallest
  frame.
- The space between frames and between rows is the **spacing** option
  (0 to 16 pixels, multiplied by the scale). There is no space at the edges.
- Pixels outside the frames, and transparent pixels, are fully transparent.
- With the scale 2x, 4x or 8x every pixel becomes a block of 2, 4 or 8 pixels,
  with no blending (nearest-neighbor).
- If a sprite has several layers they are stacked, the last layer on top.

### The JSON

```json
{
  "meta": {
    "app": "Sprixel",
    "version": 1,
    "image": "kit.png",
    "size": {
      "w": 11,
      "h": 2
    },
    "scale": 1
  },
  "frames": [
    {
      "name": "blob_1",
      "sprite": "blob",
      "frame": {
        "x": 0,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "duration": 100,
      "pivot": {
        "x": 0,
        "y": 0
      }
    },
    {
      "name": "hero_walk_1",
      "sprite": "hero",
      "frame": {
        "x": 3,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "duration": 83,
      "pivot": {
        "x": 0,
        "y": 0
      }
    },
    {
      "name": "hero_walk_2",
      "sprite": "hero",
      "frame": {
        "x": 6,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "duration": 83,
      "pivot": {
        "x": 0,
        "y": 0
      }
    },
    {
      "name": "hero_3",
      "sprite": "hero",
      "frame": {
        "x": 9,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "duration": 250,
      "pivot": {
        "x": 0,
        "y": 0
      }
    }
  ],
  "animations": [
    {
      "name": "walk",
      "sprite": "hero",
      "from": 1,
      "to": 2,
      "mode": "loop",
      "fps": 12
    }
  ]
}
```

| Field | Meaning |
| --- | --- |
| `meta.app` | Always `Sprixel`. |
| `meta.version` | The version of this layout. |
| `meta.image` | The name of the PNG, in the same folder. |
| `meta.size` | The size of the PNG in pixels. |
| `meta.scale` | The scale used (1, 2, 4 or 8). |
| `frames[].name` | The name of the frame (see [Names](#names)). |
| `frames[].sprite` | The name of the sprite the frame belongs to. |
| `frames[].frame` | The rectangle of the frame in the PNG, in pixels (`x`, `y`, `w`, `h`). |
| `frames[].duration` | How long the frame lasts, in milliseconds. |
| `frames[].pivot` | The pivot point in pixels, from the top left corner of the frame. Now always `0, 0`. |
| `animations[].name` | The name of the animation. |
| `animations[].sprite` | The sprite it belongs to. Two sprites can have an animation with the same name. |
| `animations[].from`, `to` | The first and last frame, as positions in `frames` **counted from 0**, both included. |
| `animations[].mode` | `loop`, `once` or `pingpong`. |
| `animations[].fps` | Frames per second, 1 to 60. |

### Order

The same project with the same options always gives byte-identical files.
Frames are ordered by sprite name (ignoring case), then by frame number. The
animations follow the same order: by sprite, and inside a sprite as they are
saved in the project.

### Durations

The duration of a frame is the one the person set for it. A frame that has none
of its own (100 ms is the value of a new frame) and is inside an animation
lasts `1000 / fps` of that animation, rounded (12 fps gives 83 ms). A frame in
more than one animation uses the first one.

## PNG files

One PNG per frame, with the scale applied and no spacing. The file name is the
naming pattern plus `.png`.

### Names

The default pattern is `{sprite}_{animation}_{frame}`, so `goblin_walk_1.png`
to `goblin_walk_4.png` for the animation `walk` of the sprite `goblin`.

- `{sprite}` is the sprite name, `{animation}` the name of the animation the
  frame is in, and `{frame}` its number **inside that animation**, from 1.
- A frame that is in no animation has an empty `{animation}` (the extra `_` is
  removed) and its number in the sprite, so `goblin_1.png`.
- If two frames get the same name (two sprites with the same name in different
  folders), the later ones get `_2`, `_3`... Names are compared ignoring case.
- A pattern cannot be empty or have `/`, `\` or `..`.

The same names are used for `frames[].name` in the JSON.

## Where the files go

Only inside the project folder (`assets/sprites` by default). The folder can
not be absolute, can not use `..`, can not go through a link that points
outside, and can not be `.sprixel`. Files with the same names from an earlier
export are replaced; other files are not touched. The files are written to a
temporary folder first and moved when all are ready, so a failed export leaves
no partial files.
