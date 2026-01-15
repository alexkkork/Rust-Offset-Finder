// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::finders::result::FinderResults;
use std::collections::HashMap;

pub struct ConfidenceCalculator {
    weights: ConfidenceWeights,
    history: Vec<HistoricalConfidence>,
}

impl ConfidenceCalculator {
    pub fn new() -> Self {
        Self {
            weights: ConfidenceWeights::default(),
            history: Vec::new(),
        }
    }

    pub fn with_weights(mut self, weights: ConfidenceWeights) -> Self {
        self.weights = weights;
        self
    }

    pub fn calculate_function_confidence(&self, addr: Address, evidence: &FunctionEvidence) -> ConfidenceScore {
        let mut score = 0.0;
        let mut factors = Vec::new();

        if evidence.has_valid_prologue {
            score += self.weights.valid_prologue;
            factors.push(ConfidenceFactor::new("Valid prologue", self.weights.valid_prologue));
        }

        if evidence.in_executable_region {
            score += self.weights.executable_region;
            factors.push(ConfidenceFactor::new("In executable region", self.weights.executable_region));
        }

        if evidence.aligned {
            score += self.weights.alignment;
            factors.push(ConfidenceFactor::new("Properly aligned", self.weights.alignment));
        }

        if evidence.has_cross_references {
            score += self.weights.cross_references;
            factors.push(ConfidenceFactor::new("Has cross-references", self.weights.cross_references));
        }

        if evidence.symbol_matched {
            score += self.weights.symbol_match;
            factors.push(ConfidenceFactor::new("Symbol matched", self.weights.symbol_match));
        }

        if evidence.pattern_matched {
            score += self.weights.pattern_match;
            factors.push(ConfidenceFactor::new("Pattern matched", self.weights.pattern_match));
        }

        if evidence.xref_validated {
            score += self.weights.xref_validation;
            factors.push(ConfidenceFactor::new("XRef validated", self.weights.xref_validation));
        }

        ConfidenceScore {
            score: score.min(1.0),
            level: ConfidenceLevel::from_score(score),
            factors,
        }
    }

    pub fn calculate_offset_confidence(&self, offset: u64, evidence: &OffsetEvidence) -> ConfidenceScore {
        let mut score = 0.0;
        let mut factors = Vec::new();

        if evidence.properly_aligned {
            score += 0.15;
            factors.push(ConfidenceFactor::new("Properly aligned", 0.15));
        }

        if evidence.within_struct_bounds {
            score += 0.2;
            factors.push(ConfidenceFactor::new("Within struct bounds", 0.2));
        }

        if evidence.type_consistent {
            score += 0.25;
            factors.push(ConfidenceFactor::new("Type consistent", 0.25));
        }

        if evidence.access_pattern_valid {
            score += 0.2;
            factors.push(ConfidenceFactor::new("Access pattern valid", 0.2));
        }

        if evidence.multiple_references {
            score += 0.2;
            factors.push(ConfidenceFactor::new("Multiple references found", 0.2));
        }

        ConfidenceScore {
            score: score.min(1.0),
            level: ConfidenceLevel::from_score(score),
            factors,
        }
    }

    pub fn aggregate_confidence(&self, scores: &[ConfidenceScore]) -> ConfidenceScore {
        if scores.is_empty() {
            return ConfidenceScore {
                score: 0.0,
                level: ConfidenceLevel::VeryLow,
                factors: Vec::new(),
            };
        }

        let avg_score: f64 = scores.iter().map(|s| s.score).sum::<f64>() / scores.len() as f64;

        let min_score = scores.iter()
            .map(|s| s.score)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let combined = avg_score * 0.7 + min_score * 0.3;

        ConfidenceScore {
            score: combined,
            level: ConfidenceLevel::from_score(combined),
            factors: vec![
                ConfidenceFactor::new(&format!("Average of {} scores", scores.len()), avg_score),
                ConfidenceFactor::new("Minimum score influence", min_score * 0.3),
            ],
        }
    }

    pub fn adjust_for_consistency(&self, results: &FinderResults) -> HashMap<String, f64> {
        let mut adjustments = HashMap::new();

        for (name, &addr) in &results.functions {
            let base_confidence = 0.5;

            let xref_count = 1;
            let xref_bonus = (xref_count as f64 * 0.05).min(0.2);

            let adjusted = (base_confidence + xref_bonus).min(1.0);
            adjustments.insert(name.clone(), adjusted);
        }

        adjustments
    }

    pub fn compare_with_history(&self, current: &ConfidenceScore) -> HistoryComparison {
        if self.history.is_empty() {
            return HistoryComparison {
                trend: ConfidenceTrend::Stable,
                average_historical: 0.0,
                current_vs_average: 0.0,
            };
        }

        let avg_historical: f64 = self.history.iter()
            .map(|h| h.score)
            .sum::<f64>() / self.history.len() as f64;

        let diff = current.score - avg_historical;

        let trend = if diff > 0.1 {
            ConfidenceTrend::Improving
        } else if diff < -0.1 {
            ConfidenceTrend::Declining
        } else {
            ConfidenceTrend::Stable
        };

        HistoryComparison {
            trend,
            average_historical: avg_historical,
            current_vs_average: diff,
        }
    }

