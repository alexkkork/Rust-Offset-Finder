// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryRange {
    start: Address,
    end: Address,
}

impl MemoryRange {
    pub fn new(start: Address, end: Address) -> Self {
        assert!(end.as_u64() >= start.as_u64(), "end must be >= start");
        Self { start, end }
    }

    pub fn from_start_size(start: Address, size: u64) -> Self {
        Self::new(start, start + size)
    }

    pub fn start(&self) -> Address {
        self.start
    }

    pub fn end(&self) -> Address {
        self.end
    }

    pub fn size(&self) -> u64 {
        self.end.as_u64() - self.start.as_u64()
    }

    pub fn contains(&self, addr: Address) -> bool {
        addr.as_u64() >= self.start.as_u64() && addr.as_u64() < self.end.as_u64()
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.start.as_u64() < other.end.as_u64() && self.end.as_u64() > other.start.as_u64()
    }

    pub fn intersects(&self, other: &Self) -> Option<Self> {
        let start = Address::new(self.start.as_u64().max(other.start.as_u64()));
        let end = Address::new(self.end.as_u64().min(other.end.as_u64()));
        if start.as_u64() < end.as_u64() {
            Some(Self::new(start, end))
        } else {
            None
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        let start = Address::new(self.start.as_u64().min(other.start.as_u64()));
        let end = Address::new(self.end.as_u64().max(other.end.as_u64()));
        Self::new(start, end)
    }

    pub fn is_empty(&self) -> bool {
        self.start.as_u64() >= self.end.as_u64()
    }

    pub fn align(&self, alignment: usize) -> Self {
        let start = self.start.align_down(alignment);
        let end = self.end.align_up(alignment);
        Self::new(start, end)
    }
}

impl fmt::Display for MemoryRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.start, self.end)
    }
}
