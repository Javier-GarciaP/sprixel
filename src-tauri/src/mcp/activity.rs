//! The activity log: what every client did, with the time, the client name, a
//! short description, the sprite and the result. It lives for the session
//! (it is not saved) and is shown newest first.
//!
//! Only actions are logged: every call of a tool that changes something or
//! writes files, whether it worked or not. Reading tools are not actions.

use super::tool::{ToolError, ToolResult};
use serde::Serialize;
use serde_json::Value;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// The log keeps the last entries only.
const MAX_ENTRIES: usize = 1000;

/// The event sent to the UI when an entry is added.
pub const ACTIVITY_EVENT: &str = "mcp://activity";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Entry {
    pub id: u64,
    pub at_ms: u64,
    /// The name of the client, for example `Claude Code`.
    pub client: String,
    /// The tool that was called.
    pub tool: String,
    /// A short description in English, for example `Created the sprite slime`.
    pub summary: String,
    /// The sprite (as a path like `Characters/goblin`), if the action had one.
    pub sprite: Option<String>,
    /// `ok` or `error`.
    pub result: &'static str,
    /// The error code, when the result is `error`.
    pub error: Option<String>,
    /// The revision of the project after the action, if it changed the project
    /// (a call that changed nothing has none).
    pub revision: Option<u64>,
}

/// An entry as the UI shows it: with the Undo button state.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Row {
    #[serde(flatten)]
    pub entry: Entry,
    /// True only for the most recent AI action, while it is still the latest
    /// entry of the history of the project.
    pub can_undo: bool,
}

#[derive(Default)]
pub struct Activity {
    entries: VecDeque<Entry>,
    next_id: u64,
}

impl Activity {
    pub fn add(&mut self, mut entry: Entry) -> Entry {
        self.next_id += 1;
        entry.id = self.next_id;
        if self.entries.len() >= MAX_ENTRIES {
            self.entries.pop_front();
        }
        self.entries.push_back(entry.clone());
        entry
    }

    /// The entries, newest first.
    pub fn newest_first(&self) -> Vec<Entry> {
        self.entries.iter().rev().cloned().collect()
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// The entry for a finished call of `tool` by `client`.
pub fn entry_for(client: &str, tool: &str, args: &Value, outcome: &ToolResult) -> Entry {
    let (summary, sprite, result, error, revision) = match outcome {
        // A write with a preview has its answer as the text next to the image.
        Ok(super::tool::Output::Json(answer)) | Ok(super::tool::Output::Image { info: answer, .. }) => {
            let (summary, sprite) = describe(tool, args, answer);
            // Only a call that changed the project has a revision to undo.
            let revision = if answer["changed"] == true { answer["revision"].as_u64() } else { None };
            (summary, sprite, "ok", None, revision)
        }
        Err(error) => (failed(tool, error), sprite_of(args), "error", Some(error.code.clone()), None),
    };
    // Names and paths come from the client: keep them short and clean, so a
    // client cannot fill the memory of the app through the log.
    let clip = |text: String, max: usize| super::sessions::clean_text(&text, max);
    Entry {
        id: 0,
        at_ms: now_ms(),
        client: client.to_string(),
        tool: tool.to_string(),
        summary: clip(summary, 300),
        sprite: sprite.map(|s| clip(s, 200)),
        result,
        error,
        revision,
    }
}

fn failed(tool: &str, error: &ToolError) -> String {
    format!("Could not run {tool}: {}", error.code)
}

/// The sprite an action was about, from its arguments, as a path.
fn sprite_of(args: &Value) -> Option<String> {
    let name = args["name"].as_str().or_else(|| args["sprite"].as_str())?;
    Some(match args["folder"].as_str().filter(|f| !f.is_empty()) {
        Some(folder) if !name.contains('/') => format!("{}/{name}", folder.trim_matches('/')),
        _ => name.to_string(),
    })
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or("?")
}

/// What the action did, in a few words, and the sprite it was about.
fn describe(tool: &str, args: &Value, answer: &Value) -> (String, Option<String>) {
    // The tool says the real path of the sprite; the arguments are the fallback.
    let sprite = answer["sprite"].as_str().map(str::to_string).or_else(|| sprite_of(args));
    let name = sprite.clone().unwrap_or_default();
    let summary = match tool {
        "create_folder" => format!("Created the folder {}", text(answer, "path")),
        "create_sprite" => format!("Created the sprite {}", text(answer, "name")),
        "add_frame" => format!("Added frame {} to {name}", answer["frame"]),
        "apply_grid" => format!("Applied a grid to {name}"),
        "set_pixels" => format!("Set {} pixels on {name}", answer["pixels_set"]),
        "draw_shape" => format!("Drew a {} on {name}", text(args, "shape")),
        "transform_region" => format!("Applied {} to a region of {name}", text(args, "operation")),
        "copy_region" => format!("Copied a region of {name}"),
        "delete_frame" => format!("Deleted frame {} of {name}", answer["deleted_frame"]),
        "create_tag" => format!("Created the animation {} on {name}", text(answer, "animation")),
        "set_palette_color" => format!(
            "{} the color {} in palette {}",
            if answer["action"] == "added" { "Added" } else { "Changed" },
            text(answer, "key"),
            text(answer, "palette")
        ),
        "delete_sprite" => format!("Deleted the sprite {}", text(answer, "deleted")),
        "export" => format!(
            "Exported {} files to {}",
            answer["files"].as_array().map_or(0, Vec::len),
            text(answer, "output")
        ),
        "open_project" => format!("Opened the project {}", text(answer, "name")),
        "undo_last_ai_action" => "Undid the last AI action".to_string(),
        other => other.to_string(),
    };
    (summary, sprite)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(client: &str) -> Entry {
        Entry {
            id: 0, at_ms: 0, client: client.into(), tool: "t".into(), summary: "s".into(),
            sprite: None, result: "ok", error: None, revision: None,
        }
    }

    // Checks that the log is shown newest first, gives each entry its own
    // number, and keeps only the last 1000 entries.
    #[test]
    fn the_log_is_newest_first_and_limited() {
        let mut log = Activity::default();
        for i in 0..1005 {
            log.add(entry(&format!("client {i}")));
        }
        let all = log.newest_first();
        assert_eq!(all.len(), MAX_ENTRIES);
        assert_eq!(all[0].client, "client 1004");
        assert_eq!(all[0].id, 1005);
        assert_eq!(all.last().unwrap().client, "client 5");
    }
}
