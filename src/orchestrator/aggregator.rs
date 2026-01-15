// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::finders::result::FinderResults;
use std::collections::HashMap;

pub struct ResultAggregator {
    pending_results: Vec<FinderResults>,
    weights: AggregationWeights,
}

impl ResultAggregator {
    pub fn new() -> Self {
        Self {
            pending_results: Vec::new(),
            weights: AggregationWeights::default(),
        }
    }

    pub fn with_weights(mut self, weights: AggregationWeights) -> Self {
        self.weights = weights;
        self
    }

    pub fn add(&mut self, results: FinderResults) {
        self.pending_results.push(results);
    }

    pub fn aggregate(&self) -> FinderResults {
        let mut aggregated = FinderResults::new();

        let mut function_candidates: HashMap<String, Vec<Address>> = HashMap::new();
        let mut offset_candidates: HashMap<String, HashMap<String, Vec<u64>>> = HashMap::new();
        let mut class_candidates: HashMap<String, Vec<Address>> = HashMap::new();
        let mut constant_candidates: HashMap<String, Vec<u64>> = HashMap::new();

        for result in &self.pending_results {
            for (name, addr) in &result.functions {
                function_candidates.entry(name.clone())
                    .or_default()
                    .push(*addr);
            }

            for (struct_name, offsets) in &result.structure_offsets {
                let struct_entry = offset_candidates.entry(struct_name.clone())
                    .or_default();

                for (field_name, offset) in offsets {
                    struct_entry.entry(field_name.clone())
                        .or_default()
                        .push(*offset);
                }
            }

            for (name, addr) in &result.classes {
                class_candidates.entry(name.clone())
                    .or_default()
                    .push(*addr);
            }

            for (name, value) in &result.constants {
                constant_candidates.entry(name.clone())
                    .or_default()
                    .push(*value);
            }
        }

        for (name, addrs) in function_candidates {
            if let Some(best) = self.select_best_address(&addrs) {
                aggregated.functions.insert(name, best);
            }
        }

        for (struct_name, fields) in offset_candidates {
            let mut struct_offsets = HashMap::new();

            for (field_name, offsets) in fields {
                if let Some(best) = self.select_best_offset(&offsets) {
                    struct_offsets.insert(field_name, best);
                }
            }

            if !struct_offsets.is_empty() {
                aggregated.structure_offsets.insert(struct_name, struct_offsets);
            }
        }

        for (name, addrs) in class_candidates {
            if let Some(best) = self.select_best_address(&addrs) {
                aggregated.classes.insert(name, best);
            }
        }

        for (name, values) in constant_candidates {
            if let Some(best) = self.select_best_constant(&values) {
                aggregated.constants.insert(name, best);
            }
        }

        aggregated
    }

    fn select_best_address(&self, candidates: &[Address]) -> Option<Address> {
        if candidates.is_empty() {
            return None;
        }

        if candidates.len() == 1 {
            return Some(candidates[0]);
        }

        let mut frequency: HashMap<u64, usize> = HashMap::new();
        for addr in candidates {
            *frequency.entry(addr.as_u64()).or_insert(0) += 1;
        }

        frequency.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(addr, _)| Address::new(addr))
    }

    fn select_best_offset(&self, candidates: &[u64]) -> Option<u64> {
        if candidates.is_empty() {
            return None;
        }

        if candidates.len() == 1 {
            return Some(candidates[0]);
        }

        let mut frequency: HashMap<u64, usize> = HashMap::new();
        for &offset in candidates {
            *frequency.entry(offset).or_insert(0) += 1;
        }

        frequency.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(offset, _)| offset)
    }

    fn select_best_constant(&self, candidates: &[u64]) -> Option<u64> {
        if candidates.is_empty() {
            return None;
        }

        if candidates.len() == 1 {
            return Some(candidates[0]);
        }

        let mut frequency: HashMap<u64, usize> = HashMap::new();
        for &value in candidates {
            *frequency.entry(value).or_insert(0) += 1;
        }

        frequency.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(value, _)| value)
    }

    pub fn statistics(&self) -> AggregationStatistics {
        let mut total_functions = 0;
        let mut total_offsets = 0;
        let mut total_classes = 0;
        let mut total_constants = 0;

        for result in &self.pending_results {
            total_functions += result.functions.len();
            total_offsets += result.structure_offsets.values()
                .map(|m| m.len())
                .sum::<usize>();
            total_classes += result.classes.len();
            total_constants += result.constants.len();
        }

        let aggregated = self.aggregate();

        AggregationStatistics {
            input_sources: self.pending_results.len(),
            total_function_candidates: total_functions,
            total_offset_candidates: total_offsets,
            total_class_candidates: total_classes,
            total_constant_candidates: total_constants,
            aggregated_functions: aggregated.functions.len(),
            aggregated_offsets: aggregated.structure_offsets.values()
                .map(|m| m.len())
                .sum(),
            aggregated_classes: aggregated.classes.len(),
            aggregated_constants: aggregated.constants.len(),
        }
    }
}

impl Default for ResultAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AggregationWeights {
    pub symbol_weight: f64,
    pub pattern_weight: f64,
    pub xref_weight: f64,
    pub heuristic_weight: f64,
}

impl Default for AggregationWeights {
    fn default() -> Self {
        Self {
            symbol_weight: 1.0,
            pattern_weight: 0.9,
            xref_weight: 0.8,
            heuristic_weight: 0.6,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AggregationStatistics {
    pub input_sources: usize,
    pub total_function_candidates: usize,
    pub total_offset_candidates: usize,
    pub total_class_candidates: usize,
    pub total_constant_candidates: usize,
    pub aggregated_functions: usize,
    pub aggregated_offsets: usize,
    pub aggregated_classes: usize,
    pub aggregated_constants: usize,
}

impl AggregationStatistics {
    pub fn reduction_rate(&self) -> f64 {
        let total_input = self.total_function_candidates +
            self.total_offset_candidates +
            self.total_class_candidates +
            self.total_constant_candidates;

        let total_output = self.aggregated_functions +
            self.aggregated_offsets +
            self.aggregated_classes +
            self.aggregated_constants;

        if total_input == 0 {
            0.0
        } else {
            1.0 - (total_output as f64 / total_input as f64)
        }
    }
}

pub struct WeightedAggregator {
    results: Vec<(FinderResults, f64)>,
}

impl WeightedAggregator {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add(&mut self, results: FinderResults, weight: f64) {
        self.results.push((results, weight));
    }

    pub fn aggregate(&self) -> FinderResults {
        let mut aggregated = FinderResults::new();

        let mut function_scores: HashMap<String, HashMap<u64, f64>> = HashMap::new();

        for (result, weight) in &self.results {
            for (name, addr) in &result.functions {
                function_scores.entry(name.clone())
                    .or_default()
                    .entry(addr.as_u64())
                    .and_modify(|score| *score += weight)
                    .or_insert(*weight);
            }
        }

        for (name, addr_scores) in function_scores {
            if let Some((best_addr, _)) = addr_scores.into_iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            {
                aggregated.functions.insert(name, Address::new(best_addr));
            }
        }

        aggregated
    }
}

impl Default for WeightedAggregator {
    fn default() -> Self {
        Self::new()
    }
}
