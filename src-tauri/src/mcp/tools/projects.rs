//! `list_projects` and `open_project`: an agent finds the projects of the
//! person and opens one, so it does not wait for a click. Opening needs the
//! permission "Open projects" (off by default) because any folder can be named.

use super::args;
use crate::ipc::project::{self, ProjectError};
use crate::mcp::{Ctx, Need, Output, Tool, ToolError, ToolResult};
use serde_json::{Value, json};
use std::path::Path;

/// The event that tells the UI a project was opened by an agent. The payload
/// is `{ "name", "path" }`; the UI shows it as if the person opened it.
pub const PROJECT_OPENED_EVENT: &str = "mcp://project-opened";

pub fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_projects",
            description: "Lists the recent projects of the person (name and absolute path, newest first) and the project that is open now (`open`, or null). \
Use it when `get_project` says no project is open, then call `open_project`.",
            schema: json!({ "type": "object", "properties": {} }),
            need: Need::Nothing,
            run: list_projects,
        },
        Tool {
            name: "open_project",
            description: "Opens the Sprixel project in a folder, as if the person opened it; the open project is saved and closed first. \
`path` is the absolute path of a folder that has a `sprixel.json` (see `list_projects`). \
Needs the permission \"Open projects\", which is off by default: if it fails with `permission_denied`, ask the person to turn it on in the Agents tab.",
            schema: json!({ "type": "object", "required": ["path"], "properties": {
                "path": { "type": "string", "description": "Absolute path of the project folder." }
            }}),
            need: Need::OpenProject,
            run: open_project,
        },
    ]
}

fn list_projects(ctx: &Ctx, _input: &Value) -> ToolResult {
    let open = project::read(ctx.core.host.project(), |session| {
        json!({ "name": session.project.name, "path": session.root().to_string_lossy() })
    });
    let recent: Vec<Value> = ctx
        .core
        .host
        .recent_projects()
        .into_iter()
        .map(|p| json!({ "name": p.name, "path": p.path }))
        .collect();
    Ok(Output::Json(json!({ "recent": recent, "open": open })))
}

fn open_project(ctx: &Ctx, input: &Value) -> ToolResult {
    let path = args::required_str(input, "path")?;
    if !Path::new(path).is_absolute() {
        return Err(ToolError::new("bad_argument", "`path` must be an absolute path; `list_projects` gives them").with("argument", "path"));
    }
    // Not in the middle of a write of another call.
    let _one_at_a_time = ctx.core.write_lock.lock().unwrap_or_else(|e| e.into_inner());
    let info = ctx.core.host.open_project(Path::new(path)).map_err(|error| match error {
        ProjectError::NotAProject => ToolError::new(
            "not_a_project",
            "the folder does not exist or has no sprixel.json, so it is not a Sprixel project; the open project did not change",
        )
        .with("path", path),
        ProjectError::Io { message } => ToolError::new("io_error", message).with("path", path),
        other => ToolError::new("io_error", format!("{other:?}")).with("path", path),
    })?;
    ctx.core.log("an agent opened a project");
    ctx.core.host.emit(PROJECT_OPENED_EVENT, json!({ "name": info.name, "path": info.path }));
    Ok(Output::Json(json!({ "opened": true, "name": info.name, "path": info.path, "damaged_files": info.damaged_files })))
}
