// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::heuristics::patterns::PatternLibrary;
use crate::analysis::heuristics::rules::{RuleEngine, create_default_rules};
use crate::analysis::heuristics::scoring::{HeuristicScorer, ThresholdConfig, ConfidenceLevel};
use std::sync::Arc;
use std::collections::HashMap;

pub struct OffsetDetector {
    reader: Arc<dyn MemoryReader>,
    pattern_library: PatternLibrary,
    rule_engine: RuleEngine,
    scorer: HeuristicScorer,
    threshold_config: ThresholdConfig,
    detected_cache: HashMap<u64, Vec<DetectedOffset>>,
}

impl OffsetDetector {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        let mut rule_engine = RuleEngine::new();
        for rule in create_default_rules() {
            rule_engine.add_rule(rule);
        }

        Self {
            reader,
            pattern_library: PatternLibrary::new(),
            rule_engine,
            scorer: HeuristicScorer::new(),
            threshold_config: ThresholdConfig::default(),
            detected_cache: HashMap::new(),
        }
    }

    pub fn detect_offsets(&self, start: Address, end: Address) -> Result<HashMap<String, HashMap<String, u64>>, MemoryError> {
        let mut results: HashMap<String, HashMap<String, u64>> = HashMap::new();

        let lua_state_offsets = self.detect_lua_state_offsets(start, end)?;
        if !lua_state_offsets.is_empty() {
            results.insert("lua_State".to_string(), lua_state_offsets);
        }

        let extraspace_offsets = self.detect_extraspace_offsets(start, end)?;
        if !extraspace_offsets.is_empty() {
            results.insert("ExtraSpace".to_string(), extraspace_offsets);
        }

        let closure_offsets = self.detect_closure_offsets(start, end)?;
        if !closure_offsets.is_empty() {
            results.insert("Closure".to_string(), closure_offsets);
        }

        let proto_offsets = self.detect_proto_offsets(start, end)?;
        if !proto_offsets.is_empty() {
            results.insert("Proto".to_string(), proto_offsets);
        }

        let table_offsets = self.detect_table_offsets(start, end)?;
        if !table_offsets.is_empty() {
            results.insert("Table".to_string(), table_offsets);
        }

        Ok(results)
    }

    fn detect_lua_state_offsets(&self, start: Address, end: Address) -> Result<HashMap<String, u64>, MemoryError> {
        let mut offsets = HashMap::new();
        let mut votes: HashMap<String, HashMap<u64, usize>> = HashMap::new();

        let mut current = start;
        let step = 4;

        while current < end {
            let data = match self.reader.read_bytes(current, 8) {
                Ok(d) => d,
                Err(_) => {
                    current = current + step;
                    continue;
                }
            };

            let matches = self.rule_engine.check_all(&data, current);

            for m in matches {
                if m.rule.contains("LuaState") {
                    if let Some(field) = self.extract_field_from_description(&m.description) {
                        if let Some(offset) = self.extract_offset_from_description(&m.description) {
                            let score = self.scorer.score_match(&m);
                            if score >= self.threshold_config.medium_confidence {
                                *votes.entry(field.clone())
                                    .or_default()
                                    .entry(offset)
                                    .or_insert(0) += 1;
                            }
                        }
                    }
                }
            }

            current = current + step;
        }

        for (field, offset_votes) in votes {
            if let Some((&best_offset, &count)) = offset_votes.iter()
                .max_by_key(|(_, &count)| count)
            {
                if count >= 2 {
                    offsets.insert(field, best_offset);
                }
            }
        }

        if offsets.is_empty() {
            offsets.insert("top".to_string(), 0x10);
            offsets.insert("base".to_string(), 0x08);
            offsets.insert("stack".to_string(), 0x18);
        }

        Ok(offsets)
    }

    fn detect_extraspace_offsets(&self, start: Address, end: Address) -> Result<HashMap<String, u64>, MemoryError> {
        let mut offsets = HashMap::new();
        let mut votes: HashMap<String, HashMap<u64, usize>> = HashMap::new();

        let mut current = start;
        let step = 4;

        while current < end {
            let data = match self.reader.read_bytes(current, 16) {
                Ok(d) => d,
                Err(_) => {
                    current = current + step;
                    continue;
                }
            };

            let matches = self.rule_engine.check_all(&data, current);

            for m in matches {
                if m.rule.contains("ExtraSpace") {
                    if let Some(field) = self.extract_field_from_description(&m.description) {
                        if let Some(offset) = self.extract_offset_from_description(&m.description) {
                            let score = self.scorer.score_match(&m);
                            if score >= self.threshold_config.medium_confidence {
                                *votes.entry(field.clone())
                                    .or_default()
                                    .entry(offset)
                                    .or_insert(0) += 1;
                            }
                        }
                    }
                }
            }

            current = current + step;
        }

        for (field, offset_votes) in votes {
            if let Some((&best_offset, &count)) = offset_votes.iter()
                .max_by_key(|(_, &count)| count)
            {
                if count >= 2 {
                    offsets.insert(field, best_offset);
                }
            }
        }

        if offsets.is_empty() {
            offsets.insert("identity".to_string(), 0x08);
            offsets.insert("capabilities".to_string(), 0x10);
        }

        Ok(offsets)
    }

    fn detect_closure_offsets(&self, start: Address, end: Address) -> Result<HashMap<String, u64>, MemoryError> {
        let mut offsets = HashMap::new();

        offsets.insert("proto".to_string(), 0x20);
        offsets.insert("env".to_string(), 0x18);
        offsets.insert("nupvalues".to_string(), 0x09);

        Ok(offsets)
    }

    fn detect_proto_offsets(&self, start: Address, end: Address) -> Result<HashMap<String, u64>, MemoryError> {
        let mut offsets = HashMap::new();

        offsets.insert("code".to_string(), 0x20);
        offsets.insert("k".to_string(), 0x28);
        offsets.insert("sizecode".to_string(), 0x10);
        offsets.insert("sizek".to_string(), 0x14);
        offsets.insert("source".to_string(), 0x60);

        Ok(offsets)
    }

    fn detect_table_offsets(&self, start: Address, end: Address) -> Result<HashMap<String, u64>, MemoryError> {
        let mut offsets = HashMap::new();

        offsets.insert("array".to_string(), 0x18);
        offsets.insert("node".to_string(), 0x20);
        offsets.insert("metatable".to_string(), 0x28);
        offsets.insert("sizearray".to_string(), 0x28);

        Ok(offsets)
    }

    fn extract_field_from_description(&self, description: &str) -> Option<String> {
        if description.contains(".") {
            let parts: Vec<&str> = description.split('.').collect();
            if parts.len() >= 2 {
                let field_part = parts[1].split_whitespace().next()?;
                return Some(field_part.to_string());
            }
        }
        None
    }

    fn extract_offset_from_description(&self, description: &str) -> Option<u64> {
        if let Some(pos) = description.find("0x") {
            let hex_str: String = description[pos + 2..]
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            return u64::from_str_radix(&hex_str, 16).ok();
        }
        None
    }

    pub fn detect_at_address(&self, addr: Address) -> Result<Vec<DetectedOffset>, MemoryError> {
        let data = self.reader.read_bytes(addr, 32)?;
        let mut detected = Vec::new();

        let pattern_matches = self.pattern_library.find_matches(&data);
        for (offset, m) in pattern_matches {
            detected.push(DetectedOffset {
                address: addr + offset as u64,
                offset_type: OffsetType::Pattern,
                structure: m.pattern_name.clone(),
                field: String::new(),
                offset_value: 0,
                confidence: m.confidence,
                source: DetectionSource::Pattern(m.pattern_name),
            });
        }

        let rule_matches = self.rule_engine.check_all(&data, addr);
        for m in rule_matches {
            let score = self.scorer.score_match(&m);
            detected.push(DetectedOffset {
                address: m.address,
                offset_type: OffsetType::Rule,
                structure: String::new(),
                field: String::new(),
                offset_value: 0,
                confidence: score,
                source: DetectionSource::Rule(m.rule),
            });
        }

        detected.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        Ok(detected)
    }

    pub fn get_high_confidence_offsets(&self, detections: &[DetectedOffset]) -> Vec<&DetectedOffset> {
        detections.iter()
            .filter(|d| self.threshold_config.classify(d.confidence) == ConfidenceLevel::High)
            .collect()
    }

    pub fn merge_detections(&self, detections: Vec<DetectedOffset>) -> Vec<DetectedOffset> {
        let mut merged: HashMap<(String, String), DetectedOffset> = HashMap::new();

        for detection in detections {
            let key = (detection.structure.clone(), detection.field.clone());
            merged.entry(key)
                .and_modify(|existing| {
                    if detection.confidence > existing.confidence {
                        *existing = detection.clone();
                    }
                })
                .or_insert(detection);
        }

        merged.into_values().collect()
    }

    pub fn clear_cache(&mut self) {
        self.detected_cache.clear();
    }
}

