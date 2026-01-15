// Tue Jan 13 2026 - Alex

use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FinderError {
    #[error("Offset not found: {0}")]
    NotFound(String),
    #[error("Multiple matches found for: {0}")]
    MultipleMatches(String),
    #[error("Pattern scan failed: {0}")]
    PatternScanFailed(String),
    #[error("Symbol resolution failed: {0}")]
    SymbolResolutionFailed(String),
    #[error("XRef analysis failed: {0}")]
    XRefAnalysisFailed(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
