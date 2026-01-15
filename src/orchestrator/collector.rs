// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::finders::result::FinderResults;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use parking_lot::RwLock;

pub struct ResultCollector {
    results: Arc<RwLock<CollectedResults>>,
    source_tracking: Arc<RwLock<HashMap<String, Vec<ResultSource>>>>,
    conflict_resolution: ConflictResolutionStrategy,
}

impl ResultCollector {
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(CollectedResults::new())),
            source_tracking: Arc::new(RwLock::new(HashMap::new())),
            conflict_resolution: ConflictResolutionStrategy::HighestConfidence,
        }
    }

    pub fn with_strategy(mut self, strategy: ConflictResolutionStrategy) -> Self {
        self.conflict_resolution = strategy;
        self
    }

    pub fn collect(&self, source: &str, results: FinderResults, confidence: f64) {
        let mut collected = self.results.write();
        let mut tracking = self.source_tracking.write();

        for (name, addr) in results.functions {
            let source_info = ResultSource {
                source_name: source.to_string(),
                confidence,
                timestamp: std::time::SystemTime::now(),
            };

            tracking.entry(format!("func:{}", name))
                .or_default()
                .push(source_info);

            self.add_function(&mut collected, name, addr, confidence);
        }

        for (struct_name, offsets) in results.structure_offsets {
            for (field_name, offset) in offsets {
                let source_info = ResultSource {
                    source_name: source.to_string(),
                    confidence,
                    timestamp: std::time::SystemTime::now(),
                };

                tracking.entry(format!("struct:{}:{}", struct_name, field_name))
                    .or_default()
                    .push(source_info);

                self.add_structure_offset(&mut collected, struct_name.clone(), field_name, offset, confidence);
            }
        }

        for (name, addr) in results.classes {
            let source_info = ResultSource {
                source_name: source.to_string(),
                confidence,
                timestamp: std::time::SystemTime::now(),
            };

            tracking.entry(format!("class:{}", name))
                .or_default()
                .push(source_info);

            self.add_class(&mut collected, name, addr, confidence);
        }

        for (name, value) in results.constants {
            let source_info = ResultSource {
                source_name: source.to_string(),
                confidence,
                timestamp: std::time::SystemTime::now(),
            };

            tracking.entry(format!("const:{}", name))
                .or_default()
                .push(source_info);

            self.add_constant(&mut collected, name, value, confidence);
        }
    }

    fn add_function(&self, collected: &mut CollectedResults, name: String, addr: Address, confidence: f64) {
        if let Some(existing) = collected.functions.get(&name) {
            let should_replace = match self.conflict_resolution {
                ConflictResolutionStrategy::HighestConfidence => {
                    confidence > existing.confidence
                }
                ConflictResolutionStrategy::FirstWins => false,
                ConflictResolutionStrategy::LastWins => true,
                ConflictResolutionStrategy::Consensus => {
                    collected.function_votes.entry(name.clone())
                        .or_default()
                        .push((addr, confidence));
                    false
                }
            };

            if should_replace {
                collected.functions.insert(name, CollectedFunction {
                    address: addr,
                    confidence,
                    source_count: 1,
                });
            } else {
                let entry = collected.functions.get_mut(&name).unwrap();
                entry.source_count += 1;
            }
        } else {
            collected.functions.insert(name, CollectedFunction {
                address: addr,
                confidence,
                source_count: 1,
            });
        }
    }

    fn add_structure_offset(&self, collected: &mut CollectedResults, struct_name: String, field_name: String, offset: u64, confidence: f64) {
        let struct_offsets = collected.structure_offsets
            .entry(struct_name)
            .or_default();

        if let Some(existing) = struct_offsets.get(&field_name) {
            let should_replace = match self.conflict_resolution {
                ConflictResolutionStrategy::HighestConfidence => {
                    confidence > existing.confidence
                }
                ConflictResolutionStrategy::FirstWins => false,
                ConflictResolutionStrategy::LastWins => true,
                ConflictResolutionStrategy::Consensus => false,
            };

            if should_replace {
                struct_offsets.insert(field_name, CollectedOffset {
                    offset,
                    confidence,
                    source_count: 1,
                });
            } else {
                let entry = struct_offsets.get_mut(&field_name).unwrap();
                entry.source_count += 1;
            }
        } else {
            struct_offsets.insert(field_name, CollectedOffset {
                offset,
                confidence,
                source_count: 1,
            });
        }
    }

    fn add_class(&self, collected: &mut CollectedResults, name: String, addr: Address, confidence: f64) {
        if let Some(existing) = collected.classes.get(&name) {
            let should_replace = match self.conflict_resolution {
                ConflictResolutionStrategy::HighestConfidence => {
                    confidence > existing.confidence
                }
                ConflictResolutionStrategy::FirstWins => false,
                ConflictResolutionStrategy::LastWins => true,
                ConflictResolutionStrategy::Consensus => false,
            };

            if should_replace {
                collected.classes.insert(name, CollectedClass {
                    address: addr,
                    confidence,
                    source_count: 1,
                });
            }
        } else {
            collected.classes.insert(name, CollectedClass {
                address: addr,
                confidence,
                source_count: 1,
            });
        }
    }

    fn add_constant(&self, collected: &mut CollectedResults, name: String, value: u64, confidence: f64) {
        if let Some(existing) = collected.constants.get(&name) {
            let should_replace = match self.conflict_resolution {
                ConflictResolutionStrategy::HighestConfidence => {
                    confidence > existing.confidence
                }
                ConflictResolutionStrategy::FirstWins => false,
                ConflictResolutionStrategy::LastWins => true,
                ConflictResolutionStrategy::Consensus => false,
            };

            if should_replace {
                collected.constants.insert(name, CollectedConstant {
                    value,
                    confidence,
                    source_count: 1,
                });
            }
        } else {
            collected.constants.insert(name, CollectedConstant {
                value,
                confidence,
                source_count: 1,
            });
        }
    }

    pub fn finalize(&self) -> FinderResults {
        let collected = self.results.read();
        let mut results = FinderResults::new();

        for (name, func) in &collected.functions {
            results.functions.insert(name.clone(), func.address);
        }

        for (struct_name, offsets) in &collected.structure_offsets {
            let mut offset_map = HashMap::new();
            for (field_name, offset_info) in offsets {
                offset_map.insert(field_name.clone(), offset_info.offset);
            }
            results.structure_offsets.insert(struct_name.clone(), offset_map);
        }

        for (name, class) in &collected.classes {
            results.classes.insert(name.clone(), class.address);
        }

        for (name, constant) in &collected.constants {
            results.constants.insert(name.clone(), constant.value);
        }

        results
    }

    pub fn get_conflicts(&self) -> Vec<Conflict> {
        let tracking = self.source_tracking.read();
        let mut conflicts = Vec::new();

        for (key, sources) in tracking.iter() {
            if sources.len() > 1 {
                let confidences: Vec<f64> = sources.iter().map(|s| s.confidence).collect();
                let max_conf = confidences.iter().cloned().fold(f64::MIN, f64::max);
                let min_conf = confidences.iter().cloned().fold(f64::MAX, f64::min);

                if max_conf - min_conf > 0.2 {
                    conflicts.push(Conflict {
                        key: key.clone(),
                        sources: sources.iter().map(|s| s.source_name.clone()).collect(),
                        confidence_variance: max_conf - min_conf,
                    });
                }
            }
        }

        conflicts
    }

    pub fn statistics(&self) -> CollectorStatistics {
        let collected = self.results.read();

        CollectorStatistics {
            total_functions: collected.functions.len(),
            total_structure_offsets: collected.structure_offsets.values()
                .map(|m| m.len())
                .sum(),
            total_classes: collected.classes.len(),
            total_constants: collected.constants.len(),
            average_function_confidence: if collected.functions.is_empty() {
                0.0
            } else {
                collected.functions.values()
                    .map(|f| f.confidence)
                    .sum::<f64>() / collected.functions.len() as f64
            },
            multi_source_count: collected.functions.values()
                .filter(|f| f.source_count > 1)
                .count(),
        }
    }
}

