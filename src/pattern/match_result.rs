// Tue Jan 13 2026 - Alex

use crate::memory::Address;

#[derive(Debug, Clone)]
pub struct MatchResult {
    address: Address,
    pattern: String,
    confidence: f64,
    context: Vec<u8>,
}

impl MatchResult {
    pub fn new(address: Address, pattern: String, confidence: f64) -> Self {
        Self {
            address,
            pattern,
            confidence,
            context: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: Vec<u8>) -> Self {
        self.context = context;
        self
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn confidence(&self) -> f64 {
        self.confidence
    }

    pub fn context(&self) -> &[u8] {
        &self.context
    }

    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.9
    }

    pub fn is_medium_confidence(&self) -> bool {
        self.confidence >= 0.7 && self.confidence < 0.9
    }

    pub fn is_low_confidence(&self) -> bool {
        self.confidence < 0.7
    }
}
