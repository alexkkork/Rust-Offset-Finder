// Tue Jan 13 2026 - Alex

use std::fmt;
use std::ops::{Add, Sub, Mul, Div};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address {
    value: u64,
}

impl Address {
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    pub fn zero() -> Self {
        Self { value: 0 }
    }

    pub fn from_ptr(ptr: *const u8) -> Self {
        Self { value: ptr as u64 }
    }

    pub fn as_u64(&self) -> u64 {
        self.value
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.value as *const u8
    }

    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.value as *mut u8
    }

    pub fn is_null(&self) -> bool {
        self.value == 0
    }

    pub fn is_aligned(&self, alignment: usize) -> bool {
        self.value % alignment as u64 == 0
    }

    pub fn align_down(&self, alignment: usize) -> Self {
        Self { value: self.value & !(alignment as u64 - 1) }
    }

    pub fn align_up(&self, alignment: usize) -> Self {
        Self { value: (self.value + alignment as u64 - 1) & !(alignment as u64 - 1) }
    }

    pub fn offset(&self, offset: i64) -> Self {
        Self { value: (self.value as i64 + offset) as u64 }
    }

    pub fn distance(&self, other: Self) -> i64 {
        self.value as i64 - other.value as i64
    }

    pub fn is_within_range(&self, start: Self, end: Self) -> bool {
        self.value >= start.value && self.value < end.value
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", self.value)
    }
}

impl fmt::LowerHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.value, f)
    }
}

impl fmt::UpperHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.value, f)
    }
}

impl Add<u64> for Address {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        Self { value: self.value + rhs }
    }
}

impl Sub<u64> for Address {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        Self { value: self.value - rhs }
    }
}

impl Sub<Address> for Address {
    type Output = i64;
    fn sub(self, rhs: Address) -> Self::Output {
        self.value as i64 - rhs.value as i64
    }
}

impl Mul<u64> for Address {
    type Output = Self;
    fn mul(self, rhs: u64) -> Self::Output {
        Self { value: self.value * rhs }
    }
}

impl Div<u64> for Address {
    type Output = Self;
    fn div(self, rhs: u64) -> Self::Output {
        Self { value: self.value / rhs }
    }
}

impl From<u64> for Address {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<Address> for u64 {
    fn from(addr: Address) -> Self {
        addr.value
    }
}
