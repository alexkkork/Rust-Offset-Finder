// Tue Jan 13 2026 - Alex

use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SymbolError {
    #[error("Symbol not found: {0}")]
    NotFound(String),
    #[error("Invalid symbol table: {0}")]
    InvalidTable(String),
    #[error("Demangling failed: {0}")]
    DemangleFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
}
