//! The path every write tool follows, so they all behave the same:
//!
//! 1. one write at a time (the `write_lock`);
//! 2. `base_revision`, if given, must be the current revision (`stale_revision`);
//! 3. the tool builds its operations by looking at the project (nothing is
//!    changed yet, so an error changes nothing);
//! 4. the frame is marked as busy, the operations are applied as ONE history
//!    entry whose author is the client, and the UI is told;
//! 5. the answer has the new `revision`.

use super::args;
use crate::document::{Operation, SpriteRef};
use crate::ipc::browser::{ChangedEvent, TargetInfo, CHANGED_EVENT};
use crate::ipc::editor::{AGENT_EVENT, AgentWrite, SpriteArg};
use crate::ipc::project;
use crate::mcp::{Ctx, Output, ToolError, ToolResult};
use crate::storage::Session;
use serde_json::{Map, Value};

/// What a write tool wants to do.
pub struct Plan {
    pub ops: Vec<Operation>,
    /// Fields added to the answer (next to `revision`).
    pub extra: Value,
    /// The frame to mark as busy while the operations are applied.
    pub busy: Option<(SpriteRef, u32)>,
    /// The sprite the call is about; its path is added to the answer.
    pub sprite: Option<SpriteRef>,
}

impl Plan {
    pub fn new(ops: Vec<Operation>, extra: Value) -> Plan {
        Plan { ops, extra, busy: None, sprite: None }
    }

    pub fn busy(mut self, sprite: &SpriteRef, frame: u32) -> Plan {
        self.busy = Some((sprite.clone(), frame));
        self.sprite = Some(sprite.clone());
        self
    }

    /// Says which sprite the call is about (for the answer and the activity log).
    pub fn about(mut self, sprite: &SpriteRef) -> Plan {
        self.sprite = Some(sprite.clone());
        self
    }
}

/// While it lives, the frame is busy and the UI shows who is drawing.
struct Busy<'a> {
    ctx: &'a Ctx<'a>,
}

impl<'a> Busy<'a> {
    fn start(ctx: &'a Ctx<'a>, sprite: &SpriteRef, frame: u32) -> Busy<'a> {
        let write = AgentWrite {
            agent: ctx.client.to_string(),
            sprite: SpriteArg { folder: sprite.folder.clone(), name: sprite.name.clone() },
            frame,
        };
        ctx.core.host.agents().begin(write.clone());
        ctx.core.host.emit(AGENT_EVENT, serde_json::to_value(&write).unwrap_or_default());
        Busy { ctx }
    }
}

impl Drop for Busy<'_> {
    fn drop(&mut self) {
        self.ctx.core.host.agents().end();
        self.ctx.core.host.emit(AGENT_EVENT, Value::Null);
    }
}

/// The error of a client that the user stopped.
pub fn stopped() -> ToolError {
    ToolError::new(
        "stopped_by_user",
        "the person pressed Stop. Do not continue; wait until they let you continue (Agents tab) and ask them what to do",
    )
}

pub fn stale(base: u64, current: u64) -> ToolError {
    ToolError::new(
        "stale_revision",
        format!(
            "the project changed since revision {base} (now {current}); read it again (get_project or get_sprite) and retry"
        ),
    )
    .with("base_revision", base)
    .with("revision", current)
}

/// Runs a write tool. `plan` looks at the project and says what to change.
pub fn write(
    ctx: &Ctx,
    input: &Value,
    plan: impl FnOnce(&Session) -> Result<Plan, ToolError>,
) -> ToolResult {
    let base = args::optional_u64(input, "base_revision")?;
    let core = ctx.core;
    // A panic of another write must not block all the others.
    let _one_at_a_time = core.write_lock.lock().unwrap_or_else(|e| e.into_inner());
    let mut busy = None;
    let done = project::edit(core.host.project(), |session| {
        if let Some(base) = base.filter(|base| *base != session.project.revision) {
            return Err(stale(base, session.project.revision));
        }
        let plan = plan(&*session)?;
        if let Some((sprite, frame)) = &plan.busy {
            busy = Some(Busy::start(ctx, sprite, *frame));
        }
        #[cfg(test)]
        {
            let pause = core.pause.lock().unwrap().clone();
            if let Some(pause) = pause {
                pause();
            }
        }
        // The last moment to cancel: the person may have pressed Stop while
        // the call was in progress. Nothing has been applied yet.
        if core.host.agents().is_stopped(ctx.client) {
            return Err(stopped());
        }
        let changed = !plan.ops.is_empty();
        let sprite = plan.sprite;
        let touched = if changed { session.apply(ctx.client, plan.ops)? } else { Vec::new() };
        let mut answer = match plan.extra {
            Value::Object(map) => map,
            _ => Map::new(),
        };
        if let Some(sprite) = &sprite {
            let path = args::join(&[sprite.folder.clone(), vec![sprite.name.clone()]].concat());
            answer.insert("sprite".into(), path.into());
        }
        answer.insert("revision".into(), session.project.revision.into());
        answer.insert("changed".into(), changed.into());
        let event = ChangedEvent {
            revision: session.project.revision,
            touched: touched.iter().map(TargetInfo::from).collect(),
        };
        Ok((answer, event, changed))
    })
    .ok_or_else(ToolError::no_project)?;
    let (answer, event, changed) = done?;
    if changed {
        core.host.emit(CHANGED_EVENT, serde_json::to_value(event).unwrap_or_default());
    }
    // The marker goes away after the UI knows about the change.
    drop(busy);
    Ok(Output::Json(Value::Object(answer)))
}
