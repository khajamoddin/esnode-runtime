use std::fmt;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    Unsupported(String),
    NotFound(String),
    Invalid(String),
    Internal(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::Unsupported(msg) => write!(f, "unsupported: {msg}"),
            RuntimeError::NotFound(msg) => write!(f, "not found: {msg}"),
            RuntimeError::Invalid(msg) => write!(f, "invalid: {msg}"),
            RuntimeError::Internal(msg) => write!(f, "internal: {msg}"),
        }
    }
}

impl std::error::Error for RuntimeError {}
