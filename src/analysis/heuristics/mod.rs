// Tue Jan 13 2026 - Alex

pub mod engine;
pub mod patterns;
pub mod rules;
pub mod scoring;
pub mod learning;
pub mod detector;

pub use engine::HeuristicsEngine;
pub use patterns::HeuristicPattern;
pub use rules::HeuristicRule;
pub use scoring::HeuristicScorer;
pub use learning::PatternLearner;
pub use detector::OffsetDetector;

use crate::memory::{Address, MemoryReader, MemoryError};
use std::sync::Arc;

pub type HeuristicAnalyzer = HeuristicsEngine;

#[derive(Debug, Clone)]
pub struct HeuristicResult {
    pub is_match: bool,
    pub confidence: f64,
    pub reason: String,
}

impl HeuristicResult {
    pub fn positive(confidence: f64, reason: &str) -> Self {
        Self {
            is_match: true,
            confidence,
            reason: reason.to_string(),
        }
    }

    pub fn negative(reason: &str) -> Self {
        Self {
            is_match: false,
            confidence: 0.0,
            reason: reason.to_string(),
        }
    }
}

impl HeuristicsEngine {
    pub fn is_function_entry(&self, addr: Address) -> Result<HeuristicResult, MemoryError> {
        match self.is_likely_function_start(addr) {
            Ok(true) => Ok(HeuristicResult::positive(0.85, "Function prologue detected")),
            Ok(false) => Ok(HeuristicResult::negative("No function prologue found")),
            Err(e) => Err(e),
        }
    }
}
