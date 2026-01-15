// Tue Jan 13 2026 - Alex

use crate::analysis::heuristics::engine::HeuristicMatch;
use std::collections::HashMap;

pub struct HeuristicScorer {
    weights: ScoringWeights,
    history: ScoringHistory,
}

impl HeuristicScorer {
    pub fn new() -> Self {
        Self {
            weights: ScoringWeights::default(),
            history: ScoringHistory::new(),
        }
    }

    pub fn with_weights(weights: ScoringWeights) -> Self {
        Self {
            weights,
            history: ScoringHistory::new(),
        }
    }

    pub fn score_match(&self, m: &HeuristicMatch) -> f64 {
        let mut score = m.confidence;

        score *= self.weights.get_rule_weight(&m.rule);

        if let Some(historical) = self.history.get_accuracy(&m.rule) {
            score *= 0.5 + (historical * 0.5);
        }

        score.clamp(0.0, 1.0)
    }

    pub fn score_matches(&self, matches: &[HeuristicMatch]) -> Vec<ScoredMatch> {
        matches.iter()
            .map(|m| ScoredMatch {
                original: m.clone(),
                final_score: self.score_match(m),
            })
            .collect()
    }

    pub fn aggregate_scores(&self, matches: &[HeuristicMatch]) -> AggregatedScore {
        if matches.is_empty() {
            return AggregatedScore::empty();
        }

        let scores: Vec<f64> = matches.iter()
            .map(|m| self.score_match(m))
            .collect();

        let sum: f64 = scores.iter().sum();
        let mean = sum / scores.len() as f64;

        let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = scores.iter().cloned().fold(f64::INFINITY, f64::min);

        let variance = scores.iter()
            .map(|s| (s - mean).powi(2))
            .sum::<f64>() / scores.len() as f64;
        let std_dev = variance.sqrt();

        let mut sorted = scores.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let weighted_sum: f64 = matches.iter()
            .map(|m| {
                let base = self.score_match(m);
                let weight = self.weights.get_rule_weight(&m.rule);
                base * weight
            })
            .sum();
        let total_weight: f64 = matches.iter()
            .map(|m| self.weights.get_rule_weight(&m.rule))
            .sum();
        let weighted_average = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };

        AggregatedScore {
            count: scores.len(),
            mean,
            median,
            max,
            min,
            std_dev,
            weighted_average,
        }
    }

    pub fn rank_matches(&self, matches: &[HeuristicMatch]) -> Vec<RankedMatch> {
        let mut scored: Vec<_> = matches.iter()
            .enumerate()
            .map(|(i, m)| (i, self.score_match(m)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.iter()
            .enumerate()
            .map(|(rank, (original_idx, score))| RankedMatch {
                rank: rank + 1,
                original_index: *original_idx,
                score: *score,
                match_data: matches[*original_idx].clone(),
            })
            .collect()
    }

    pub fn filter_by_threshold<'a>(&self, matches: &'a [HeuristicMatch], threshold: f64) -> Vec<&'a HeuristicMatch> {
        matches.iter()
            .filter(|m| self.score_match(m) >= threshold)
            .collect()
    }

    pub fn record_outcome(&mut self, rule: &str, was_correct: bool) {
        self.history.record(rule, was_correct);
    }

    pub fn get_rule_accuracy(&self, rule: &str) -> Option<f64> {
        self.history.get_accuracy(rule)
    }

    pub fn update_weight(&mut self, rule: &str, weight: f64) {
        self.weights.set_rule_weight(rule, weight);
    }
}

impl Default for HeuristicScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ScoringWeights {
    rule_weights: HashMap<String, f64>,
    default_weight: f64,
}

impl ScoringWeights {
    pub fn new() -> Self {
        let mut rule_weights = HashMap::new();
        rule_weights.insert("FunctionPrologue".to_string(), 1.0);
        rule_weights.insert("StackAccess".to_string(), 0.8);
        rule_weights.insert("GlobalAccess".to_string(), 0.9);
        rule_weights.insert("StringReference".to_string(), 0.85);
        rule_weights.insert("VTableAccess".to_string(), 0.75);
        rule_weights.insert("LuaStateAccess".to_string(), 0.95);
        rule_weights.insert("ExtraSpaceAccess".to_string(), 0.9);
        rule_weights.insert("ClosureAccess".to_string(), 0.8);
        rule_weights.insert("ProtoAccess".to_string(), 0.8);
        rule_weights.insert("TableAccess".to_string(), 0.75);

        Self {
            rule_weights,
            default_weight: 0.7,
        }
    }

