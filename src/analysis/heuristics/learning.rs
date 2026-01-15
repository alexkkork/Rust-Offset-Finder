// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::heuristics::patterns::{HeuristicPattern, InstructionPattern, PatternType};
use crate::analysis::heuristics::scoring::ScoringWeights;
use std::sync::Arc;
use std::collections::HashMap;

pub struct PatternLearner {
    reader: Arc<dyn MemoryReader>,
    samples: Vec<LearningSample>,
    learned_patterns: Vec<LearnedPattern>,
    config: LearningConfig,
}

impl PatternLearner {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            samples: Vec::new(),
            learned_patterns: Vec::new(),
            config: LearningConfig::default(),
        }
    }

    pub fn add_positive_sample(&mut self, addr: Address, label: &str) -> Result<(), MemoryError> {
        let data = self.reader.read_bytes(addr, self.config.sample_size)?;
        self.samples.push(LearningSample {
            address: addr,
            data,
            label: label.to_string(),
            is_positive: true,
        });
        Ok(())
    }

    pub fn add_negative_sample(&mut self, addr: Address, label: &str) -> Result<(), MemoryError> {
        let data = self.reader.read_bytes(addr, self.config.sample_size)?;
        self.samples.push(LearningSample {
            address: addr,
            data,
            label: label.to_string(),
            is_positive: false,
        });
        Ok(())
    }

    pub fn learn_patterns(&mut self) -> Vec<LearnedPattern> {
        let positive_samples: Vec<_> = self.samples.iter()
            .filter(|s| s.is_positive)
            .collect();

        if positive_samples.is_empty() {
            return Vec::new();
        }

        let mut patterns = Vec::new();

        let by_label = self.group_by_label(&positive_samples);

        for (label, samples) in by_label {
            if samples.len() >= self.config.min_samples {
                if let Some(pattern) = self.extract_pattern(&label, &samples) {
                    patterns.push(pattern);
                }
            }
        }

        self.learned_patterns = patterns.clone();
        patterns
    }

    fn group_by_label<'a>(&self, samples: &[&'a LearningSample]) -> HashMap<String, Vec<&'a LearningSample>> {
        let mut groups: HashMap<String, Vec<&LearningSample>> = HashMap::new();

        for sample in samples {
            groups.entry(sample.label.clone())
                .or_default()
                .push(sample);
        }

        groups
    }

    fn extract_pattern(&self, label: &str, samples: &[&LearningSample]) -> Option<LearnedPattern> {
        if samples.is_empty() {
            return None;
        }

        let min_len = samples.iter()
            .map(|s| s.data.len())
            .min()
            .unwrap_or(0);

        if min_len < 4 {
            return None;
        }

        let mut common_mask = vec![0xFFu8; min_len];
        let mut common_value = samples[0].data[..min_len].to_vec();

        for sample in samples.iter().skip(1) {
            for i in 0..min_len {
                if sample.data[i] != common_value[i] {
                    common_mask[i] = 0x00;
                }
            }
        }

        let specificity = common_mask.iter()
            .filter(|&&b| b != 0)
            .count() as f64 / min_len as f64;

        if specificity < self.config.min_specificity {
            return None;
        }

        let negative_samples: Vec<_> = self.samples.iter()
            .filter(|s| !s.is_positive)
            .collect();

        let mut false_positives = 0;
        for neg in &negative_samples {
            if self.matches_pattern(&neg.data, &common_value, &common_mask) {
                false_positives += 1;
            }
        }

        let false_positive_rate = if negative_samples.is_empty() {
            0.0
        } else {
            false_positives as f64 / negative_samples.len() as f64
        };

        let confidence = (specificity * (1.0 - false_positive_rate)).max(0.0);

        Some(LearnedPattern {
            name: format!("learned_{}", label),
            label: label.to_string(),
            pattern_bytes: common_value,
            mask_bytes: common_mask,
            specificity,
            confidence,
            sample_count: samples.len(),
            false_positive_rate,
        })
    }

    fn matches_pattern(&self, data: &[u8], pattern: &[u8], mask: &[u8]) -> bool {
        if data.len() < pattern.len() {
            return false;
        }

        for i in 0..pattern.len() {
            if mask[i] != 0 && data[i] != pattern[i] {
                return false;
            }
        }

        true
    }

    pub fn to_heuristic_patterns(&self) -> Vec<HeuristicPattern> {
        self.learned_patterns.iter()
            .filter(|p| p.confidence >= self.config.min_confidence)
            .map(|p| p.to_heuristic_pattern())
            .collect()
    }

    pub fn evaluate(&self, test_samples: &[LearningSample]) -> EvaluationResult {
        let mut true_positives = 0;
        let mut false_positives = 0;
        let mut true_negatives = 0;
        let mut false_negatives = 0;

        for sample in test_samples {
            let matched = self.learned_patterns.iter()
                .any(|p| {
                    p.label == sample.label &&
                    self.matches_pattern(&sample.data, &p.pattern_bytes, &p.mask_bytes)
                });

            match (sample.is_positive, matched) {
                (true, true) => true_positives += 1,
                (true, false) => false_negatives += 1,
                (false, true) => false_positives += 1,
                (false, false) => true_negatives += 1,
            }
        }

        let total = true_positives + false_positives + true_negatives + false_negatives;
        let accuracy = if total > 0 {
            (true_positives + true_negatives) as f64 / total as f64
        } else {
            0.0
        };

        let precision = if true_positives + false_positives > 0 {
            true_positives as f64 / (true_positives + false_positives) as f64
        } else {
            0.0
        };

        let recall = if true_positives + false_negatives > 0 {
            true_positives as f64 / (true_positives + false_negatives) as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };

        EvaluationResult {
            true_positives,
            false_positives,
            true_negatives,
            false_negatives,
            accuracy,
            precision,
            recall,
            f1_score,
        }
    }

    pub fn refine_weights(&self, evaluation: &EvaluationResult) -> ScoringWeights {
        let mut weights = ScoringWeights::new();

        for pattern in &self.learned_patterns {
            let base_weight = pattern.confidence;
            let adjusted_weight = base_weight * evaluation.precision;
            weights.set_rule_weight(&pattern.name, adjusted_weight);
        }

        weights
    }

    pub fn clear_samples(&mut self) {
        self.samples.clear();
    }

    pub fn clear_patterns(&mut self) {
        self.learned_patterns.clear();
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    pub fn pattern_count(&self) -> usize {
        self.learned_patterns.len()
    }

    pub fn configure(&mut self, config: LearningConfig) {
        self.config = config;
    }
}

