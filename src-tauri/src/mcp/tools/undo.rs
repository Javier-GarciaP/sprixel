//! `undo_last_ai_action`: takes back the last thing the agent did, but only
//! when nothing happened after it. If the person (or another agent) changed
//! the project since, undoing would also hide their work, so it fails with
//! `newer_changes` and the person decides in the app.

use super::args;
use super::create::base_revision;
use super::write::{stale, stopped};
use crate::ipc::browser::{CHANGED_EVENT, ChangedEvent, TargetInfo};
use crate::ipc::project;
use crate::mcp::{Ctx, Need, Output, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "undo_last_ai_action",
        description: "Takes back the last action YOU made (one call, however many pixels). It works only if it is still the latest change of the project: \
if the person or another agent changed anything after it, the call fails with `newer_changes` and nothing is undone. \
Returns the new `revision`. Pass `base_revision` to fail with `stale_revision` if the project changed since you read it.",
        schema: json!({ "type": "object", "properties": { "base_revision": base_revision() } }),
        need: Need::Edit,
        run: undo_last_ai_action,
    }]
}

fn undo_last_ai_action(ctx: &Ctx, input: &Value) -> ToolResult {
    let base = args::optional_u64(input, "base_revision")?;
    let core = ctx.core;
    let _one_at_a_time = core.write_lock.lock().unwrap_or_else(|e| e.into_inner());
    let done = project::edit(core.host.project(), |session| {
        if let Some(base) = base.filter(|base| *base != session.project.revision) {
            return Err(stale(base, session.project.revision));
        }
        let last = session
            .history
            .entries()
            .last()
            .map(|entry| entry.author.clone())
            .ok_or_else(|| ToolError::new("nothing_to_undo", "there is nothing to undo"))?;
        if last != ctx.client {
            // The latest change is by the person or by another agent.
            let who = if last == crate::history::USER { "the person".to_string() } else { last };
            return Err(ToolError::new(
                "newer_changes",
                format!("the latest change of the project was made by {who}, not by you, so nothing was undone; read the project again"),
            )
            .with("latest_by", who));
        }
        if core.host.agents().is_stopped(ctx.client) {
            return Err(stopped());
        }
        let touched = session.undo()?;
        let event = ChangedEvent {
            revision: session.project.revision,
            touched: touched.iter().map(TargetInfo::from).collect(),
        };
        Ok((session.project.revision, event))
    })
    .ok_or_else(ToolError::no_project)?;
    let (revision, event) = done?;
    core.host.emit(CHANGED_EVENT, serde_json::to_value(event).unwrap_or_default());
    Ok(Output::Json(json!({ "undone": true, "revision": revision, "changed": true })))
}