    pub fn get_rule_weight(&self, rule: &str) -> f64 {
        self.rule_weights.get(rule).copied().unwrap_or(self.default_weight)
    }

    pub fn set_rule_weight(&mut self, rule: &str, weight: f64) {
        self.rule_weights.insert(rule.to_string(), weight.clamp(0.0, 2.0));
    }

    pub fn set_default_weight(&mut self, weight: f64) {
        self.default_weight = weight.clamp(0.0, 2.0);
    }
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ScoringHistory {
    outcomes: HashMap<String, RuleOutcomes>,
}

impl ScoringHistory {
    pub fn new() -> Self {
        Self {
            outcomes: HashMap::new(),
        }
    }

    pub fn record(&mut self, rule: &str, was_correct: bool) {
        let entry = self.outcomes.entry(rule.to_string()).or_insert(RuleOutcomes::new());
        if was_correct {
            entry.correct += 1;
        } else {
            entry.incorrect += 1;
        }
    }

    pub fn get_accuracy(&self, rule: &str) -> Option<f64> {
        self.outcomes.get(rule).map(|o| o.accuracy())
    }

    pub fn total_predictions(&self, rule: &str) -> usize {
        self.outcomes.get(rule)
            .map(|o| o.total())
            .unwrap_or(0)
    }

    pub fn clear(&mut self) {
        self.outcomes.clear();
    }

    pub fn all_rules(&self) -> impl Iterator<Item = &String> {
        self.outcomes.keys()
    }
}

impl Default for ScoringHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
struct RuleOutcomes {
    correct: usize,
    incorrect: usize,
}

impl RuleOutcomes {
    fn new() -> Self {
        Self::default()
    }

    fn total(&self) -> usize {
        self.correct + self.incorrect
    }

    fn accuracy(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            0.5
        } else {
            self.correct as f64 / total as f64
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoredMatch {
    pub original: HeuristicMatch,
    pub final_score: f64,
}

#[derive(Debug, Clone)]
pub struct RankedMatch {
    pub rank: usize,
    pub original_index: usize,
    pub score: f64,
    pub match_data: HeuristicMatch,
}

#[derive(Debug, Clone)]
pub struct AggregatedScore {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub max: f64,
    pub min: f64,
    pub std_dev: f64,
    pub weighted_average: f64,
}

impl AggregatedScore {
    pub fn empty() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            median: 0.0,
            max: 0.0,
            min: 0.0,
            std_dev: 0.0,
            weighted_average: 0.0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn summary(&self) -> String {
        format!(
            "Count: {}, Mean: {:.3}, Median: {:.3}, Max: {:.3}, Min: {:.3}, StdDev: {:.3}",
            self.count, self.mean, self.median, self.max, self.min, self.std_dev
        )
    }
}

pub struct ThresholdConfig {
    pub high_confidence: f64,
    pub medium_confidence: f64,
    pub low_confidence: f64,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            high_confidence: 0.85,
            medium_confidence: 0.6,
            low_confidence: 0.4,
        }
    }
}

impl ThresholdConfig {
    pub fn classify(&self, score: f64) -> ConfidenceLevel {
        if score >= self.high_confidence {
            ConfidenceLevel::High
        } else if score >= self.medium_confidence {
            ConfidenceLevel::Medium
        } else if score >= self.low_confidence {
            ConfidenceLevel::Low
        } else {
            ConfidenceLevel::VeryLow
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
    VeryLow,
}

impl ConfidenceLevel {
    pub fn name(&self) -> &'static str {
        match self {
            ConfidenceLevel::High => "High",
            ConfidenceLevel::Medium => "Medium",
            ConfidenceLevel::Low => "Low",
            ConfidenceLevel::VeryLow => "Very Low",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            ConfidenceLevel::High => "\x1b[32m",
            ConfidenceLevel::Medium => "\x1b[33m",
            ConfidenceLevel::Low => "\x1b[31m",
            ConfidenceLevel::VeryLow => "\x1b[90m",
        }
    }
}
