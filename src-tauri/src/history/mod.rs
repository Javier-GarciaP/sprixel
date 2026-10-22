//! Undo and redo: history entries made of operations with inverses.

// Nothing outside the tests uses this module yet (IPC and MCP come later).
#![allow(dead_code, unused_imports)]

use crate::document::{DocError, Operation, Project, Target};

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
}

#[derive(Debug, Default)]
pub struct History {
    entries: Vec<Entry>,
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
        });
        Ok(applied.touched)
    }
}
