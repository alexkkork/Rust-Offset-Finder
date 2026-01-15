// Tue Jan 13 2026 - Alex

use crate::output::{OffsetOutput, FunctionOffset, StructureOffsets, ClassOffset, PropertyOffset, MethodOffset, ConstantOffset, ConstantValue};
use std::collections::HashMap;

pub struct OutputFormatter {
    address_format: AddressFormat,
    include_confidence: bool,
    include_category: bool,
    max_name_width: usize,
    alignment_char: char,
    group_by_category: bool,
    sort_order: SortOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFormat {
    Hex16,
    Hex12,
    Hex8,
    Decimal,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    ByName,
    ByAddress,
    ByConfidence,
    ByCategory,
}

impl OutputFormatter {
    pub fn new() -> Self {
        Self {
            address_format: AddressFormat::Hex16,
            include_confidence: true,
            include_category: true,
            max_name_width: 50,
            alignment_char: ' ',
            group_by_category: false,
            sort_order: SortOrder::ByName,
        }
    }

    pub fn with_address_format(mut self, format: AddressFormat) -> Self {
        self.address_format = format;
        self
    }

    pub fn with_confidence(mut self, include: bool) -> Self {
        self.include_confidence = include;
        self
    }

    pub fn with_category(mut self, include: bool) -> Self {
        self.include_category = include;
        self
    }

    pub fn with_max_name_width(mut self, width: usize) -> Self {
        self.max_name_width = width;
        self
    }

    pub fn with_grouping(mut self, group: bool) -> Self {
        self.group_by_category = group;
        self
    }

    pub fn with_sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = order;
        self
    }

    pub fn format_address(&self, address: u64) -> String {
        match self.address_format {
            AddressFormat::Hex16 => format!("0x{:016x}", address),
            AddressFormat::Hex12 => format!("0x{:012x}", address),
            AddressFormat::Hex8 => format!("0x{:08x}", address),
            AddressFormat::Decimal => format!("{}", address),
            AddressFormat::Both => format!("0x{:016x} ({})", address, address),
        }
    }

    pub fn format_function(&self, name: &str, func: &FunctionOffset) -> String {
        let mut parts = Vec::new();

        let truncated_name = if name.len() > self.max_name_width {
            format!("{}...", &name[..self.max_name_width - 3])
        } else {
            name.to_string()
        };

        parts.push(format!("{:<width$}", truncated_name, width = self.max_name_width));
        parts.push(self.format_address(func.address));

        if self.include_confidence {
            parts.push(format!("{:>6.1}%", func.confidence * 100.0));
        }

        if self.include_category {
            parts.push(format!("[{}]", func.category));
        }

        parts.join(&format!("{}", self.alignment_char))
    }

    pub fn format_function_list(&self, functions: &HashMap<String, FunctionOffset>) -> String {
        let mut lines = Vec::new();
        let mut sorted: Vec<_> = functions.iter().collect();

        match self.sort_order {
            SortOrder::ByName => sorted.sort_by(|a, b| a.0.cmp(b.0)),
            SortOrder::ByAddress => sorted.sort_by(|a, b| a.1.address.cmp(&b.1.address)),
            SortOrder::ByConfidence => sorted.sort_by(|a, b| b.1.confidence.partial_cmp(&a.1.confidence).unwrap()),
            SortOrder::ByCategory => sorted.sort_by(|a, b| a.1.category.cmp(&b.1.category)),
        }

        if self.group_by_category {
            let mut by_category: HashMap<&str, Vec<_>> = HashMap::new();
            for (name, func) in &sorted {
                by_category.entry(func.category.as_str())
                    .or_insert_with(Vec::new)
                    .push((*name, *func));
            }

            let mut categories: Vec<_> = by_category.keys().collect();
            categories.sort();

            for category in categories {
                lines.push(format!("\n=== {} ===", category.to_uppercase()));
                for (name, func) in &by_category[category] {
                    lines.push(self.format_function(name, func));
                }
            }
        } else {
            for (name, func) in sorted {
                lines.push(self.format_function(name, func));
            }
        }

        lines.join("\n")
    }

