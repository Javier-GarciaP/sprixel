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

/// The topics of `get_guide`, in the order they are listed.
pub const GUIDE_TOPICS: [&str; 3] = ["design", "animation", "palette"];

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
9. Big sprites (more than 32x32): work region by region. Write one part at a time (head, body, legs) with `apply_grid` and `x` and `y`, look at the image after each part, then go on.";

/// How to animate. Also available as the topic `animation`.
pub const ANIMATION_GUIDE: &str = "\
Animation guide: how to make a sprite move\n\
\n\
1. Key poses first. Draw the main poses as whole frames before anything else (an idle needs 2 frames; a walk needs contact and passing poses). Add the in-between frames after.\n\
2. Same palette in every frame. Do not add new colors for one frame. Keep the same light direction and the same outline.\n\
3. Keep the size and the ground line. The body must not grow or jump by accident between frames: feet stay on the same row unless the pose is a jump.\n\
4. Change little. Start a new frame with `add_frame` and `copy_of`, then change only what moves, with `set_pixels` or `apply_grid` and `x` and `y`.\n\
5. The loop closes. The last frame must lead back to the first without a jump. Then make the animation with `create_tag` (mode loop, 6 to 12 fps).\n\
6. Look at every frame with `render_preview` (argument `frame`) and fix what looks wrong before you say you are done.";

/// How to choose and build the colors. Also available as the topic `palette`.
pub const PALETTE_GUIDE: &str = "\
Palette guide: how to choose colors\n\
\n\
1. Read the palette first with `get_palette`. A key is one character; `.` is always transparent.\n\
2. Keep it small: 8 to 16 colors is enough for most sprites (the maximum is 93).\n\
3. Build ramps. For each hue make 3 tones: shadow, base and light. Make the shadow a little cooler and the light a little warmer, not just darker and lighter.\n\
4. Avoid pure black and pure white. Use a very dark tone of the material for the outline.\n\
5. Add colors with a `legend` line in `apply_grid` (`<key> <name> #RRGGBB`) or with `set_palette_color`. Give related colors related keys, for example `G` base green, `D` dark green, `L` light green.\n\
6. Changing a color with `set_palette_color` recolors every pixel that uses its key, so you can fix a tone at the end without redrawing.";

/// The text of a guide topic, or `None` when it is not a topic.
pub fn guide(topic: &str) -> Option<&'static str> {
    match topic {
        "design" => Some(DESIGN_GUIDE),
        "animation" => Some(ANIMATION_GUIDE),
        "palette" => Some(PALETTE_GUIDE),
        _ => None,
    }
}

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "get_guide",
        description: "Returns the pixel art guide for a topic: `design` (palette ramps, light, silhouette, shading, outline, how to review your work), `animation` (key poses, loops) or `palette` (how to choose colors). Read `design` before you draw a new sprite. It changes nothing and needs no permission.",
        schema: json!({ "type": "object", "required": ["topic"], "properties": {
            "topic": { "type": "string", "enum": GUIDE_TOPICS, "description": "design, animation or palette." },
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
