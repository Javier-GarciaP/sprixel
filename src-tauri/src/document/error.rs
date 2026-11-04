use std::fmt;

/// An error with a short stable `code` (for example `too_big`) and a message
/// for people. Callers compare the code; the message can change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocError {
    pub code: &'static str,
    pub message: String,
}

impl DocError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        DocError {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for DocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DocError {}
