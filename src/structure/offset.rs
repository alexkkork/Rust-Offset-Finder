// Tue Jan 13 2026 - Alex

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Offset {
    value: u64,
}

impl Offset {
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    pub fn zero() -> Self {
        Self { value: 0 }
    }

    pub fn as_u64(&self) -> u64 {
        self.value
    }

    pub fn as_usize(&self) -> usize {
        self.value as usize
    }

    pub fn is_aligned(&self, alignment: usize) -> bool {
        self.value % alignment as u64 == 0
    }
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.value)
    }
}

impl From<u64> for Offset {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}
