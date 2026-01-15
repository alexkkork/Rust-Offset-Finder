// Tue Jan 13 2026 - Alex

use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PatternError {
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    #[error("Pattern too long: {0} bytes")]
    PatternTooLong(usize),
    #[error("Pattern too short: {0} bytes")]
    PatternTooShort(usize),
    #[error("Invalid wildcard: {0}")]
    InvalidWildcard(String),
    #[error("Pattern match failed: {0}")]
    MatchFailed(String),
    #[error("Pattern compilation failed: {0}")]
    CompilationFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
