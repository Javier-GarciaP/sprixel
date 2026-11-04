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
      "index": 0,
      "frame_number": 1,
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
      },
      "anchors": []
    },
    {
      "name": "hero_walk_1",
      "sprite": "hero",
      "index": 1,
      "frame_number": 1,
      "frame": {
        "x": 3,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "duration": 83,
      "pivot": {
        "x": 0,
        "y": 1
      },
      "anchors": [
        {
          "name": "feet",
          "x": 0,
          "y": 1
        }
      ]
    },
    {
      "name": "hero_walk_2",
      "sprite": "hero",
      "index": 2,
      "frame_number": 2,
      "frame": {
        "x": 6,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "duration": 83,
      "pivot": {
        "x": 0,
        "y": 1
      },
      "anchors": [
        {
          "name": "feet",
          "x": 0,
          "y": 1
        },
        {
          "name": "hand",
          "x": 1,
          "y": 0
        }
      ]
    },
    {
      "name": "hero_3",
      "sprite": "hero",
      "index": 3,
      "frame_number": 3,
      "frame": {
        "x": 9,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "duration": 250,
      "pivot": {
        "x": 1,
        "y": 1
      },
      "anchors": [
        {
          "name": "Feet",
          "x": 1,
          "y": 1
        }
      ]
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
| `frames[].index` | The position of the frame in `frames`, **counted from 0**. It is the number that `animations[].from` and `to` use. |
| `frames[].frame_number` | The number of the frame inside its sprite, **counted from 1** (the number the editor and the tools show). |
| `frames[].frame` | The rectangle of the frame in the PNG, in pixels (`x`, `y`, `w`, `h`). |
| `frames[].duration` | How long the frame lasts, in milliseconds. |
| `frames[].pivot` | The pivot point in pixels, from the top left corner of the frame: the anchor named `feet` of the frame, else its first anchor (by name), else `0, 0`. Scaled like the anchors. |
| `frames[].anchors` | The anchor points of the frame (see [Anchors](#anchors)): a list of `name`, `x`, `y`. Empty when there are none. |
| `animations[].name` | The name of the animation. |
| `animations[].sprite` | The sprite it belongs to. Two sprites can have an animation with the same name. |
| `animations[].from`, `to` | The first and last frame, as positions in `frames` **counted from 0**, both included. |
| `animations[].mode` | `loop`, `once` or `pingpong`. |
| `animations[].fps` | Frames per second, 1 to 60. |

### Aseprite JSON

The JSON layout is chosen with the option **`json_format`** (the tool
`export` argument and the "JSON format" selector of the Export screen):

- `sprixel` (default): the layout above.
- `aseprite`: the **array form** of the Aseprite sprite sheet JSON. The PNG is
  the same. It is what the Aseprite importers of Godot and Unity read. It is
  the same project and options as the sample above:

```json aseprite
{
  "frames": [
    {
      "filename": "blob_1",
      "frame": {
        "x": 0,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "rotated": false,
      "trimmed": false,
      "spriteSourceSize": {
        "x": 0,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "sourceSize": {
        "w": 2,
        "h": 2
      },
      "duration": 100
    },
    {
      "filename": "hero_walk_1",
      "frame": {
        "x": 3,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "rotated": false,
      "trimmed": false,
      "spriteSourceSize": {
        "x": 0,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "sourceSize": {
        "w": 2,
        "h": 2
      },
      "duration": 83
    },
    {
      "filename": "hero_walk_2",
      "frame": {
        "x": 6,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "rotated": false,
      "trimmed": false,
      "spriteSourceSize": {
        "x": 0,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "sourceSize": {
        "w": 2,
        "h": 2
      },
      "duration": 83
    },
    {
      "filename": "hero_3",
      "frame": {
        "x": 9,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "rotated": false,
      "trimmed": false,
      "spriteSourceSize": {
        "x": 0,
        "y": 0,
        "w": 2,
        "h": 2
      },
      "sourceSize": {
        "w": 2,
        "h": 2
      },
      "duration": 250
    }
  ],
  "meta": {
    "app": "Sprixel",
    "version": "1",
    "image": "kit.png",
    "format": "RGBA8888",
    "size": {
      "w": 11,
      "h": 2
    },
    "scale": "1",
    "frameTags": [
      {
        "name": "hero_walk",
        "from": 1,
        "to": 2,
        "direction": "forward"
      }
    ]
  }
}
```

| Field | Meaning |
| --- | --- |
| `frames[].filename` | The name of the frame, as `frames[].name` above. |
| `frames[].frame` | The rectangle of the frame in the PNG. |
| `frames[].rotated`, `trimmed` | Always `false`. |
| `frames[].spriteSourceSize`, `sourceSize` | The size of the frame in pixels (with the scale): `x`, `y` are `0`. |
| `frames[].duration` | Milliseconds, as above. |
| `meta.image`, `meta.size` | The PNG name and size. |
| `meta.scale` | The scale **as text** (`"1"`), as Aseprite writes it. |
| `meta.format` | Always `RGBA8888`. |
| `meta.frameTags` | The animations: `name`, `from`, `to` (positions in `frames` from 0, as in the Sprixel layout) and `direction`. |

`direction` is `pingpong` for a `pingpong` animation and `forward` for `loop`
and `once` (Aseprite has no loop flag: the engine decides). When the export has
more than one sprite, the tag name is `sprite_animation` (for example
`hero_walk`) so that two sprites can have a `walk`. The Sprixel anchors and
the pivot are not in this layout (Aseprite has no such field): use the Sprixel
layout if you need them.

### Anchors

An anchor is a named point of a sprite (`feet`, `hand`, `muzzle`) that the
person or an AI set in the editor. `x` and `y` are in pixels of the sheet, from
the top left corner of the frame, so with the scale 2x an anchor at pixel
3,5 is written as `6`, `10`. A frame lists the anchors of its sprite; an
anchor set for one frame replaces the one of the whole sprite with the same
name. The list is sorted by name.

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

## Using the JSON in a game engine

The JSON has what an engine needs: the rectangle of each frame, how long it
lasts, its anchor points and the animations. The mapping below is the
intended one. It was **not** tested with the engines themselves; it is a
guide to write a small import script.

Take the example above (the sprite `hero`, animation `walk`, positions 1 to 2 in
`frames`, 12 fps, `loop`).

### Godot 4 (`SpriteFrames`)

| Sprixel | Godot |
| --- | --- |
| `meta.image` | One `Texture2D`. |
| `frames[].frame` | An `AtlasTexture` with `atlas` = the texture and `region` = `Rect2(x, y, w, h)`. |
| `animations[]` | One animation of the `SpriteFrames` per entry. Use `sprite + "_" + name` as the name when several sprites share the sheet. |
| `animations[].fps` | `set_animation_speed(name, fps)`. |
| `animations[].mode` | `loop` is `set_animation_loop(name, true)`; `once` is `false`. `pingpong` does not exist in `SpriteFrames`: add the frames again in reverse order (without the first and last) and loop. |
| `frames[].duration` | The `duration` argument of `add_frame`. In Godot it is **relative**: a frame lasts `duration / fps` seconds, so use `duration_ms / (1000.0 / fps)` (1.0 is a normal frame). |
| `frames[].anchors` | A `Marker2D` child with that name. Its `position` is `Vector2(x, y)` minus half the frame size when the `AnimatedSprite2D` is `centered`. To follow frame anchors, change the marker in the `frame_changed` signal. |

```gdscript
var data = JSON.parse_string(FileAccess.get_file_as_string("res://kit.json"))
var texture: Texture2D = load("res://" + data.meta.image)
var frames := SpriteFrames.new()
for anim in data.animations:
    var name = anim.sprite + "_" + anim.name
    frames.add_animation(name)
    frames.set_animation_speed(name, anim.fps)
    frames.set_animation_loop(name, anim.mode == "loop")
    for i in range(anim.from, anim.to + 1):
        var f = data.frames[i]
        var atlas := AtlasTexture.new()
        atlas.atlas = texture
        atlas.region = Rect2(f.frame.x, f.frame.y, f.frame.w, f.frame.h)
        frames.add_frame(name, atlas, f.duration / (1000.0 / anim.fps))
```

### Unity

| Sprixel | Unity |
| --- | --- |
| `meta.image` | A texture with Sprite Mode **Multiple** and **Filter Mode: Point (no filter)**. |
| `frames[].frame` | One sprite rectangle per frame. Unity counts `y` from the **bottom**: `y_unity = meta.size.h - y - h`. |
| `frames[].anchors` | A custom pivot (normalized: `x / w`, `1 - y / h`), or a child `Transform` per anchor at `((x - w / 2) / pixelsPerUnit, (h / 2 - y) / pixelsPerUnit)`. |
| `animations[]` | One `AnimationClip` per entry with a sprite curve (`ObjectReferenceKeyframe`). |
| `frames[].duration` | The time of each keyframe is the sum of the durations before it, divided by 1000. |
| `animations[].mode` | `loop` is `clip.wrapMode = Loop` (`loopTime` in the clip settings); `once` is `Once`; `pingpong` is `PingPong`. |
| `animations[].fps` | Not needed: the durations already hold the time. Set `clip.frameRate` to `fps` if you like. |

```csharp
// Editor script: build the keyframes of one animation.
var keys = new List<ObjectReferenceKeyframe>();
float time = 0f;
for (int i = anim.from; i <= anim.to; i++) {
    keys.Add(new ObjectReferenceKeyframe { time = time, value = sprites[i] });
    time += data.frames[i].duration / 1000f;
}
var binding = EditorCurveBinding.PPtrCurve("", typeof(SpriteRenderer), "m_Sprite");
AnimationUtility.SetObjectReferenceCurve(clip, binding, keys.ToArray());
```

In both engines `frames[].duration` is already final: a frame without its own
duration uses `1000 / fps` (see [Durations](#durations)).

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