impl Default for ResultCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct CollectedResults {
    functions: HashMap<String, CollectedFunction>,
    structure_offsets: HashMap<String, HashMap<String, CollectedOffset>>,
    classes: HashMap<String, CollectedClass>,
    constants: HashMap<String, CollectedConstant>,
    function_votes: HashMap<String, Vec<(Address, f64)>>,
}

impl CollectedResults {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            structure_offsets: HashMap::new(),
            classes: HashMap::new(),
            constants: HashMap::new(),
            function_votes: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct CollectedFunction {
    address: Address,
    confidence: f64,
    source_count: usize,
}

#[derive(Debug, Clone)]
struct CollectedOffset {
    offset: u64,
    confidence: f64,
    source_count: usize,
}

#[derive(Debug, Clone)]
struct CollectedClass {
    address: Address,
    confidence: f64,
    source_count: usize,
}

#[derive(Debug, Clone)]
struct CollectedConstant {
    value: u64,
    confidence: f64,
    source_count: usize,
}

#[derive(Debug, Clone)]
struct ResultSource {
    source_name: String,
    confidence: f64,
    timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictResolutionStrategy {
    HighestConfidence,
    FirstWins,
    LastWins,
    Consensus,
}

#[derive(Debug, Clone)]
pub struct Conflict {
    pub key: String,
    pub sources: Vec<String>,
    pub confidence_variance: f64,
}

#[derive(Debug, Clone)]
pub struct CollectorStatistics {
    pub total_functions: usize,
    pub total_structure_offsets: usize,
    pub total_classes: usize,
    pub total_constants: usize,
    pub average_function_confidence: f64,
    pub multi_source_count: usize,
}