    pub fn format_structure(&self, name: &str, structure: &StructureOffsets) -> String {
        let mut lines = Vec::new();
        lines.push(format!("struct {} {{  // size: {}, align: {}", name, structure.size, structure.alignment));

        let mut fields: Vec<_> = structure.fields.iter().collect();
        fields.sort_by_key(|(_, f)| f.offset);

        for (field_name, field) in fields {
            lines.push(format!(
                "    /* +0x{:04x} */ {} {};  // {} bytes",
                field.offset,
                field.field_type,
                field_name,
                field.size
            ));
        }

        lines.push("};".to_string());
        lines.join("\n")
    }

    pub fn format_structure_list(&self, structures: &HashMap<String, StructureOffsets>) -> String {
        let mut lines = Vec::new();
        let mut sorted: Vec<_> = structures.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        for (name, structure) in sorted {
            lines.push(self.format_structure(name, structure));
            lines.push(String::new());
        }

        lines.join("\n")
    }

    pub fn format_class(&self, class: &ClassOffset) -> String {
        let mut lines = Vec::new();

        let mut header = format!("class {}", class.name);
        if let Some(parent) = &class.parent {
            header.push_str(&format!(" : public {}", parent));
        }
        header.push_str(" {");
        lines.push(header);

        if let Some(vtable) = class.vtable_address {
            lines.push(format!("    // VTable: {}", self.format_address(vtable)));
        }
        lines.push(format!("    // Size: {} bytes", class.size));

        if !class.properties.is_empty() {
            lines.push("    // Properties:".to_string());
            for prop in &class.properties {
                lines.push(format!("    //   {}", prop));
            }
        }

        if !class.methods.is_empty() {
            lines.push("    // Methods:".to_string());
            for method in &class.methods {
                lines.push(format!("    //   {}", method));
            }
        }

        lines.push("};".to_string());
        lines.join("\n")
    }

    pub fn format_class_list(&self, classes: &[ClassOffset]) -> String {
        let mut lines = Vec::new();
        let mut sorted = classes.to_vec();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        for class in sorted {
            lines.push(self.format_class(&class));
            lines.push(String::new());
        }

        lines.join("\n")
    }

    pub fn format_property(&self, property: &PropertyOffset) -> String {
        let mut parts = Vec::new();
        parts.push(format!("{}::{}", property.class_name, property.name));
        parts.push(format!("type: {}", property.property_type));

        if let Some(getter) = property.getter {
            parts.push(format!("getter: {}", self.format_address(getter)));
        }
        if let Some(setter) = property.setter {
            parts.push(format!("setter: {}", self.format_address(setter)));
        }
        if let Some(offset) = property.offset {
            parts.push(format!("offset: 0x{:x}", offset));
        }

        parts.join(" | ")
    }

    pub fn format_method(&self, method: &MethodOffset) -> String {
        let mut parts = Vec::new();
        parts.push(format!("{}::{}", method.class_name, method.name));
        parts.push(self.format_address(method.address));

        if method.is_virtual {
            if let Some(idx) = method.vtable_index {
                parts.push(format!("vtable[{}]", idx));
            } else {
                parts.push("virtual".to_string());
            }
        }

        if let Some(sig) = &method.signature {
            parts.push(sig.clone());
        }

        parts.join(" | ")
    }

    pub fn format_constant(&self, constant: &ConstantOffset) -> String {
        let value_str = match &constant.value {
            ConstantValue::Integer(i) => format!("{} (0x{:x})", i, i),
            ConstantValue::Float(f) => format!("{:.6}", f),
            ConstantValue::String(s) => format!("\"{}\"", s),
            ConstantValue::Address(a) => self.format_address(*a),
            ConstantValue::Unknown => "unknown".to_string(),
        };

        format!(
            "{}: {} @ {} [{}]",
            constant.name,
            value_str,
            self.format_address(constant.address),
            constant.category
        )
    }

    pub fn format_full_output(&self, output: &OffsetOutput) -> String {
        let mut result = String::new();

        result.push_str(&format!("// Generated: {}\n", output.generated_at));
        result.push_str(&format!("// Target: {} ({} / {})\n", output.target.name, output.target.architecture, output.target.platform));
        result.push_str(&format!("// Base Address: {}\n\n", self.format_address(output.target.base_address)));

        result.push_str("// === FUNCTIONS ===\n\n");
        result.push_str(&self.format_function_list(&output.functions));

        result.push_str("\n\n// === STRUCTURES ===\n\n");
        result.push_str(&self.format_structure_list(&output.structure_offsets));

        result.push_str("\n\n// === CLASSES ===\n\n");
        result.push_str(&self.format_class_list(&output.classes));

        result
    }

