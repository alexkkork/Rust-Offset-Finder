// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use std::collections::HashMap;
use std::sync::Arc;

pub struct PatternAnalyzer {
    reader: Arc<dyn MemoryReader>,
    patterns: Vec<AnalysisPattern>,
    results: HashMap<String, Vec<PatternResult>>,
}

#[derive(Debug, Clone)]
pub struct AnalysisPattern {
    pub name: String,
    pub bytes: Vec<u8>,
    pub mask: Vec<u8>,
    pub description: String,
    pub category: PatternCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternCategory {
    Function,
    String,
    VTable,
    Class,
    Constant,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PatternResult {
    pub pattern_name: String,
    pub address: Address,
    pub matched_bytes: Vec<u8>,
    pub context: PatternContext,
}

#[derive(Debug, Clone, Default)]
pub struct PatternContext {
    pub preceding_bytes: Vec<u8>,
    pub following_bytes: Vec<u8>,
    pub xrefs: Vec<Address>,
}

impl PatternAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        let mut analyzer = Self {
            reader,
            patterns: Vec::new(),
            results: HashMap::new(),
        };
        analyzer.register_default_patterns();
        analyzer
    }

    fn register_default_patterns(&mut self) {
        self.add_pattern(AnalysisPattern {
            name: "lua_pushvalue".to_string(),
            bytes: vec![0xFD, 0x7B, 0xBF, 0xA9, 0xFD, 0x03, 0x00, 0x91],
            mask: vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xFF],
            description: "Lua pushvalue function prologue".to_string(),
            category: PatternCategory::Function,
        });

        self.add_pattern(AnalysisPattern {
            name: "roblox_instance".to_string(),
            bytes: vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
            mask: vec![0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF],
            description: "Roblox instance marker".to_string(),
            category: PatternCategory::Class,
        });

