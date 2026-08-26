use std::fmt;
#[derive(Debug)]
pub enum LoxError {
    Scan { line: usize, message: String },
}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxError::Scan { line, message } => write!(f, "[line {line}] Error: {message}"),
        }
    }
}

impl std::error::Error for LoxError {}