    pub fn format_cpp_header(&self, output: &OffsetOutput) -> String {
        let mut header = String::new();

        header.push_str("#pragma once\n\n");
        header.push_str("#include <cstdint>\n\n");
        header.push_str("namespace Offsets {\n\n");

        header.push_str("namespace Functions {\n");
        let mut sorted_funcs: Vec<_> = output.functions.iter().collect();
        sorted_funcs.sort_by(|a, b| a.0.cmp(b.0));

        for (name, func) in sorted_funcs {
            let safe_name = name.replace(".", "_").replace("::", "_");
            header.push_str(&format!("    constexpr uintptr_t {} = 0x{:x};\n", safe_name, func.address));
        }
        header.push_str("}\n\n");

        for (struct_name, structure) in &output.structure_offsets {
            let safe_name = struct_name.replace(".", "_").replace("::", "_");
            header.push_str(&format!("namespace {} {{\n", safe_name));

            let mut fields: Vec<_> = structure.fields.iter().collect();
            fields.sort_by_key(|(_, f)| f.offset);

            for (field_name, field) in fields {
                let safe_field = field_name.replace(".", "_").replace("::", "_");
                header.push_str(&format!("    constexpr size_t {} = 0x{:x};\n", safe_field, field.offset));
            }
            header.push_str("}\n\n");
        }

        header.push_str("}\n");
        header
    }

    pub fn format_rust_consts(&self, output: &OffsetOutput) -> String {
        let mut rust_code = String::new();

        rust_code.push_str("pub mod offsets {\n\n");

        rust_code.push_str("    pub mod functions {\n");
        let mut sorted_funcs: Vec<_> = output.functions.iter().collect();
        sorted_funcs.sort_by(|a, b| a.0.cmp(b.0));

        for (name, func) in sorted_funcs {
            let safe_name = name.to_uppercase().replace(".", "_").replace("::", "_");
            rust_code.push_str(&format!("        pub const {}: usize = 0x{:x};\n", safe_name, func.address));
        }
        rust_code.push_str("    }\n\n");

        for (struct_name, structure) in &output.structure_offsets {
            let safe_name = struct_name.to_lowercase().replace(".", "_").replace("::", "_");
            rust_code.push_str(&format!("    pub mod {} {{\n", safe_name));

            let mut fields: Vec<_> = structure.fields.iter().collect();
            fields.sort_by_key(|(_, f)| f.offset);

            for (field_name, field) in fields {
                let safe_field = field_name.to_uppercase().replace(".", "_").replace("::", "_");
                rust_code.push_str(&format!("        pub const {}: usize = 0x{:x};\n", safe_field, field.offset));
            }
            rust_code.push_str("    }\n\n");
        }

        rust_code.push_str("}\n");
        rust_code
    }

    pub fn format_lua_table(&self, output: &OffsetOutput) -> String {
        let mut lua_code = String::new();

        lua_code.push_str("local Offsets = {\n");

        lua_code.push_str("    Functions = {\n");
        let mut sorted_funcs: Vec<_> = output.functions.iter().collect();
        sorted_funcs.sort_by(|a, b| a.0.cmp(b.0));

        for (name, func) in sorted_funcs {
            lua_code.push_str(&format!("        [\"{}\"] = 0x{:x},\n", name, func.address));
        }
        lua_code.push_str("    },\n\n");

        lua_code.push_str("    Structures = {\n");
        for (struct_name, structure) in &output.structure_offsets {
            lua_code.push_str(&format!("        [\"{}\"] = {{\n", struct_name));

            let mut fields: Vec<_> = structure.fields.iter().collect();
            fields.sort_by_key(|(_, f)| f.offset);

            for (field_name, field) in fields {
                lua_code.push_str(&format!("            [\"{}\"] = 0x{:x},\n", field_name, field.offset));
            }
            lua_code.push_str("        },\n");
        }
        lua_code.push_str("    },\n");

        lua_code.push_str("}\n\nreturn Offsets\n");
        lua_code
    }
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn format_hex(value: u64) -> String {
    format!("0x{:016x}", value)
}

pub fn format_hex_short(value: u64) -> String {
    format!("0x{:x}", value)
}

pub fn format_confidence(confidence: f64) -> String {
    format!("{:.1}%", confidence * 100.0)
}
