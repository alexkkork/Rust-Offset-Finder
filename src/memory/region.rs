// Tue Jan 13 2026 - Alex

use crate::memory::{Address, Protection, MemoryRange};
use std::fmt;

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    range: MemoryRange,
    protection: Protection,
    name: String,
    offset: u64,
    file_path: Option<String>,
}

impl MemoryRegion {
    pub fn new(range: MemoryRange, protection: Protection, name: String) -> Self {
        Self {
            range,
            protection,
            name,
            offset: 0,
            file_path: None,
        }
    }

    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_file_path(mut self, path: String) -> Self {
        self.file_path = Some(path);
        self
    }

    pub fn range(&self) -> &MemoryRange {
        &self.range
    }

    pub fn protection(&self) -> Protection {
        self.protection
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    pub fn start(&self) -> Address {
        self.range.start()
    }

    pub fn end(&self) -> Address {
        self.range.end()
    }

    pub fn size(&self) -> u64 {
        self.range.size()
    }

    pub fn contains(&self, addr: Address) -> bool {
        self.range.contains(addr)
    }

    pub fn is_executable(&self) -> bool {
        self.protection.can_execute()
    }

    pub fn is_readable(&self) -> bool {
        self.protection.can_read()
    }

    pub fn is_writable(&self) -> bool {
        self.protection.can_write()
    }

    pub fn is_code(&self) -> bool {
        self.is_executable() && !self.is_writable()
    }

    pub fn is_data(&self) -> bool {
        !self.is_executable() && (self.is_readable() || self.is_writable())
    }
}

impl fmt::Display for MemoryRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} {}", self.range, self.protection, self.name, self.offset)
    }
}
