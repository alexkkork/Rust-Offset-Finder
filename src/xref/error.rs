// Tue Jan 13 2026 - Alex

use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum XRefError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Edge not found")]
    EdgeNotFound,
    #[error("Cycle detected in call graph")]
    CycleDetected,
    #[error("Traversal depth exceeded: {0}")]
    DepthExceeded(usize),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid reference: {0}")]
    InvalidReference(String),
}
