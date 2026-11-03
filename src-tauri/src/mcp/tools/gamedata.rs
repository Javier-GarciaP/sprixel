//! The tools for game engines: frame durations, anchor points and seamless
//! tiles.

use super::args;
use super::create::base_revision;
use super::read::check_frame;
use super::write::{Plan, write};
use crate::document::Operation;
use crate::mcp::{Ctx, Need, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "set_frame_duration",
        description: concat!("Sets how long one frame lasts, in milliseconds (10 to 10000). Frames are numbered from 1. `get_sprite` lists the durations in `frame_durations`. A frame that keeps exactly 100 ms is treated as having no duration of its own: inside an animation it lasts 1000 / fps. Set any other value to fix the time of the frame.", write_note!()),
        schema: json!({ "type": "object", "required": ["name", "frame", "milliseconds"], "properties": {
            "name": { "type": "string" },
            "folder": { "type": "string" },
            "frame": { "type": "integer", "minimum": 1 },
            "milliseconds": { "type": "integer", "minimum": 10, "maximum": 10000 },
            "base_revision": base_revision(),
        }}),
        need: Need::Edit,
        run: set_frame_duration,
    }]
}

fn set_frame_duration(ctx: &Ctx, input: &Value) -> ToolResult {
    let frame = args::required_u32(input, "frame")?;
    let ms = args::required_u32(input, "milliseconds")?;
    if !(10..=10000).contains(&ms) {
        return Err(ToolError::new("bad_argument", "`milliseconds` must be from 10 to 10000").with("argument", "milliseconds"));
    }
    write(ctx, input, |session| {
        let target = args::sprite(&session.project, input)?;
        let sprite = session.project.find_sprite(&target).expect("just found");
        check_frame(sprite, frame)?;
        let ops = vec![Operation::SetFrameDuration { sprite: target.clone(), frame, duration_ms: ms }];
        Ok(Plan::new(ops, json!({ "frame": frame, "milliseconds": ms })).about(&target))
    })
}
