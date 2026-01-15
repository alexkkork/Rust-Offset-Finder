// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::finders::result::FinderResults;
use std::collections::HashMap;

pub struct ResultAggregator {
    dedup_threshold: f64,
    merge_strategy: MergeStrategy,
}

impl ResultAggregator {
    pub fn new() -> Self {
        Self {
            dedup_threshold: 0.9,
            merge_strategy: MergeStrategy::HighestConfidence,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.dedup_threshold = threshold;
        self
    }

    pub fn with_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.merge_strategy = strategy;
        self
    }

    pub fn aggregate(&self, results_list: Vec<FinderResults>) -> FinderResults {
        let mut aggregated = FinderResults::new();

        for results in results_list {
            self.merge_into(&mut aggregated, results);
        }

        self.deduplicate(&mut aggregated);
        aggregated
    }

    fn merge_into(&self, target: &mut FinderResults, source: FinderResults) {
        match self.merge_strategy {
            MergeStrategy::HighestConfidence => {
                for (name, addr) in source.functions {
                    target.functions.entry(name).or_insert(addr);
                }

                for (struct_name, fields) in source.structure_offsets {
                    let entry = target.structure_offsets.entry(struct_name).or_default();
                    for (field, offset) in fields {
                        entry.entry(field).or_insert(offset);
                    }
                }

                for (name, addr) in source.classes {
                    target.classes.entry(name).or_insert(addr);
                }

                for (class, props) in source.properties {
                    let entry = target.properties.entry(class).or_default();
                    for (prop, offset) in props {
                        entry.entry(prop).or_insert(offset);
                    }
                }

                for (class, methods) in source.methods {
                    let entry = target.methods.entry(class).or_default();
                    for (method, addr) in methods {
                        entry.entry(method).or_insert(addr);
                    }
                }

                for (name, value) in source.constants {
                    target.constants.entry(name).or_insert(value);
                }
            }
            MergeStrategy::FirstFound => {
                target.merge(source);
            }
            MergeStrategy::Average => {
                target.merge(source);
            }
        }
    }

    fn deduplicate(&self, results: &mut FinderResults) {
        let mut seen_addresses: HashMap<u64, String> = HashMap::new();
        let mut to_remove = Vec::new();

        for (name, addr) in results.functions.iter() {
            if let Some(existing) = seen_addresses.get(&addr.as_u64()) {
                if name.len() > existing.len() {
                    to_remove.push(existing.clone());
                    seen_addresses.insert(addr.as_u64(), name.clone());
                } else {
                    to_remove.push(name.clone());
                }
            } else {
                seen_addresses.insert(addr.as_u64(), name.clone());
            }
        }

        for name in to_remove {
            results.functions.remove(&name);
        }
    }

    pub fn filter_by_confidence(&self, results: &mut FinderResults, min_confidence: f64) {
    }

    pub fn sort_by_address(&self, results: &FinderResults) -> Vec<(String, Address)> {
        let mut sorted: Vec<_> = results.functions.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        sorted.sort_by_key(|(_, addr)| addr.as_u64());
        sorted
    }

    pub fn group_by_category(&self, results: &FinderResults) -> HashMap<String, FinderResults> {
        let mut groups: HashMap<String, FinderResults> = HashMap::new();

        for (name, addr) in &results.functions {
            let category = self.categorize_function(name);
            let group = groups.entry(category).or_insert_with(FinderResults::new);
            group.functions.insert(name.clone(), *addr);
        }

        for (struct_name, fields) in &results.structure_offsets {
            let category = self.categorize_structure(struct_name);
            let group = groups.entry(category).or_insert_with(FinderResults::new);
            group.structure_offsets.insert(struct_name.clone(), fields.clone());
        }

        groups
    }

    fn categorize_function(&self, name: &str) -> String {
        let name_lower = name.to_lowercase();

        if name_lower.starts_with("lua_") || name_lower.starts_with("luau_") {
            "lua_api".to_string()
        } else if name_lower.starts_with("lual_") {
            "lua_aux".to_string()
        } else if name_lower.contains("task") {
            "task".to_string()
        } else if name_lower.contains("script") {
            "script".to_string()
        } else if name_lower.contains("instance") {
            "instance".to_string()
        } else if name_lower.contains("rbx") || name_lower.contains("roblox") {
            "roblox".to_string()
        } else {
            "other".to_string()
        }
    }

    fn categorize_structure(&self, name: &str) -> String {
        let name_lower = name.to_lowercase();

        if name_lower.contains("lua") || name_lower.contains("state") {
            "lua".to_string()
        } else if name_lower.contains("proto") || name_lower.contains("closure") {
            "function".to_string()
        } else if name_lower.contains("table") {
            "table".to_string()
        } else if name_lower.contains("string") {
            "string".to_string()
        } else if name_lower.contains("extra") || name_lower.contains("sctx") {
            "extraspace".to_string()
        } else {
            "other".to_string()
        }
    }
}

impl Default for ResultAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    HighestConfidence,
    FirstFound,
    Average,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        MergeStrategy::HighestConfidence
    }
}

pub struct AggregationStatistics {
    pub total_before: usize,
    pub total_after: usize,
    pub duplicates_removed: usize,
    pub conflicts_resolved: usize,
}

impl AggregationStatistics {
    pub fn new() -> Self {
        Self {
            total_before: 0,
            total_after: 0,
            duplicates_removed: 0,
            conflicts_resolved: 0,
        }
    }

    pub fn reduction_percentage(&self) -> f64 {
        if self.total_before == 0 {
            0.0
        } else {
            ((self.total_before - self.total_after) as f64 / self.total_before as f64) * 100.0
        }
    }
}

impl Default for AggregationStatistics {
    fn default() -> Self {
        Self::new()
    }
}
