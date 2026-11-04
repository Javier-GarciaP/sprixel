use std::fmt;

/// Why an export did not happen. The `code` is stable; the message is for
/// people.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportError {
    pub code: &'static str,
    pub message: String,
}

impl ExportError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        ExportError { code, message: message.into() }
    }
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ExportError {}
