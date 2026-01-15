// Tue Jan 13 2026 - Alex

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Alignment {
    value: usize,
}

impl Alignment {
    pub fn new(value: usize) -> Self {
        assert!(value > 0 && value.is_power_of_two());
        Self { value }
    }

    pub fn from_size(size: usize) -> Self {
        Self::new(size.next_power_of_two())
    }

    pub fn as_usize(&self) -> usize {
        self.value
    }

    pub fn align(&self, offset: u64) -> u64 {
        (offset + self.value as u64 - 1) & !(self.value as u64 - 1)
    }
}

impl Default for Alignment {
    fn default() -> Self {
        Self::new(8)
    }
}
