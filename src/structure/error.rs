// Tue Jan 13 2026 - Alex

use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StructureError {
    #[error("Field not found: {0}")]
    FieldNotFound(String),
    #[error("Invalid offset: {0}")]
    InvalidOffset(u64),
    #[error("Invalid alignment: {0}")]
    InvalidAlignment(usize),
    #[error("Invalid size: {0}")]
    InvalidSize(usize),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}
