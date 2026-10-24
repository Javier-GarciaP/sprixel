//! Undo and redo: history entries made of operations with inverses.

// Nothing outside the tests uses this module yet (IPC and MCP come later).
#![allow(dead_code, unused_imports)]

use crate::document::{DocError, Operation, Project, SpriteRef, Target};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
mod history_tests;

/// The author of the changes made by a person.
pub const USER: &str = "user";

/// One step of the history: what was done, who did it, and how to undo it.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// `user` or the name of an AI client.
    pub author: String,
    pub forward: Vec<Operation>,
    pub inverse: Vec<Operation>,
    /// The parts of the project the entry changed.
    pub touched: Vec<Target>,
    /// When the entry was made, in milliseconds since 1970.
    pub at_ms: u64,
}

/// Who changed a sprite last, and when.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Editor {
    pub author: String,
    pub at_ms: u64,
}

#[derive(Debug, Default)]
pub struct History {
    /// Done entries, oldest first. Undo takes from the end.
    entries: Vec<Entry>,
    /// Undone entries. Redo takes from the end.
    undone: Vec<Entry>,
}

impl History {
    pub fn new() -> History {
        History::default()
    }

    /// The entries done so far, oldest first.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Applies the operations of one action (a stroke, an AI call) for
    /// `author` as one entry. If one of them is rejected, none is applied and
    /// the project and the history do not change.
    pub fn apply(
        &mut self,
        project: &mut Project,
        author: &str,
        ops: Vec<Operation>,
    ) -> Result<Vec<Target>, DocError> {
        if ops.is_empty() {
            return Ok(Vec::new());
        }
        let applied = project.apply_group(&ops)?;
        self.entries.push(Entry {
            author: author.to_string(),
            forward: ops,
            inverse: applied.inverses,
            touched: applied.touched.clone(),
            at_ms: now_ms(),
        });
        self.undone.clear();
        Ok(applied.touched)
    }

    /// Who changed the sprite last: the latest done entry that touched it.
    pub fn last_editor(&self, sprite: &SpriteRef) -> Option<Editor> {
        let target = Target::Sprite(sprite.clone());
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.touched.contains(&target))
            .map(|entry| Editor {
                author: entry.author.clone(),
                at_ms: entry.at_ms,
            })
    }

    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    /// Undoes the latest entry, whoever made it. Undo is a change too, so the
    /// revision grows.
    pub fn undo(&mut self, project: &mut Project) -> Result<Vec<Target>, DocError> {
        let entry = self
            .entries
            .pop()
            .ok_or_else(|| DocError::new("nothing_to_undo", "there is nothing to undo"))?;
        match project.apply_group(&entry.inverse) {
            Ok(applied) => {
                self.undone.push(entry);
                Ok(applied.touched)
            }
            Err(error) => {
                self.entries.push(entry);
                Err(error)
            }
        }
    }

    pub fn redo(&mut self, project: &mut Project) -> Result<Vec<Target>, DocError> {
        let entry = self
            .undone
            .pop()
            .ok_or_else(|| DocError::new("nothing_to_redo", "there is nothing to redo"))?;
        match project.apply_group(&entry.forward) {
            Ok(applied) => {
                self.entries.push(entry);
                Ok(applied.touched)
            }
            Err(error) => {
                self.undone.push(entry);
                Err(error)
            }
        }
    }

    /// Undoes the latest entry, but only when an AI made it. If something
    /// newer exists (a change by the user), nothing is undone.
    pub fn undo_last_ai_action(&mut self, project: &mut Project) -> Result<Vec<Target>, DocError> {
        match self.entries.iter().rposition(|e| e.author != USER) {
            None => Err(DocError::new("no_ai_action", "there is no AI action to undo")),
            Some(i) if i + 1 < self.entries.len() => Err(DocError::new(
                "newer_changes",
                "something changed after the last AI action, so it was not undone",
            )),
            Some(_) => self.undo(project),
        }
    }

    /// Forgets all the history. Used when the project changes in a way the
    /// inverses do not know about (a file reloaded from disk).
    pub fn clear(&mut self) {
        self.entries.clear();
        self.undone.clear();
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
