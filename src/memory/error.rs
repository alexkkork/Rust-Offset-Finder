// Tue Jan 13 2026 - Alex

use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Access violation at address {0}")]
    AccessViolation(u64),
    #[error("Region not found: {0}")]
    RegionNotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Read failed at address {0}")]
    ReadFailed(u64),
    #[error("Write failed at address {0}")]
    WriteFailed(u64),
    #[error("Process not found: {0}")]
    ProcessNotFound(String),
    #[error("Binary parse error: {0}")]
    BinaryParseError(String),
    #[error("Invalid memory range")]
    InvalidRange,
    #[error("Out of bounds: address {0} not in range")]
    OutOfBounds(u64),
    #[error("Alignment error: address {0} not aligned to {1}")]
    AlignmentError(u64, usize),
    #[error("Timeout while accessing memory")]
    Timeout,
    #[error("Not supported: {0}")]
    NotSupported(String),
}
