// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::finders::result::FinderResults;
use crate::validation::report::ValidationReport;
use std::collections::HashMap;

pub struct OffsetFinalizer {
    filters: Vec<Box<dyn ResultFilter>>,
    transformers: Vec<Box<dyn ResultTransformer>>,
    min_confidence: f64,
}

impl OffsetFinalizer {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            transformers: Vec::new(),
            min_confidence: 0.5,
        }
    }

    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = confidence;
        self
    }

    pub fn add_filter<F: ResultFilter + 'static>(&mut self, filter: F) {
        self.filters.push(Box::new(filter));
    }

    pub fn add_transformer<T: ResultTransformer + 'static>(&mut self, transformer: T) {
        self.transformers.push(Box::new(transformer));
    }

    pub fn finalize(&self, mut results: FinderResults, validation: &ValidationReport) -> FinderResults {
        results = self.apply_filters(results, validation);

        results = self.apply_transformers(results);

        results = self.apply_final_cleanup(results);

        results
    }

    fn apply_filters(&self, mut results: FinderResults, validation: &ValidationReport) -> FinderResults {
        let invalid_functions: Vec<String> = results.functions.keys()
            .filter(|name| {
                validation.get_function_result(name)
                    .map(|r| !r.valid)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        for name in invalid_functions {
            results.functions.remove(&name);
        }

        let invalid_structures: Vec<(String, String)> = results.structure_offsets.iter()
            .flat_map(|(struct_name, fields)| {
                fields.keys()
                    .filter(|field_name| {
                        validation.get_structure_result(struct_name, field_name)
                            .map(|r| !r.valid)
                            .unwrap_or(false)
                    })
                    .map(|field_name| (struct_name.clone(), field_name.clone()))
                    .collect::<Vec<_>>()
            })
            .collect();

        for (struct_name, field_name) in invalid_structures {
            if let Some(fields) = results.structure_offsets.get_mut(&struct_name) {
                fields.remove(&field_name);
            }
        }

        for filter in &self.filters {
            results = filter.filter(results);
        }

        results
    }

    fn apply_transformers(&self, mut results: FinderResults) -> FinderResults {
        for transformer in &self.transformers {
            results = transformer.transform(results);
        }

        results
    }

    fn apply_final_cleanup(&self, mut results: FinderResults) -> FinderResults {
        results.functions.retain(|_, addr| addr.as_u64() != 0);

        results.structure_offsets.retain(|_, fields| !fields.is_empty());

        results.classes.retain(|_, addr| addr.as_u64() != 0);

        let mut seen_addresses: HashMap<u64, Vec<String>> = HashMap::new();
        for (name, addr) in &results.functions {
            seen_addresses.entry(addr.as_u64())
                .or_default()
                .push(name.clone());
        }

        for (_, names) in seen_addresses {
            if names.len() > 1 {
                log::warn!("Multiple functions at same address: {:?}", names);
            }
        }

        results
    }

    pub fn generate_output(&self, results: &FinderResults) -> FinalizedOutput {
        let mut output = FinalizedOutput::new();

        for (name, addr) in &results.functions {
            output.functions.push(FinalizedFunction {
                name: name.clone(),
                address: format!("0x{:016X}", addr.as_u64()),
                category: self.categorize_function(name),
            });
        }

        for (struct_name, fields) in &results.structure_offsets {
            let mut finalized_struct = FinalizedStructure {
                name: struct_name.clone(),
                offsets: Vec::new(),
            };

            for (field_name, offset) in fields {
                finalized_struct.offsets.push(FinalizedOffset {
                    field_name: field_name.clone(),
                    offset: *offset,
                    hex_offset: format!("0x{:X}", offset),
                });
            }

            finalized_struct.offsets.sort_by_key(|o| o.offset);
            output.structures.push(finalized_struct);
        }

        for (name, addr) in &results.classes {
            output.classes.push(FinalizedClass {
                name: name.clone(),
                address: format!("0x{:016X}", addr.as_u64()),
            });
        }

        for (name, value) in &results.constants {
            output.constants.push(FinalizedConstant {
                name: name.clone(),
                value: *value,
                hex_value: format!("0x{:X}", value),
            });
        }

        output.functions.sort_by(|a, b| a.name.cmp(&b.name));
        output.structures.sort_by(|a, b| a.name.cmp(&b.name));
        output.classes.sort_by(|a, b| a.name.cmp(&b.name));
        output.constants.sort_by(|a, b| a.name.cmp(&b.name));

        output
    }

    fn categorize_function(&self, name: &str) -> String {
        if name.starts_with("lua_") {
            "Lua API".to_string()
        } else if name.starts_with("luau_") {
            "Luau".to_string()
        } else if name.starts_with("luaL_") {
            "Lua Auxiliary".to_string()
        } else if name.contains("Roblox") || name.contains("rbx_") {
            "Roblox".to_string()
        } else {
            "Other".to_string()
        }
    }
}

impl Default for OffsetFinalizer {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ResultFilter {
    fn filter(&self, results: FinderResults) -> FinderResults;
}

pub trait ResultTransformer {
    fn transform(&self, results: FinderResults) -> FinderResults;
}

pub struct ConfidenceFilter {
    min_confidence: f64,
}

impl ConfidenceFilter {
    pub fn new(min_confidence: f64) -> Self {
        Self { min_confidence }
    }
}

impl ResultFilter for ConfidenceFilter {
    fn filter(&self, results: FinderResults) -> FinderResults {
        results
    }
}

pub struct AlignmentFilter;

impl ResultFilter for AlignmentFilter {
    fn filter(&self, mut results: FinderResults) -> FinderResults {
        results.functions.retain(|_, addr| addr.as_u64() % 4 == 0);
        results
    }
}

pub struct NamingTransformer;

impl ResultTransformer for NamingTransformer {
    fn transform(&self, mut results: FinderResults) -> FinderResults {
        let mut renamed: HashMap<String, Address> = HashMap::new();

        for (name, addr) in results.functions.drain() {
            let normalized = name.trim_start_matches('_').to_string();
            renamed.insert(normalized, addr);
        }

        results.functions = renamed;
        results
    }
}

#[derive(Debug, Clone)]
pub struct FinalizedOutput {
    pub functions: Vec<FinalizedFunction>,
    pub structures: Vec<FinalizedStructure>,
    pub classes: Vec<FinalizedClass>,
    pub constants: Vec<FinalizedConstant>,
}

impl FinalizedOutput {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            structures: Vec::new(),
            classes: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn total_items(&self) -> usize {
        self.functions.len() +
        self.structures.iter().map(|s| s.offsets.len()).sum::<usize>() +
        self.classes.len() +
        self.constants.len()
    }
}

impl Default for FinalizedOutput {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FinalizedFunction {
    pub name: String,
    pub address: String,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct FinalizedStructure {
    pub name: String,
    pub offsets: Vec<FinalizedOffset>,
}

#[derive(Debug, Clone)]
pub struct FinalizedOffset {
    pub field_name: String,
    pub offset: u64,
    pub hex_offset: String,
}

#[derive(Debug, Clone)]
pub struct FinalizedClass {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct FinalizedConstant {
    pub name: String,
    pub value: u64,
    pub hex_value: String,
}
