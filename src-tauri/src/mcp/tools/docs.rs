//! The long texts that tools show to the AI. They are the instructions an AI
//! has, so they explain the text grid format and the palette keys, and how to
//! make pixel art that looks good (the guides of `get_guide`).

use super::args;
use crate::mcp::{Need, Output, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

/// The description of `apply_grid`.
pub const APPLY_GRID: &str = "\
Writes a text grid into a sprite. One character is one pixel and each character is a palette key; `.` is always transparent. \
Read the palette first (get_palette) and use only its keys.\n\
\n\
Format (keywords are English):\n\
palette: main            (optional; it must be the palette of the sprite)\n\
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
A `palette:` line is never ignored: another existing palette is `palette_mismatch` (use `set_sprite_palette` or drop the line) and an unknown one is `palette_not_found`. \
If a legend line gives an existing key a different color, the old color stays, the pixels move to a free key and the answer has `remapped` and `warnings`. \
The answer has the new `revision`; pass `base_revision` (the revision you last read) to fail with `stale_revision` instead of overwriting newer changes. The person can undo the call. The answer has an image of the edited frame: LOOK at it and fix what is wrong (send `preview` false to skip it).";

/// The steps every agent follows. The same text is in the server instructions
/// and in the Claude Code skill (a test checks both). A macro, so `concat!`
/// can use it.
macro_rules! workflow {
    () => {
        "\
1. `get_project`: read the project. If it says no project is open, call `list_projects` and then `open_project`.\n\
2. `get_guide` with the topic `design` (and `animation` for a sprite that moves). Follow it.\n\
3. `get_palette`: read the colors. Draw with the keys of the palette only; a `legend` line in `apply_grid` adds a new color. If a legend line gives a used key another color, the pixels move to a free key and the answer says so.\n\
4. `create_sprite` for a new sprite (`own_palette: true` gives it a palette of its own), or `get_sprite` to read an existing one.\n\
5. Draw: `apply_grid` for a whole frame or a region, `set_pixels` and `draw_shape` for details and fixes.\n\
6. LOOK at the image every write returns (or call `render_preview`). If something looks wrong, fix it and look again. Do not say you are done before you have seen the result and it looks right."
    };
}

/// The sentence that lists the topics. The same text is in the server
/// instructions, the Claude Code skill and `docs/ai-and-mcp.md` (a test checks
/// all three). A macro, so `concat!` can use it.
macro_rules! guide_list {
    () => {
        "Guides (`get_guide`, argument `topic`): `design`, `animation`, `palette`, `large-sprites`, `faces-and-animals`, `walk-and-attack-cycles`, `game-ready` and `tools`."
    };
}

/// The grid of the worked example: a 16x16 slime, an idle frame and a jump
/// frame, with the colors in a `legend`.
macro_rules! example_grid {
    () => {
        "\
legend\n\
\x20\x20K outline #1E3A2B\n\
\x20\x20D shade #3F9B4F\n\
\x20\x20B base #6FD36B\n\
\x20\x20L light #B8F2A6\n\
\x20\x20W eye #FFFFFF\n\
frame 1\n\
................\n\
................\n\
................\n\
................\n\
................\n\
.....KKKKKK.....\n\
....KLLLBBBK....\n\
...KLLBBBBBBK...\n\
..KLLBBBBBBBBK..\n\
..KLBBWKBBWKBK..\n\
.KBBBBWKBBWKBBK.\n\
.KBBBBBBBBBBBBK.\n\
.KBBBBBBBBBBDDK.\n\
.KBBBBBBBBDDDDK.\n\
..KDDDDDDDDDDK..\n\
...KKKKKKKKKK...\n\
frame 2\n\
................\n\
................\n\
................\n\
......KKKK......\n\
.....KLLBBK.....\n\
....KLLBBBBK....\n\
....KLBBBBBK....\n\
...KLBWKBWKBK...\n\
...KBBWKBWKBK...\n\
...KBBBBBBBBK...\n\
...KBBBBBBDDK...\n\
....KBBBBDDK....\n\
....KDDDDDDK....\n\
.....KKKKKK.....\n\
................\n\
................\n"
    };
}

/// The grid of the worked example, for the tests.
pub const EXAMPLE_GRID: &str = example_grid!();

/// The steps of the workflow (see `workflow!`).
pub const WORKFLOW: &str = workflow!();

/// A worked example an agent can copy: the calls, in order, for a 16x16 sprite
/// with an idle frame and an action frame. The same text is in the skill.
macro_rules! example {
    () => {
        concat!(
            "Worked example: a 16x16 slime with an idle frame and a jump frame.\n\
1. `create_sprite` with name slime, width 16 and height 16.\n\
2. `add_frame` with name slime (now it has 2 frames).\n\
3. `apply_grid` with name slime and this grid. The `legend` adds the colors; frame 1 is the idle pose and frame 2 is the jump:\n\
\n",
            example_grid!(),
            "\n\
4. LOOK at the image that came back. Fix what is wrong with `set_pixels` (a few pixels) or `apply_grid` with `x` and `y` (a region) and look again.\n\
5. `create_tag` with sprite slime, name jump, from 1, to 2, fps 4: the two frames become an animation."
        )
    };
}

/// The worked example (see `example!`).
pub const EXAMPLE: &str = example!();

/// What the server tells every client when it connects (`instructions` in the
/// answer of `initialize`). A client may put it in every prompt, so it has the
/// workflow and one worked example, and sends the rest to `get_guide`.
pub const INSTRUCTIONS: &str = concat!(
    "Sprixel is a pixel art editor for game sprites. You draw on the person's real canvas while they watch: their screen shows a cursor with your name that draws your pixels.\n\
\n\
Workflow:\n",
    workflow!(),
    "\n\
\n",
    example!(),
    "\n\
\n",
    guide_list!(),
    "\n\
\n\
Pixels: x and y count from 0 at the top left; frames count from 1; a color is a one-character palette key; `.` is always transparent. \
A text grid is one character per pixel, one row per line; the exact format is in the description of `apply_grid`. \
Small sprites (up to about 32x32) are drawn most reliably. Every pixel write returns an image of the edited frame; send `preview` false to skip it.\n\
\n\
Every write returns the new `revision`. Pass it back as `base_revision` and the call fails with `stale_revision` if the project changed. \
Every error has a `code` and a `message` that says how to fix it, and each call is applied whole or not at all. \
If a call fails with `permission_denied`, ask the person to change the permission in the Agents tab. \
The person can stop you (`stopped_by_user`) and undo your last action."
);

/// The topics of `get_guide`, in the order they are listed.
pub const GUIDE_TOPICS: [&str; 8] =
    ["design", "animation", "palette", "large-sprites", "faces-and-animals", "walk-and-attack-cycles", "game-ready", "tools"];

/// The sentence that lists the topics (see `guide_list!`).
pub const GUIDE_LIST: &str = guide_list!();

/// How to design a sprite so it looks good. The same text is in the Claude
/// Code skill (a test checks it).
pub const DESIGN_GUIDE: &str = "\
Design guide: how to make a sprite that looks good\n\
\n\
1. Small palette with ramps. Use 4 to 16 colors. For each material make a ramp of 2 or 3 tones (dark, base, light). Reuse the same colors everywhere. Add a color only when you need it.\n\
2. One light direction. Choose it first (for example from the top left) and keep it in the whole sprite: light tones on the side that faces the light, dark tones on the other side.\n\
3. Silhouette first. Block the whole shape with flat colors before any detail. If the silhouette is not clear, shading will not fix it.\n\
4. Flat fills. Fill big areas with one tone. No noise, no dithering, unless the person asks for it.\n\
5. Two or three tones of shading per material: base, shadow and highlight. Put them in small clean shapes, not in scattered pixels.\n\
6. Selective outline. Outline the outside of the shape with a dark tone of the material (not black everywhere). Inside the shape use no lines, or lines lighter than the outer outline.\n\
7. No stray pixels. Every pixel must belong to a shape. Remove single pixels that touch nothing, and keep lines clean, with steady steps.\n\
8. Look at the result. Every pixel write (`apply_grid`, `set_pixels`, `draw_shape`) returns an image of the edited frame. LOOK at it; if you have no image, call `render_preview`. If something looks wrong, fix it and look again. Do not say you are done before you have seen the image and it looks right.\n\
9. Big sprites (more than 32x32): work region by region. Write one part at a time (head, body, legs) with `apply_grid` and `x` and `y`, look at the image after each part, then go on. Read a part with `get_sprite` (`frame`, `region`, `include_palette` false) instead of the whole sprite.\n\
10. Edit without redrawing. `transform_region` flips, mirrors, moves or rotates a region; `copy_region` copies a region to another place, frame or sprite; `symmetry` on `set_pixels`, `draw_shape` and `apply_grid` draws both sides at once; `only_over` changes only the pixels that have one key (shade a color without touching the outline). `import_reference` draws a PNG from the folder `references/` of the project, reduced to the palette.\n\
11. Let the server finish the sprite. When the shape is done, call `auto_outline` (the outline) and `auto_shade` (lit and shadow tones from one light direction), then `analyze_sprite` to find loose pixels, gaps in the outline, unused colors and, for animations, frames where the feet (`ground_kept`) moved. Fix what it reports.";

/// How to animate. Also available as the topic `animation`.
pub const ANIMATION_GUIDE: &str = "\
Animation guide: how to make a sprite move\n\
\n\
1. Key poses first. Draw the main poses as whole frames before anything else (an idle needs 2 frames; a walk needs contact and passing poses). Add the in-between frames after.\n\
2. Same palette in every frame. Do not add new colors for one frame. Keep the same light direction and the same outline.\n\
3. Keep the size and the ground line. The body must not grow or jump by accident between frames: feet stay on the same row unless the pose is a jump.\n\
4. Change little. Start a new frame with `add_frame` and `copy_of`, then change only what moves, with `set_pixels` or `apply_grid` and `x` and `y`.\n\
5. The loop closes. The last frame must lead back to the first without a jump. Then make the animation with `create_tag` (mode loop, 6 to 12 fps).\n\
6. Look at the whole animation with `render_frames` (all frames in one image, `labels` true shows the numbers) and at the movement with `render_onion` (earlier frames tinted red under the last one). Fix what looks wrong before you say you are done. All previews take a `background` (checker, light, dark or transparent).\n\
7. Game data. Give a frame its own time with `set_frame_duration` (milliseconds) when it must last longer, like a hit or a pause. Mark the points the game needs (feet, hand) with `set_anchor`, and keep them on the same pixel when the pose does not move.";

/// How to choose and build the colors. Also available as the topic `palette`.
pub const PALETTE_GUIDE: &str = "\
Palette guide: how to choose colors\n\
\n\
1. Read the palette first with `get_palette`. A key is one character; `.` is always transparent.\n\
2. Keep it small: 8 to 16 colors is enough for most sprites (the maximum is 93).\n\
3. Build ramps. For each hue make 3 tones: shadow, base and light. Make the shadow a little cooler and the light a little warmer, not just darker and lighter.\n\
4. Avoid pure black and pure white. Use a very dark tone of the material for the outline.\n\
5. Add colors with a `legend` line in `apply_grid` (`<key> <name> #RRGGBB`) or with `set_palette_color`. Give related colors related keys, for example `G` base green, `D` dark green, `L` light green.\n\
6. Changing a color with `set_palette_color` recolors every pixel that uses its key, so you can fix a tone at the end without redrawing.\n\
7. Make a ramp for you: `generate_ramp` adds 3 to 9 tones of one color (use `hue_shift` for cooler shadows and warmer lights). `remove_unused_colors` cleans a palette at the end.\n\
8. A sprite can have its own palette: `create_sprite` with `own_palette: true`, or `create_palette` and then `set_sprite_palette`. Check the palette of a sprite with `get_palette` before you write a grid for it.";

/// How to draw sprites bigger than 32x32. Also available as the topic `large-sprites`.
pub const LARGE_SPRITES_GUIDE: &str = "\
Large sprites guide: how to draw more than 32x32 without losing count\n\
\n\
1. Plan the parts. Before you draw, list the parts (head, body, arms, legs) and give each one a box: `x`, `y`, `width`, `height`. Parts must not overlap by accident.\n\
2. One part at a time. Write each part with `apply_grid` and `x` and `y`, with at most about 24 rows. LOOK at the image after each part and fix it before you go on. A row with a wrong length is refused with `row_length`, so count the columns of the first row and copy that length.\n\
3. Draw half, mirror the rest. For a front view write only the left half and send `symmetry` horizontal: the mirror line is the center of the sprite, so the half must end at the center column. To mirror a finished half, use `transform_region` with the operation mirror, or `copy_region`.\n\
4. Start from a picture. Put a PNG in the folder `references/` of the project and call `import_reference` (`width` and `height` scale it down, `max_colors` adds colors to the palette). It is a rough start: clean it with `apply_grid` and `set_pixels`.\n\
5. Read only what you need. `get_sprite` with `frame`, `region` and `include_palette` false returns one part, still as a valid grid. Do not read the whole sprite after every change.\n\
6. Let the server finish. When the parts are done, call `auto_outline` once, then `auto_shade` for each material with the same light direction, then `analyze_sprite` with `outline_key`. Fix the loose pixels and the outline gaps it lists and call it again until both counts are 0.\n\
7. Keep your changes safe. Pass `base_revision` on every write; if the person edits at the same time you get `stale_revision` and you read again instead of overwriting.";

/// How to draw faces and animals. Also available as the topic `faces-and-animals`.
pub const FACES_GUIDE: &str = "\
Faces and animals guide: heads, eyes and bodies that read well\n\
\n\
1. Silhouette first. An animal is known by its outline: ears, snout, tail, legs. Block the shape with one flat color and look at it with `render_preview`. If you cannot tell what it is, change the shape, not the details.\n\
2. Big head, small body. In a small sprite the head is about half of the height. It reads as cute and the face has room.\n\
3. Eyes are the face. In 16x16 use 1 or 2 pixels wide; in 32x32 use 2x2. Put them just below the middle of the head and leave a gap of at least one eye between them. Use a dark tone (not the outline color) and one light pixel as a highlight, in the corner the light comes from.\n\
4. Use symmetry for a front face. Write one half with `apply_grid` (`x` and `y`) and `symmetry` horizontal, and draw the eyes with `set_pixels` and `symmetry`, so both are equal. A side view has no symmetry: face the animal to the right.\n\
5. Less is more. A nose is 1 or 2 pixels on the center column (2 for an even width). A mouth is a dark line of 1 to 3 pixels under it. If the face looks busy, remove a feature.\n\
6. Animal parts. Ears are small triangles of 2 to 4 pixels (pointed for a cat, round for a bear). The muzzle is a lighter patch under the nose. Legs and tails are 2 pixels wide at 16x16.\n\
7. Fur is 3 tones: base, shadow and light. The belly and the muzzle are one tone lighter. Use the same palette for all animals of a game so they feel like one world.\n\
8. Finish with `auto_outline`, then `analyze_sprite` with `outline_key`. A stray pixel near an eye is easy to make: fix every loose pixel it lists.";

/// How to make walk and attack animations. Also available as the topic `walk-and-attack-cycles`.
pub const WALK_GUIDE: &str = "\
Walk and attack cycles guide: frames that move well\n\
\n\
1. Walk: 4 frames is the minimum: contact, passing, contact (the other leg), passing. 6 or 8 frames are smoother. Make the key poses first, as whole frames.\n\
2. Contact: the legs are far apart and the body is at its lowest. Passing: the legs are close and the body is 1 pixel higher. The body goes up and down by 1 pixel at 16x16 and by 2 at 32x32.\n\
3. Ground contact. The lowest row of the feet must be the same row in every frame, or the character floats. `analyze_sprite` compares each frame with the next: `ground_kept` true means the bottom rows did not change. That is right for an idle, but in a walk a foot moves, so false is normal; look at the frames with `render_frames` (`labels` true) to check the feet.\n\
4. Time. Give each frame its time with `set_frame_duration` (milliseconds). A walk: contact 100 to 140, passing 60 to 100. Without it, a frame lasts 1000 divided by the fps of the animation (8 to 12 fps is good for a walk).\n\
5. Start a new frame with `add_frame` and `copy_of`, then change only what moves. Arms swing the opposite way to the legs. Do not add colors for one frame.\n\
6. Attack has three phases. Anticipation: pull back, slow (150 to 250 ms). Strike: the biggest pose, fast (40 to 80 ms, 1 or 2 frames). Recovery: come back to the first idle pose (100 to 150 ms). A long wind-up and a short strike feel powerful.\n\
7. Move a body part without redrawing it: `transform_region` with the operation move and a `region` around the body, so the feet stay in place.\n\
8. The loop closes. A walk goes back to its first frame: `create_tag` with mode loop. An attack is not a loop: use mode once and end on the idle pose, so the game can go back to idle without a jump.\n\
9. Look at the whole animation with `render_frames` and at the movement with `render_onion`. If the body grows, shrinks or jumps by accident, fix that frame.";

/// What a game needs from a sprite. Also available as the topic `game-ready`.
pub const GAME_READY_GUIDE: &str = "\
Game-ready guide: anchors, times, tiles and export\n\
\n\
1. One size. All frames of a character have the same size, and so do the characters of one game (16, 32 or 48). Leave one empty pixel around the shape for the outline.\n\
2. Anchors are the points the game needs: feet (where the sprite stands, usually the bottom center), hand (a weapon), head, muzzle. Use `set_anchor`; without `frame` it is for the whole sprite, with `frame` only for that frame (a hand that moves). Keep feet on the same pixel in every frame. Read them back with `get_sprite`; `remove_anchor` removes one.\n\
3. Times. `set_frame_duration` gives a frame its milliseconds (10 to 10000). A frame that has exactly 100 lasts 1000 divided by the fps of its animation.\n\
4. Animations are tags: `create_tag` with the usual names (idle, walk, attack, hurt). Idle and walk use mode loop; attack and death use mode once.\n\
5. Tiles repeat, so the first column must equal the last column and the first row must equal the last row. Call `check_tile`: it says if the tile is seamless and lists the pixels that differ. `render_tiled` shows the tile 3 by 3 so you can see the seams. Fix them with `set_pixels`. Avoid a single bright pixel that makes the repetition obvious.\n\
6. One palette for a set. Keep the keys of the same material equal in every sprite. At the end call `remove_unused_colors`.\n\
7. Check before you export: `analyze_sprite` for each sprite (no loose pixels, no outline gaps).\n\
8. Export with `export`: format sheet writes one PNG and a JSON file with the frames, animations, times and anchors; format png writes one file per frame. Choose the sprites with `scope`. Use `scale` 1 for the engine. Files go inside the project (default `assets/sprites`) and cannot be undone.";

/// The grid of the example of the `tools` guide: an 8x8 gem whose outline has
/// a gap at x 7, y 3.
macro_rules! gem_grid {
    () => {
        "\
legend\n\
\x20\x20K outline #2B1B4A\n\
\x20\x20J gem #2FB5D6\n\
\x20\x20L light #B9F3FF\n\
frame 1\n\
........\n\
..KKKK..\n\
.KLJJJK.\n\
KJJJJJJ.\n\
.KJJJJK.\n\
..KJJK..\n\
...KK...\n\
........\n"
    };
}

/// The grid of the example of the `tools` guide, for the tests.
pub const GEM_GRID: &str = gem_grid!();

/// A map of the tools. Also available as the topic `tools`.
pub const TOOLS_GUIDE: &str = concat!("\
Tools guide: which tool for which job\n\
\n\
1. Read (they change nothing): `get_project` (folders, palettes, revision), `list_sprites` (sizes, frames, animations), `get_sprite` (the grid; `frame`, `region` and `include_palette` read a part), `get_palette` (the keys), `list_projects` and `open_project` (find and open a project), `get_guide`.\n\
2. See: `render_preview` (one frame), `render_frames` (many frames, `labels`), `render_onion` (the movement), `render_tiled` (a tile 3 by 3), `open_sprite` (shows a sprite to the person). Every pixel write also returns an image.\n\
3. Create: `create_folder`, `create_sprite` (`own_palette` for a palette of its own), `add_frame` (`copy_of` copies a frame), `delete_frame`, `delete_sprite`.\n\
4. Draw: `apply_grid` (a frame or a region from text; best for shapes), `set_pixels` (single pixels), `draw_shape` (line, rectangle, ellipse, fill). `symmetry` and `only_over` work on all three.\n\
5. Edit: `transform_region` (flip, mirror, move, rotate), `copy_region` (copy to another place, frame or sprite), `import_reference` (a PNG from `references/`).\n\
6. Finish: `auto_outline` when the shape is done, `auto_shade` for lit and shadow tones.\n\
7. Check: `analyze_sprite` (loose pixels, outline gaps, unused colors, what changes between frames), `check_tile` (seams of a tile).\n\
8. Colors: `set_palette_color`, `generate_ramp`, `create_palette`, `set_sprite_palette`, `remove_unused_colors`.\n\
9. Animation and game data: `create_tag`, `set_frame_duration`, `set_anchor`, `remove_anchor`, `export`.\n\
10. Mistakes: `undo_last_ai_action` takes back your last action, if nothing happened after it.\n\
Pick the smallest tool: one pixel is `set_pixels`; a shape or a part is `apply_grid`; a copy is `copy_region`; both sides at once is `symmetry`; an outline or shading is `auto_outline` or `auto_shade`; a move or a flip is `transform_region`. Never redraw a whole frame to move or flip something.\n\
When a call fails read its `message`. Common codes: `unknown_key` (the key is not in the palette), `out_of_range` and `region_out` (outside the sprite), `stale_revision` (read again), `permission_denied` (ask the person).\n\
\n\
Worked example: find and fix a gap in an outline (an 8x8 gem).\n\
1. `create_sprite` with name gem, width 8 and height 8.\n\
2. `apply_grid` with name gem and this grid. The outline is missing at the right of row 4:\n\
\n",
    gem_grid!(),
    "\n\
3. `analyze_sprite` with name gem and `outline_key` K. The answer has `outline_gaps` with count 1 and `at` [[6, 3]]: the pixel x 6, y 3 is open to the outside.\n\
4. `set_pixels` with name gem and pixels [{x: 7, y: 3, key: K}]. LOOK at the image that comes back.\n\
5. `analyze_sprite` again: `outline_gaps` has count 0 and `loose_pixels` has count 0. Now the sprite is clean."
);

/// The text of a guide topic, or `None` when it is not a topic.
pub fn guide(topic: &str) -> Option<&'static str> {
    match topic {
        "design" => Some(DESIGN_GUIDE),
        "animation" => Some(ANIMATION_GUIDE),
        "palette" => Some(PALETTE_GUIDE),
        "large-sprites" => Some(LARGE_SPRITES_GUIDE),
        "faces-and-animals" => Some(FACES_GUIDE),
        "walk-and-attack-cycles" => Some(WALK_GUIDE),
        "game-ready" => Some(GAME_READY_GUIDE),
        "tools" => Some(TOOLS_GUIDE),
        _ => None,
    }
}

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "get_guide",
        description: "Returns the pixel art guide for a topic: `design` (palette ramps, light, silhouette, shading, outline, how to review your work), `animation` (key poses, loops), `palette` (how to choose colors), `large-sprites` (more than 32x32), `faces-and-animals`, `walk-and-attack-cycles`, `game-ready` (anchors, times, tiles, export) or `tools` (which tool for which job). Read `design` before you draw a new sprite. It changes nothing and needs no permission.",
        schema: json!({ "type": "object", "required": ["topic"], "properties": {
            "topic": { "type": "string", "enum": GUIDE_TOPICS, "description": "The guide to read. `tools` is a map of the tools." },
        }}),
        need: Need::Nothing,
        run: get_guide,
    }]
}

fn get_guide(_ctx: &crate::mcp::Ctx, input: &Value) -> ToolResult {
    let topic = args::required_str(input, "topic")?;
    let text = guide(&topic.to_lowercase()).ok_or_else(|| {
        ToolError::new(
            "unknown_topic",
            format!("there is no guide {topic:?}; the topics are: {}", GUIDE_TOPICS.join(", ")),
        )
        .with("topics", GUIDE_TOPICS.to_vec())
    })?;
    Ok(Output::Json(json!({ "topic": topic.to_lowercase(), "guide": text })))
}