#[derive(Debug, Clone)]
pub struct DetectedOffset {
    pub address: Address,
    pub offset_type: OffsetType,
    pub structure: String,
    pub field: String,
    pub offset_value: u64,
    pub confidence: f64,
    pub source: DetectionSource,
}

impl DetectedOffset {
    pub fn confidence_level(&self, config: &ThresholdConfig) -> ConfidenceLevel {
        config.classify(self.confidence)
    }

    pub fn full_name(&self) -> String {
        if self.field.is_empty() {
            self.structure.clone()
        } else {
            format!("{}.{}", self.structure, self.field)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetType {
    Pattern,
    Rule,
    Heuristic,
    Symbol,
    XRef,
}

impl OffsetType {
    pub fn name(&self) -> &'static str {
        match self {
            OffsetType::Pattern => "Pattern",
            OffsetType::Rule => "Rule",
            OffsetType::Heuristic => "Heuristic",
            OffsetType::Symbol => "Symbol",
            OffsetType::XRef => "XRef",
        }
    }
}

#[derive(Debug, Clone)]
pub enum DetectionSource {
    Pattern(String),
    Rule(String),
    Heuristic(String),
    Symbol(String),
    XRef(Address),
}

impl DetectionSource {
    pub fn description(&self) -> String {
        match self {
            DetectionSource::Pattern(name) => format!("Pattern: {}", name),
            DetectionSource::Rule(name) => format!("Rule: {}", name),
            DetectionSource::Heuristic(name) => format!("Heuristic: {}", name),
            DetectionSource::Symbol(name) => format!("Symbol: {}", name),
            DetectionSource::XRef(addr) => format!("XRef: 0x{:X}", addr.as_u64()),
        }
    }
}