#[derive(Debug, Clone)]
pub struct LearningSample {
    pub address: Address,
    pub data: Vec<u8>,
    pub label: String,
    pub is_positive: bool,
}

#[derive(Debug, Clone)]
pub struct LearnedPattern {
    pub name: String,
    pub label: String,
    pub pattern_bytes: Vec<u8>,
    pub mask_bytes: Vec<u8>,
    pub specificity: f64,
    pub confidence: f64,
    pub sample_count: usize,
    pub false_positive_rate: f64,
}

impl LearnedPattern {
    pub fn to_heuristic_pattern(&self) -> HeuristicPattern {
        let mut pattern = HeuristicPattern::new(&self.name, &format!("Learned pattern for {}", self.label))
            .with_type(PatternType::Generic)
            .with_confidence(self.confidence);

        for i in (0..self.pattern_bytes.len()).step_by(4) {
            if i + 4 <= self.pattern_bytes.len() {
                let value = u32::from_le_bytes([
                    self.pattern_bytes[i],
                    self.pattern_bytes[i + 1],
                    self.pattern_bytes[i + 2],
                    self.pattern_bytes[i + 3],
                ]);

                let mask = u32::from_le_bytes([
                    self.mask_bytes[i],
                    self.mask_bytes[i + 1],
                    self.mask_bytes[i + 2],
                    self.mask_bytes[i + 3],
                ]);

                pattern = pattern.with_instruction(
                    InstructionPattern::new(&format!("learned_{}", i / 4), mask, value & mask)
                );
            }
        }

        pattern
    }

    pub fn to_hex_string(&self) -> String {
        self.pattern_bytes.iter()
            .zip(self.mask_bytes.iter())
            .map(|(b, m)| {
                if *m == 0xFF {
                    format!("{:02X}", b)
                } else if *m == 0x00 {
                    "??".to_string()
                } else {
                    format!("{:02X}", b & m)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub true_positives: usize,
    pub false_positives: usize,
    pub true_negatives: usize,
    pub false_negatives: usize,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
}

impl EvaluationResult {
    pub fn summary(&self) -> String {
        format!(
            "Accuracy: {:.2}%, Precision: {:.2}%, Recall: {:.2}%, F1: {:.2}%",
            self.accuracy * 100.0,
            self.precision * 100.0,
            self.recall * 100.0,
            self.f1_score * 100.0
        )
    }

    pub fn confusion_matrix(&self) -> String {
        format!(
            "            Predicted\n            +    -\nActual +   {:4} {:4}\n       -   {:4} {:4}",
            self.true_positives,
            self.false_negatives,
            self.false_positives,
            self.true_negatives
        )
    }
}

#[derive(Debug, Clone)]
pub struct LearningConfig {
    pub sample_size: usize,
    pub min_samples: usize,
    pub min_specificity: f64,
    pub min_confidence: f64,
    pub max_false_positive_rate: f64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            sample_size: 64,
            min_samples: 3,
            min_specificity: 0.3,
            min_confidence: 0.5,
            max_false_positive_rate: 0.1,
        }
    }
}

pub struct IncrementalLearner {
    base_learner: PatternLearner,
    version: u32,
    history: Vec<LearningSnapshot>,
}

impl IncrementalLearner {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            base_learner: PatternLearner::new(reader),
            version: 0,
            history: Vec::new(),
        }
    }

    pub fn add_sample(&mut self, addr: Address, label: &str, is_positive: bool) -> Result<(), MemoryError> {
        if is_positive {
            self.base_learner.add_positive_sample(addr, label)
        } else {
            self.base_learner.add_negative_sample(addr, label)
        }
    }

    pub fn update(&mut self) -> Vec<LearnedPattern> {
        let snapshot = LearningSnapshot {
            version: self.version,
            sample_count: self.base_learner.sample_count(),
            pattern_count: self.base_learner.pattern_count(),
        };
        self.history.push(snapshot);

        self.version += 1;
        self.base_learner.learn_patterns()
    }

    pub fn get_version(&self) -> u32 {
        self.version
    }

    pub fn get_history(&self) -> &[LearningSnapshot] {
        &self.history
    }
}

#[derive(Debug, Clone)]
pub struct LearningSnapshot {
    pub version: u32,
    pub sample_count: usize,
    pub pattern_count: usize,
}
