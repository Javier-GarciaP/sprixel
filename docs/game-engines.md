# From a drawing to a game engine

This page is a step by step guide: you draw a sprite in Sprixel and use it in
**Godot 4** or **Unity**. It explains which export options to choose and where
the `feet` anchor goes. The files themselves are defined in the
[export format](export-format.md).

> The steps were **not** tested with the engines themselves (only the files
> Sprixel writes are tested). Treat the engine parts as a guide.

## 1. Prepare the sprite

1. **Draw the frames** and give each animation a name with a tag (for example
   `walk`, frames 1 to 4, 8 fps). A frame that is in no tag is a still image.
2. **Set the times** if a frame must last longer (a hit, a pause). A frame
   with no time of its own lasts `1000 / fps` of its animation.
3. **Set the `feet` anchor**: the pixel where the character stands, usually
   the bottom center of the body. Put it on the **same pixel in every frame**
   (set it once for the whole sprite, and a frame anchor only when the body
   really moves). The export uses it as the `pivot` of every frame. An AI sets
   it with the tool `set_anchor`; if there is no `feet` anchor, the first
   anchor (by name) is the pivot, and with no anchors the pivot is `0, 0`.
4. If the canvas is too small or too big, use **Change canvas…** (or the tool
   `resize_sprite`). The anchors move with the drawing.

| Tools you will use | What for |
| --- | --- |
| `create_tag` | Names an animation (frames, fps, mode). |
| `set_frame_duration` | The time of one frame, in milliseconds. |
| `set_anchor` | The `feet` anchor and others (`hand`, `muzzle`). |
| `resize_sprite` | Change the canvas size. |
| `export` | Writes the files for the engine. |

## 2. Export

Open the **Export** tab, choose what to export, **Sheet + JSON** and the
options, and press **Export**. The files go to `assets/sprites` inside the
project folder (you can change it). The tool `export` takes the same options.

| Export options | Value |
| --- | --- |
| `scope` | What to export: `project`, `folder` or `sprites`. |
| `folder` | For the scope `folder`: the folder path, for example `Characters`. |
| `sprites` | For the scope `sprites`: the sprite names or paths. |
| `format` | `sheet` writes a PNG sheet and a JSON file; `png` writes one PNG per frame (no JSON, no pivot). Use `sheet` for engines. |
| `scale` | `1`, `2`, `4` or `8`. Use `1` and let the engine scale the pixels; the `pivot` and anchors are scaled with it. |
| `spacing` | Pixels between frames, 0 to 16. Use 1 or more so a filter does not mix neighbors. |
| `json_format` | `sprixel` (default) or `aseprite`. See below. |
| `naming` | The frame names, with `{sprite}`, `{animation}` and `{frame}`. |
| `output` | The folder inside the project. Default `assets/sprites`. |

Which `json_format` to choose:

- **`sprixel`**: has everything: `pivot`, `anchors`, `index` (position in
  `frames`, from 0) and `frame_number` (number in the sprite, from 1). Use it
  with the small scripts below.
- **`aseprite`**: the Aseprite sprite sheet layout, for tools that already
  read Aseprite JSON (array form). It has the frames, times and animation
  tags, but **no pivot and no anchors**.

You get, for example, `personajes.png` and `personajes.json` in
`assets/sprites`.

## 3. Godot 4

1. Copy the PNG and the JSON into your Godot project (for example under
   `res://sprites/`).
2. Turn the texture filter off for pixel art: in *Project Settings > Rendering
   > Textures* set the **Default Texture Filter** to **Nearest**. Without this
   the pixels look blurry.
3. Build the `SpriteFrames` from the JSON with the script in the
   [export format](export-format.md#godot-4-spriteframes) (one animation per
   entry of `animations`, one `AtlasTexture` per frame).
4. Put the feet on the node origin. On the `AnimatedSprite2D` set **Centered**
   off and `offset = Vector2(-pivot.x, -pivot.y)`, using the `pivot` of the
   frame (it is the same in every frame when you kept `feet` on one pixel).
   Now the node position is where the character stands, which is what
   collisions and y-sorting need.
5. For other anchors (`hand`) use a `Marker2D` child, as the table in the
   export format says.

If you used `json_format` `aseprite`, import it with a tool that reads the
Aseprite sheet JSON, and set the offset by hand as in step 4.

## 4. Unity

1. Copy the PNG and the JSON into `Assets/`.
2. Select the PNG: **Texture Type** Sprite (2D and UI), **Sprite Mode**
   Multiple, **Filter Mode** Point (no filter), **Compression** None. Choose
   **Pixels Per Unit** equal to the size of one tile in your game (16 for a
   16x16 grid).
3. Cut the sprites with the rectangles of the JSON. Unity counts `y` from the
   bottom: `y_unity = meta.size.h - y - h`.
4. Set the **Pivot** of every sprite to **Custom** with the normalized value
   `x = pivot.x / frame.w` and `y = 1 - pivot.y / frame.h`, using the `pivot`
   of the frame. The `feet` pixel is now the position of the object.
5. Make one `AnimationClip` per animation with the script in the
   [export format](export-format.md#unity), using `duration` for the times.
6. If you chose `json_format` `aseprite`, use an importer that reads the
   Aseprite sheet JSON, and set the pivot by hand as in step 4.

## Check list

- The `feet` anchor is on the same pixel in all frames.
- Filter is **Nearest** or **Point** in the engine.
- `scale` is `1` in the export; scale in the engine.
- The animation names in the engine are the tag names (`walk`, `idle`).
