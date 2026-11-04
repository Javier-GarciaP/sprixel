//! What a tool is: its name, its schema, the permission it needs and the
//! function that runs it, plus the shared result and error types.

use super::Core;
use crate::document::DocError;
use serde_json::{Map, Value, json};

/// The permission a tool needs (see `policy`). Read tools need none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Need {
    Nothing,
    Create,
    Edit,
    Delete,
    Export,
    OpenProject,
}

/// An error an AI can read and act on: a stable `code`, a message that says
/// how to fix it, and extra details (name, line, row, limit...).
#[derive(Debug, Clone, PartialEq)]
pub struct ToolError {
    pub code: String,
    pub message: String,
    pub details: Map<String, Value>,
}

impl ToolError {
    pub fn new(code: &str, message: impl Into<String>) -> ToolError {
        ToolError { code: code.to_string(), message: message.into(), details: Map::new() }
    }

    /// Adds one detail, for example `.with("line", 3)`.
    pub fn with(mut self, key: &str, value: impl Into<Value>) -> ToolError {
        self.details.insert(key.to_string(), value.into());
        self
    }

    /// `{"code": ..., "message": ..., ...details}`
    pub fn to_json(&self) -> Value {
        let mut object = self.details.clone();
        object.insert("code".to_string(), json!(self.code));
        object.insert("message".to_string(), json!(self.message));
        Value::Object(object)
    }
}

/// What a tool gives back.
#[derive(Debug)]
pub enum Output {
    Json(Value),
    /// A PNG image, with a JSON text next to it.
    Image { png: Vec<u8>, info: Value },
}

pub type ToolResult = Result<Output, ToolError>;

/// Who is calling a tool, and the core it works on.
pub struct Ctx<'a> {
    pub core: &'a Core,
    /// The name of the client, which is the author of its changes.
    pub client: &'a str,
    /// The name of the tool being run.
    pub tool: &'a str,
}

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    /// The JSON Schema of the arguments.
    pub schema: Value,
    pub need: Need,
    pub run: fn(&Ctx, &Value) -> ToolResult,
}

impl Tool {
    /// Checks the permission this tool needs.
    pub(super) fn need_check(&self, permissions: &super::Permissions) -> Result<(), ToolError> {
        permissions.check(self.need)
    }
}

impl From<DocError> for ToolError {
    fn from(error: DocError) -> Self {
        ToolError::new(error.code, error.message)
    }
}

impl ToolError {
    pub fn no_project() -> ToolError {
        ToolError::new("no_project", "no project is open in Sprixel; ask the user to open or create one")
    }
}
