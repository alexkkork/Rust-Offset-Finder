// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::pattern::{PatternMask, MatchResult, PatternError};
use std::fmt;

pub struct Signature {
    name: String,
    mask: PatternMask,
    expected_matches: usize,
}

impl Signature {
    pub fn new(name: String, pattern: String) -> Result<Self, PatternError> {
        let mask = PatternMask::from_pattern(&pattern);
        if mask.is_empty() {
            return Err(PatternError::InvalidPattern(format!("Pattern '{}' is empty", pattern)));
        }
        Ok(Self {
            name,
            mask,
            expected_matches: 1,
        })
    }

    pub fn with_expected_matches(mut self, count: usize) -> Self {
        self.expected_matches = count;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mask(&self) -> &PatternMask {
        &self.mask
    }

    pub fn expected_matches(&self) -> usize {
        self.expected_matches
    }

    pub fn matches(&self, data: &[u8], offset: usize) -> bool {
        if offset + self.mask.len() > data.len() {
            return false;
        }
        self.mask.matches(&data[offset..])
    }

    pub fn scan(&self, data: &[u8], base_address: Address) -> Vec<MatchResult> {
        let mut results = Vec::new();
        let mask_len = self.mask.len();
        for (i, chunk) in data.windows(mask_len).enumerate() {
            if self.mask.matches(chunk) {
                let address = base_address + i as u64;
                let context = chunk.to_vec();
                let result = MatchResult::new(address, self.name.clone(), 1.0)
                    .with_context(context);
                results.push(result);
            }
        }
        results
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
