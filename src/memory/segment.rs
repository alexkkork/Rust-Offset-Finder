// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryError, MemoryRange, Protection};
use std::fmt;

#[derive(Debug, Clone)]
pub struct MemorySegment {
    range: MemoryRange,
    protection: Protection,
    name: String,
    offset: u64,
    file_size: u64,
    virtual_size: u64,
}

impl MemorySegment {
    pub fn new(range: MemoryRange, protection: Protection, name: String) -> Self {
        Self {
            range,
            protection,
            name,
            offset: 0,
            file_size: range.size(),
            virtual_size: range.size(),
        }
    }

    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_file_size(mut self, size: u64) -> Self {
        self.file_size = size;
        self
    }

    pub fn with_virtual_size(mut self, size: u64) -> Self {
        self.virtual_size = size;
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

    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    pub fn virtual_size(&self) -> u64 {
        self.virtual_size
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
}

impl fmt::Display for MemorySegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {} (file: {}, virtual: {})", self.range, self.protection, self.name, self.file_size, self.virtual_size)
    }
}
