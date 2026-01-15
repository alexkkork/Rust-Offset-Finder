// Tue Jan 13 2026 - Alex

use crate::finders::result::FinderResults;
use std::collections::HashMap;

pub struct ConfidenceScorer {
    weights: ConfidenceWeights,
}

impl ConfidenceScorer {
    pub fn new() -> Self {
        Self {
            weights: ConfidenceWeights::default(),
        }
    }

    pub fn with_weights(weights: ConfidenceWeights) -> Self {
        Self { weights }
    }

    pub fn calculate_all(&self, results: &FinderResults) -> HashMap<String, f64> {
        let mut scores = HashMap::new();

        for (name, _addr) in &results.functions {
            let score = self.score_function(name, results);
            scores.insert(format!("func:{}", name), score);
        }

        for (struct_name, fields) in &results.structure_offsets {
            for (field, offset) in fields {
                let score = self.score_offset(struct_name, field, *offset, results);
                scores.insert(format!("offset:{}.{}", struct_name, field), score);
            }
        }

        scores
    }

    fn score_function(&self, name: &str, results: &FinderResults) -> f64 {
        let mut score = 0.5;

        if name.starts_with("lua_") || name.starts_with("luaL_") || name.starts_with("luau_") {
            score += 0.2;
        }

        if results.functions.len() > 10 {
            score += 0.1;
        }

        let name_lower = name.to_lowercase();
        let known_functions = [
            "lua_pushvalue", "lua_settop", "lua_gettop", "lua_pcall",
            "lua_newthread", "lua_pushstring", "lua_getfield",
            "luau_load", "lua_call", "lua_createtable",
        ];
        if known_functions.iter().any(|&f| name_lower.contains(f)) {
            score += 0.15;
        }

        (score as f64).min(1.0)
    }

    fn score_offset(&self, struct_name: &str, field: &str, offset: u64, results: &FinderResults) -> f64 {
        let mut score = 0.5;

        if offset % 8 == 0 {
            score += 0.1;
        } else if offset % 4 == 0 {
            score += 0.05;
        }

        if offset < 0x200 {
            score += 0.1;
        } else if offset < 0x1000 {
            score += 0.05;
        }

        let struct_lower = struct_name.to_lowercase();
        if struct_lower.contains("lua") || struct_lower.contains("state") ||
           struct_lower.contains("proto") || struct_lower.contains("closure") {
            score += 0.1;
        }

        let field_lower = field.to_lowercase();
        let known_fields = ["top", "base", "stack", "identity", "capabilities", "proto", "env"];
        if known_fields.iter().any(|&f| field_lower.contains(f)) {
            score += 0.1;
        }

        if let Some(fields) = results.structure_offsets.get(struct_name) {
            if fields.len() > 3 {
                score += 0.05;
            }
        }

        (score as f64).min(1.0)
    }

    pub fn score_overall(&self, results: &FinderResults) -> f64 {
        let mut total_score = 0.0;
        let mut count = 0;

        for name in results.functions.keys() {
            total_score += self.score_function(name, results);
            count += 1;
        }

        for (struct_name, fields) in &results.structure_offsets {
            for (field, offset) in fields {
                total_score += self.score_offset(struct_name, field, *offset, results);
                count += 1;
            }
        }

        if count == 0 {
            0.0
        } else {
            total_score / count as f64
        }
    }

    pub fn filter_by_confidence(&self, results: &FinderResults, min_confidence: f64) -> FinderResults {
        let mut filtered = FinderResults::new();

        for (name, addr) in &results.functions {
            if self.score_function(name, results) >= min_confidence {
                filtered.functions.insert(name.clone(), *addr);
            }
        }

        for (struct_name, fields) in &results.structure_offsets {
            for (field, offset) in fields {
                if self.score_offset(struct_name, field, *offset, results) >= min_confidence {
                    filtered.structure_offsets
                        .entry(struct_name.clone())
                        .or_default()
                        .insert(field.clone(), *offset);
                }
            }
        }

        filtered.classes = results.classes.clone();
        filtered.properties = results.properties.clone();
        filtered.methods = results.methods.clone();
        filtered.constants = results.constants.clone();

        filtered
    }

    pub fn get_high_confidence(&self, results: &FinderResults) -> FinderResults {
        self.filter_by_confidence(results, 0.8)
    }

    pub fn get_medium_confidence(&self, results: &FinderResults) -> FinderResults {
        self.filter_by_confidence(results, 0.5)
    }
}

impl Default for ConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ConfidenceWeights {
    pub pattern_match: f64,
    pub symbol_match: f64,
    pub xref_match: f64,
    pub heuristic_match: f64,
    pub alignment_bonus: f64,
    pub known_name_bonus: f64,
}

impl Default for ConfidenceWeights {
    fn default() -> Self {
        Self {
            pattern_match: 0.3,
            symbol_match: 0.4,
            xref_match: 0.25,
            heuristic_match: 0.15,
            alignment_bonus: 0.1,
            known_name_bonus: 0.2,
        }
    }
}

impl ConfidenceWeights {
    pub fn high_precision() -> Self {
        Self {
            pattern_match: 0.4,
            symbol_match: 0.5,
            xref_match: 0.3,
            heuristic_match: 0.1,
            alignment_bonus: 0.05,
            known_name_bonus: 0.15,
        }
    }

    pub fn balanced() -> Self {
        Self::default()
    }

    pub fn high_recall() -> Self {
        Self {
            pattern_match: 0.2,
            symbol_match: 0.3,
            xref_match: 0.2,
            heuristic_match: 0.25,
            alignment_bonus: 0.15,
            known_name_bonus: 0.25,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfidenceReport {
    pub scores: HashMap<String, f64>,
    pub average_score: f64,
    pub high_confidence_count: usize,
    pub medium_confidence_count: usize,
    pub low_confidence_count: usize,
}

impl ConfidenceReport {
    pub fn from_scores(scores: HashMap<String, f64>) -> Self {
        let average_score = if scores.is_empty() {
            0.0
        } else {
            scores.values().sum::<f64>() / scores.len() as f64
        };

        let high_confidence_count = scores.values().filter(|&&s| s >= 0.8).count();
        let medium_confidence_count = scores.values().filter(|&&s| s >= 0.5 && s < 0.8).count();
        let low_confidence_count = scores.values().filter(|&&s| s < 0.5).count();

        Self {
            scores,
            average_score,
            high_confidence_count,
            medium_confidence_count,
            low_confidence_count,
        }
    }

    pub fn format_report(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("=== Confidence Report ===\n"));
        output.push_str(&format!("Average Score: {:.2}\n", self.average_score));
        output.push_str(&format!("High Confidence (>=0.8): {}\n", self.high_confidence_count));
        output.push_str(&format!("Medium Confidence (0.5-0.8): {}\n", self.medium_confidence_count));
        output.push_str(&format!("Low Confidence (<0.5): {}\n", self.low_confidence_count));

        output
    }
}
