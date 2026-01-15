// Tue Jan 13 2026 - Alex

use crate::config::Config;
use crate::finders::result::FinderResults;
use crate::output::manager::OutputManager;
use crate::output::OffsetOutput;
use std::collections::HashMap;

pub struct OutputFinalizer {
    format_addresses: bool,
    include_metadata: bool,
    sort_output: bool,
}

impl OutputFinalizer {
    pub fn new() -> Self {
        Self {
            format_addresses: true,
            include_metadata: true,
            sort_output: true,
        }
    }

    pub fn with_format_addresses(mut self, format: bool) -> Self {
        self.format_addresses = format;
        self
    }

    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    pub fn with_sorting(mut self, sort: bool) -> Self {
        self.sort_output = sort;
        self
    }

    pub fn finalize(&self, results: FinderResults, config: &Config) -> OutputManager {
        let mut output = OutputManager::new();

        for (name, addr) in results.functions {
            output.add_function(name, addr.as_u64());
        }

        for (struct_name, fields) in results.structure_offsets {
            for (field, offset) in fields {
                output.add_structure_offset(struct_name.clone(), field, offset);
            }
        }

        for (name, addr) in results.classes {
            output.add_class(name, addr.as_u64());
        }

        for (class, props) in results.properties {
            for (prop, offset) in props {
                output.add_property(class.clone(), prop, offset);
            }
        }

        for (class, methods) in results.methods {
            for (method, addr) in methods {
                output.add_method(class.clone(), method, addr.as_u64());
            }
        }

        for (name, value) in results.constants {
            output.add_constant(name, value);
        }

        if self.include_metadata {
            output.set_metadata("version", "1.0.0");
            output.set_metadata("generator", "roblox-offset-generator");
            output.set_metadata("target", &config.target_binary.to_string_lossy());
        }

        output
    }

    pub fn to_offset_output(&self, results: FinderResults) -> OffsetOutput {
        let mut functions = HashMap::new();
        for (name, addr) in results.functions {
            functions.insert(name, addr.as_u64());
        }

        let mut structure_offsets = HashMap::new();
        for (struct_name, fields) in results.structure_offsets {
            structure_offsets.insert(struct_name, fields);
        }

        OffsetOutput {
            functions,
            structure_offsets,
        }
    }

    pub fn format_for_display(&self, results: &FinderResults) -> String {
        let mut output = String::new();

        output.push_str("=== Functions ===\n");
        let mut funcs: Vec<_> = results.functions.iter().collect();
        if self.sort_output {
            funcs.sort_by_key(|(name, _)| name.to_lowercase());
        }
        for (name, addr) in funcs {
            if self.format_addresses {
                output.push_str(&format!("  {}: 0x{:X}\n", name, addr.as_u64()));
            } else {
                output.push_str(&format!("  {}: {}\n", name, addr.as_u64()));
            }
        }

        output.push_str("\n=== Structure Offsets ===\n");
        let mut structs: Vec<_> = results.structure_offsets.iter().collect();
        if self.sort_output {
            structs.sort_by_key(|(name, _)| name.to_lowercase());
        }
        for (struct_name, fields) in structs {
            output.push_str(&format!("  {}:\n", struct_name));
            let mut fields_sorted: Vec<_> = fields.iter().collect();
            if self.sort_output {
                fields_sorted.sort_by_key(|(_, &offset)| offset);
            }
            for (field, offset) in fields_sorted {
                if self.format_addresses {
                    output.push_str(&format!("    {}: 0x{:X}\n", field, offset));
                } else {
                    output.push_str(&format!("    {}: {}\n", field, offset));
                }
            }
        }

        if !results.classes.is_empty() {
            output.push_str("\n=== Classes ===\n");
            let mut classes: Vec<_> = results.classes.iter().collect();
            if self.sort_output {
                classes.sort_by_key(|(name, _)| name.to_lowercase());
            }
            for (name, addr) in classes {
                if self.format_addresses {
                    output.push_str(&format!("  {}: 0x{:X}\n", name, addr.as_u64()));
                } else {
                    output.push_str(&format!("  {}: {}\n", name, addr.as_u64()));
                }
            }
        }

        if !results.constants.is_empty() {
            output.push_str("\n=== Constants ===\n");
            let mut constants: Vec<_> = results.constants.iter().collect();
            if self.sort_output {
                constants.sort_by_key(|(name, _)| name.to_lowercase());
            }
            for (name, value) in constants {
                if self.format_addresses {
                    output.push_str(&format!("  {}: 0x{:X}\n", name, value));
                } else {
                    output.push_str(&format!("  {}: {}\n", name, value));
                }
            }
        }

        output
    }

    pub fn create_summary(&self, results: &FinderResults) -> FinalizationSummary {
        FinalizationSummary {
            function_count: results.functions.len(),
            structure_count: results.structure_offsets.len(),
            offset_count: results.structure_offsets.values()
                .map(|m| m.len())
                .sum(),
            class_count: results.classes.len(),
            property_count: results.properties.values()
                .map(|m| m.len())
                .sum(),
            method_count: results.methods.values()
                .map(|m| m.len())
                .sum(),
            constant_count: results.constants.len(),
        }
    }
}

impl Default for OutputFinalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct FinalizationSummary {
    pub function_count: usize,
    pub structure_count: usize,
    pub offset_count: usize,
    pub class_count: usize,
    pub property_count: usize,
    pub method_count: usize,
    pub constant_count: usize,
}

impl FinalizationSummary {
    pub fn total_items(&self) -> usize {
        self.function_count
            + self.offset_count
            + self.class_count
            + self.property_count
            + self.method_count
            + self.constant_count
    }

    pub fn display(&self) -> String {
        format!(
            "Functions: {}, Structures: {} ({} offsets), Classes: {}, Properties: {}, Methods: {}, Constants: {}",
            self.function_count,
            self.structure_count,
            self.offset_count,
            self.class_count,
            self.property_count,
            self.method_count,
            self.constant_count
        )
    }
}
