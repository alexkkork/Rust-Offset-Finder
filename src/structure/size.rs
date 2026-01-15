// Tue Jan 13 2026 - Alex

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Size {
    value: usize,
}

impl Size {
    pub fn new(value: usize) -> Self {
        Self { value }
    }

    pub fn zero() -> Self {
        Self { value: 0 }
    }

    pub fn as_usize(&self) -> usize {
        self.value
    }

    pub fn as_u64(&self) -> u64 {
        self.value as u64
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<usize> for Size {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}