        self.add_pattern(AnalysisPattern {
            name: "vtable_start".to_string(),
            bytes: vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00],
            mask: vec![0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF],
            description: "VTable start marker".to_string(),
            category: PatternCategory::VTable,
        });
    }

    pub fn add_pattern(&mut self, pattern: AnalysisPattern) {
        self.patterns.push(pattern);
    }

    pub fn scan(&mut self, start: Address, end: Address) -> Result<Vec<PatternResult>, MemoryError> {
        let mut all_results = Vec::new();

        for pattern in &self.patterns.clone() {
            let results = self.scan_for_pattern(pattern, start, end)?;
            for result in results {
                self.results
                    .entry(result.pattern_name.clone())
                    .or_insert_with(Vec::new)
                    .push(result.clone());
                all_results.push(result);
            }
        }

        Ok(all_results)
    }

    pub fn scan_for_pattern(&self, pattern: &AnalysisPattern, start: Address, end: Address) -> Result<Vec<PatternResult>, MemoryError> {
        let mut results = Vec::new();
        let pattern_len = pattern.bytes.len();
        let scan_size = (end.as_u64() - start.as_u64()) as usize;

        let data = self.reader.read_bytes(start, scan_size)?;

        for i in 0..data.len().saturating_sub(pattern_len) {
            if self.matches_at(&data[i..], &pattern.bytes, &pattern.mask) {
                let addr = start + i as u64;
                let matched_bytes = data[i..i + pattern_len].to_vec();

                let context = self.build_context(addr, &data, i, pattern_len)?;

                results.push(PatternResult {
                    pattern_name: pattern.name.clone(),
                    address: addr,
                    matched_bytes,
                    context,
                });
            }
        }

        Ok(results)
    }

    fn matches_at(&self, data: &[u8], pattern: &[u8], mask: &[u8]) -> bool {
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

    fn build_context(&self, _addr: Address, data: &[u8], offset: usize, _pattern_len: usize) -> Result<PatternContext, MemoryError> {
        let context_size = 32;

        let preceding_start = offset.saturating_sub(context_size);
        let preceding_bytes = data[preceding_start..offset].to_vec();

        let following_start = offset + _pattern_len;
        let following_end = (following_start + context_size).min(data.len());
        let following_bytes = data[following_start..following_end].to_vec();

        Ok(PatternContext {
            preceding_bytes,
            following_bytes,
            xrefs: Vec::new(),
        })
    }

    pub fn get_results(&self, pattern_name: &str) -> Vec<&PatternResult> {
        self.results
            .get(pattern_name)
            .map(|r| r.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_all_results(&self) -> Vec<&PatternResult> {
        self.results.values().flatten().collect()
    }

    pub fn get_results_by_category(&self, category: PatternCategory) -> Vec<&PatternResult> {
        let pattern_names: Vec<_> = self.patterns
            .iter()
            .filter(|p| p.category == category)
            .map(|p| p.name.as_str())
            .collect();

        self.results
            .iter()
            .filter(|(name, _)| pattern_names.contains(&name.as_str()))
            .flat_map(|(_, results)| results.iter())
            .collect()
    }

    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn result_count(&self) -> usize {
        self.results.values().map(|v| v.len()).sum()
    }

    pub fn find_similar_patterns(&self, data: &[u8], threshold: f64) -> Vec<(&AnalysisPattern, f64)> {
        let mut similar = Vec::new();

        for pattern in &self.patterns {
            if pattern.bytes.len() != data.len() {
                continue;
            }

            let mut match_count = 0;
            let mut total_count = 0;

            for i in 0..pattern.bytes.len() {
                if pattern.mask[i] != 0 {
                    total_count += 1;
                    if data[i] == pattern.bytes[i] {
                        match_count += 1;
                    }
                }
            }

            if total_count > 0 {
                let similarity = match_count as f64 / total_count as f64;
                if similarity >= threshold {
                    similar.push((pattern, similarity));
                }
            }
        }

        similar.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similar
    }

    pub fn create_pattern_from_bytes(name: &str, bytes: &[u8], wildcards: &[usize]) -> AnalysisPattern {
        let mut mask = vec![0xFF; bytes.len()];
        for &pos in wildcards {
            if pos < mask.len() {
                mask[pos] = 0x00;
            }
        }

        AnalysisPattern {
            name: name.to_string(),
            bytes: bytes.to_vec(),
            mask,
            description: String::new(),
            category: PatternCategory::Unknown,
        }
    }
}

impl AnalysisPattern {
    pub fn new(name: &str, bytes: Vec<u8>) -> Self {
        let mask = vec![0xFF; bytes.len()];
        Self {
            name: name.to_string(),
            bytes,
            mask,
            description: String::new(),
            category: PatternCategory::Unknown,
        }
    }

    pub fn with_mask(mut self, mask: Vec<u8>) -> Self {
        self.mask = mask;
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_category(mut self, cat: PatternCategory) -> Self {
        self.category = cat;
        self
    }

    pub fn from_ida_pattern(name: &str, pattern_str: &str) -> Option<Self> {
        let mut bytes = Vec::new();
        let mut mask = Vec::new();

        for part in pattern_str.split_whitespace() {
            if part == "?" || part == "??" {
                bytes.push(0x00);
                mask.push(0x00);
            } else if let Ok(byte) = u8::from_str_radix(part, 16) {
                bytes.push(byte);
                mask.push(0xFF);
            } else {
                return None;
            }
        }

        Some(Self {
            name: name.to_string(),
            bytes,
            mask,
            description: String::new(),
            category: PatternCategory::Unknown,
        })
    }

    pub fn to_ida_pattern(&self) -> String {
        let mut parts = Vec::new();
        for i in 0..self.bytes.len() {
            if self.mask[i] == 0x00 {
                parts.push("??".to_string());
            } else {
                parts.push(format!("{:02X}", self.bytes[i]));
            }
        }
        parts.join(" ")
    }
}

pub fn find_byte_sequence(reader: &dyn MemoryReader, sequence: &[u8], start: Address, end: Address) -> Result<Vec<Address>, MemoryError> {
    let mut results = Vec::new();
    let scan_size = (end.as_u64() - start.as_u64()) as usize;
    let data = reader.read_bytes(start, scan_size)?;

    for i in 0..data.len().saturating_sub(sequence.len()) {
        if &data[i..i + sequence.len()] == sequence {
            results.push(start + i as u64);
        }
    }

    Ok(results)
}

pub fn find_dword(reader: &dyn MemoryReader, value: u32, start: Address, end: Address) -> Result<Vec<Address>, MemoryError> {
    find_byte_sequence(reader, &value.to_le_bytes(), start, end)
}

pub fn find_qword(reader: &dyn MemoryReader, value: u64, start: Address, end: Address) -> Result<Vec<Address>, MemoryError> {
    find_byte_sequence(reader, &value.to_le_bytes(), start, end)
}
