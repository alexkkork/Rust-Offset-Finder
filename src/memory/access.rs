// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryError, MemoryReader, MemoryRange};
use std::sync::Arc;

pub struct MemoryAccess {
    reader: Arc<dyn MemoryReader>,
    range: MemoryRange,
}

impl MemoryAccess {
    pub fn new(reader: Arc<dyn MemoryReader>, range: MemoryRange) -> Self {
        Self { reader, range }
    }

    pub fn range(&self) -> &MemoryRange {
        &self.range
    }

    pub fn read_bytes(&self, addr: Address, len: usize) -> Result<Vec<u8>, MemoryError> {
        if !self.range.contains(addr) {
            return Err(MemoryError::OutOfBounds(addr.as_u64()));
        }
        if addr.as_u64() + len as u64 > self.range.end().as_u64() {
            return Err(MemoryError::OutOfBounds(addr.as_u64() + len as u64));
        }
        self.reader.read_bytes(addr, len)
    }

    pub fn read_u8(&self, addr: Address) -> Result<u8, MemoryError> {
        if !self.range.contains(addr) {
            return Err(MemoryError::OutOfBounds(addr.as_u64()));
        }
        self.reader.read_u8(addr)
    }

    pub fn read_u16(&self, addr: Address) -> Result<u16, MemoryError> {
        if !self.range.contains(addr) {
            return Err(MemoryError::OutOfBounds(addr.as_u64()));
        }
        self.reader.read_u16(addr)
    }

    pub fn read_u32(&self, addr: Address) -> Result<u32, MemoryError> {
        if !self.range.contains(addr) {
            return Err(MemoryError::OutOfBounds(addr.as_u64()));
        }
        self.reader.read_u32(addr)
    }

    pub fn read_u64(&self, addr: Address) -> Result<u64, MemoryError> {
        if !self.range.contains(addr) {
            return Err(MemoryError::OutOfBounds(addr.as_u64()));
        }
        self.reader.read_u64(addr)
    }

    pub fn read_ptr(&self, addr: Address) -> Result<Address, MemoryError> {
        if !self.range.contains(addr) {
            return Err(MemoryError::OutOfBounds(addr.as_u64()));
        }
        self.reader.read_ptr(addr)
    }

    pub fn read_string(&self, addr: Address, max_len: usize) -> Result<String, MemoryError> {
        if !self.range.contains(addr) {
            return Err(MemoryError::OutOfBounds(addr.as_u64()));
        }
        self.reader.read_string(addr, max_len)
    }

    pub fn read_c_string(&self, addr: Address) -> Result<String, MemoryError> {
        if !self.range.contains(addr) {
            return Err(MemoryError::OutOfBounds(addr.as_u64()));
        }
        self.reader.read_c_string(addr)
    }

    pub fn scan_pattern(&self, pattern: &[u8], start: Address, end: Address) -> Result<Vec<Address>, MemoryError> {
        let mut results = Vec::new();
        let mut current = start;
        while current.as_u64() + pattern.len() as u64 <= end.as_u64() {
            if let Ok(bytes) = self.read_bytes(current, pattern.len()) {
                if bytes == pattern {
                    results.push(current);
                }
            }
            current = current + 1;
        }
        Ok(results)
    }
}