    pub fn add_to_history(&mut self, name: String, score: f64) {
        self.history.push(HistoricalConfidence {
            name,
            score,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });

        if self.history.len() > 1000 {
            self.history.remove(0);
        }
    }
}

impl Default for ConfidenceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ConfidenceScore {
    pub score: f64,
    pub level: ConfidenceLevel,
    pub factors: Vec<ConfidenceFactor>,
}

impl ConfidenceScore {
    pub fn new(score: f64) -> Self {
        Self {
            score,
            level: ConfidenceLevel::from_score(score),
            factors: Vec::new(),
        }
    }

    pub fn with_factor(mut self, factor: ConfidenceFactor) -> Self {
        self.factors.push(factor);
        self
    }

    pub fn is_high(&self) -> bool {
        matches!(self.level, ConfidenceLevel::High | ConfidenceLevel::VeryHigh)
    }

    pub fn is_acceptable(&self) -> bool {
        self.score >= 0.5
    }

    pub fn as_percentage(&self) -> f64 {
        self.score * 100.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLevel {
    VeryHigh,
    High,
    Medium,
    Low,
    VeryLow,
}

impl ConfidenceLevel {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 {
            ConfidenceLevel::VeryHigh
        } else if score >= 0.75 {
            ConfidenceLevel::High
        } else if score >= 0.5 {
            ConfidenceLevel::Medium
        } else if score >= 0.25 {
            ConfidenceLevel::Low
        } else {
            ConfidenceLevel::VeryLow
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ConfidenceLevel::VeryHigh => "Very High",
            ConfidenceLevel::High => "High",
            ConfidenceLevel::Medium => "Medium",
            ConfidenceLevel::Low => "Low",
            ConfidenceLevel::VeryLow => "Very Low",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfidenceFactor {
    pub name: String,
    pub contribution: f64,
}

impl ConfidenceFactor {
    pub fn new(name: &str, contribution: f64) -> Self {
        Self {
            name: name.to_string(),
            contribution,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfidenceWeights {
    pub valid_prologue: f64,
    pub executable_region: f64,
    pub alignment: f64,
    pub cross_references: f64,
    pub symbol_match: f64,
    pub pattern_match: f64,
    pub xref_validation: f64,
}

impl Default for ConfidenceWeights {
    fn default() -> Self {
        Self {
            valid_prologue: 0.15,
            executable_region: 0.1,
            alignment: 0.05,
            cross_references: 0.15,
            symbol_match: 0.2,
            pattern_match: 0.2,
            xref_validation: 0.15,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FunctionEvidence {
    pub has_valid_prologue: bool,
    pub in_executable_region: bool,
    pub aligned: bool,
    pub has_cross_references: bool,
    pub symbol_matched: bool,
    pub pattern_matched: bool,
    pub xref_validated: bool,
}

impl FunctionEvidence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_prologue(mut self) -> Self {
        self.has_valid_prologue = true;
        self
    }

    pub fn in_executable(mut self) -> Self {
        self.in_executable_region = true;
        self
    }

    pub fn is_aligned(mut self) -> Self {
        self.aligned = true;
        self
    }

    pub fn has_xrefs(mut self) -> Self {
        self.has_cross_references = true;
        self
    }

    pub fn symbol_match(mut self) -> Self {
        self.symbol_matched = true;
        self
    }

    pub fn pattern_match(mut self) -> Self {
        self.pattern_matched = true;
        self
    }

    pub fn xref_valid(mut self) -> Self {
        self.xref_validated = true;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct OffsetEvidence {
    pub properly_aligned: bool,
    pub within_struct_bounds: bool,
    pub type_consistent: bool,
    pub access_pattern_valid: bool,
    pub multiple_references: bool,
}

impl OffsetEvidence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn aligned(mut self) -> Self {
        self.properly_aligned = true;
        self
    }

    pub fn in_bounds(mut self) -> Self {
        self.within_struct_bounds = true;
        self
    }

    pub fn type_ok(mut self) -> Self {
        self.type_consistent = true;
        self
    }

    pub fn access_ok(mut self) -> Self {
        self.access_pattern_valid = true;
        self
    }

    pub fn multi_ref(mut self) -> Self {
        self.multiple_references = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct HistoricalConfidence {
    pub name: String,
    pub score: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct HistoryComparison {
    pub trend: ConfidenceTrend,
    pub average_historical: f64,
    pub current_vs_average: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceTrend {
    Improving,
    Stable,
    Declining,
}
