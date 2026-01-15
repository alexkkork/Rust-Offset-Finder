// Tue Jan 13 2026 - Alex

use crate::output::{OffsetOutput, FunctionOffset, StructureOffsets, ClassOffset, ConstantValue};
use crate::output::formatter::OutputFormatter;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;

pub struct OffsetExporter {
    formatter: OutputFormatter,
    include_comments: bool,
    include_types: bool,
    namespace_prefix: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    CppHeader,
    CppSource,
    RustModule,
    LuaTable,
    PythonDict,
    JavaScriptModule,
    IdaScript,
    GhidraScript,
    CheatEngine,
    FridaScript,
}

impl OffsetExporter {
    pub fn new() -> Self {
        Self {
            formatter: OutputFormatter::new(),
            include_comments: true,
            include_types: true,
            namespace_prefix: String::new(),
        }
    }

    pub fn with_comments(mut self, include: bool) -> Self {
        self.include_comments = include;
        self
    }

    pub fn with_types(mut self, include: bool) -> Self {
        self.include_types = include;
        self
    }

    pub fn with_namespace_prefix(mut self, prefix: &str) -> Self {
        self.namespace_prefix = prefix.to_string();
        self
    }

    pub fn export(&self, output: &OffsetOutput, format: ExportFormat) -> String {
        match format {
            ExportFormat::CppHeader => self.export_cpp_header(output),
            ExportFormat::CppSource => self.export_cpp_source(output),
            ExportFormat::RustModule => self.export_rust_module(output),
            ExportFormat::LuaTable => self.export_lua_table(output),
            ExportFormat::PythonDict => self.export_python_dict(output),
            ExportFormat::JavaScriptModule => self.export_javascript_module(output),
            ExportFormat::IdaScript => self.export_ida_script(output),
            ExportFormat::GhidraScript => self.export_ghidra_script(output),
            ExportFormat::CheatEngine => self.export_cheat_engine(output),
            ExportFormat::FridaScript => self.export_frida_script(output),
        }
    }

    pub fn export_to_file(&self, output: &OffsetOutput, format: ExportFormat, path: &Path) -> std::io::Result<()> {
        let content = self.export(output, format);
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(content.as_bytes())?;
        Ok(())
    }

    fn export_cpp_header(&self, output: &OffsetOutput) -> String {
        let mut code = String::new();

        code.push_str("#pragma once\n\n");

        if self.include_comments {
            code.push_str(&format!("// Generated: {}\n", output.generated_at));
            code.push_str(&format!("// Target: {}\n", output.target.name));
            code.push_str(&format!("// Architecture: {}\n", output.target.architecture));
            code.push_str(&format!("// Base Address: 0x{:x}\n\n", output.target.base_address));
        }

        code.push_str("#include <cstdint>\n");
        code.push_str("#include <cstddef>\n\n");

        let ns = if self.namespace_prefix.is_empty() { "Offsets" } else { &self.namespace_prefix };
        code.push_str(&format!("namespace {} {{\n\n", ns));

        code.push_str("constexpr uintptr_t BASE_ADDRESS = 0x{:x};\n\n");
        let base_addr_line = format!("constexpr uintptr_t BASE_ADDRESS = 0x{:x};\n\n", output.target.base_address);
        code = code.replace("constexpr uintptr_t BASE_ADDRESS = 0x{:x};\n\n", &base_addr_line);

        code.push_str("namespace Functions {\n");
        let mut sorted: Vec<_> = output.functions.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        for (name, func) in &sorted {
            let safe_name = Self::sanitize_cpp_name(name);
            if self.include_comments {
                code.push_str(&format!("    // {} - {:.1}% confidence\n", func.discovery_method, func.confidence * 100.0));
            }
            code.push_str(&format!("    constexpr uintptr_t {} = 0x{:x};\n", safe_name, func.address));
        }
        code.push_str("} // namespace Functions\n\n");

        code.push_str("namespace Structures {\n");
        for (struct_name, structure) in &output.structure_offsets {
            let safe_struct = Self::sanitize_cpp_name(struct_name);
            code.push_str(&format!("\n    namespace {} {{\n", safe_struct));
            if self.include_comments {
                code.push_str(&format!("        // Size: {} bytes, Alignment: {}\n", structure.size, structure.alignment));
            }
            code.push_str(&format!("        constexpr size_t SIZE = {};\n", structure.size));
            code.push_str(&format!("        constexpr size_t ALIGNMENT = {};\n", structure.alignment));

            let mut fields: Vec<_> = structure.fields.iter().collect();
            fields.sort_by_key(|(_, f)| f.offset);

            for (field_name, field) in fields {
                let safe_field = Self::sanitize_cpp_name(field_name);
                if self.include_comments {
                    code.push_str(&format!("        // Type: {}, Size: {}\n", field.field_type, field.size));
                }
                code.push_str(&format!("        constexpr size_t {} = 0x{:x};\n", safe_field, field.offset));
            }
            code.push_str(&format!("    }} // namespace {}\n", safe_struct));
        }
        code.push_str("} // namespace Structures\n\n");

        code.push_str("namespace Classes {\n");
        for class in &output.classes {
            let safe_class = Self::sanitize_cpp_name(&class.name);
            code.push_str(&format!("\n    namespace {} {{\n", safe_class));
            if let Some(vtable) = class.vtable_address {
                code.push_str(&format!("        constexpr uintptr_t VTABLE = 0x{:x};\n", vtable));
            }
            code.push_str(&format!("        constexpr size_t SIZE = {};\n", class.size));
            code.push_str(&format!("    }} // namespace {}\n", safe_class));
        }
        code.push_str("} // namespace Classes\n\n");

        code.push_str(&format!("}} // namespace {}\n", ns));
        code
    }

    fn export_cpp_source(&self, output: &OffsetOutput) -> String {
        let mut code = String::new();

        code.push_str("#include \"offsets.hpp\"\n\n");

        if self.include_comments {
            code.push_str(&format!("// Generated: {}\n", output.generated_at));
            code.push_str(&format!("// Target: {}\n\n", output.target.name));
        }

        let ns = if self.namespace_prefix.is_empty() { "Offsets" } else { &self.namespace_prefix };

        code.push_str(&format!("namespace {} {{\n\n", ns));

        code.push_str("bool validate_offsets() {\n");
        code.push_str("    // Runtime validation can be added here\n");
        code.push_str("    return true;\n");
        code.push_str("}\n\n");

        code.push_str("const char* get_version() {\n");
        code.push_str(&format!("    return \"{}\";\n", output.version));
        code.push_str("}\n\n");

        code.push_str(&format!("}} // namespace {}\n", ns));
        code
    }

    fn export_rust_module(&self, output: &OffsetOutput) -> String {
        let mut code = String::new();

        if self.include_comments {
            code.push_str(&format!("//! Generated: {}\n", output.generated_at));
            code.push_str(&format!("//! Target: {}\n", output.target.name));
            code.push_str(&format!("//! Architecture: {}\n\n", output.target.architecture));
        }

        code.push_str("#![allow(dead_code)]\n\n");

        code.push_str(&format!("pub const BASE_ADDRESS: usize = 0x{:x};\n\n", output.target.base_address));

        code.push_str("pub mod functions {\n");
        let mut sorted: Vec<_> = output.functions.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        for (name, func) in &sorted {
            let safe_name = Self::sanitize_rust_name(name);
            if self.include_comments {
                code.push_str(&format!("    /// {} - {:.1}% confidence\n", func.discovery_method, func.confidence * 100.0));
            }
            code.push_str(&format!("    pub const {}: usize = 0x{:x};\n", safe_name, func.address));
        }
        code.push_str("}\n\n");

        code.push_str("pub mod structures {\n");
        for (struct_name, structure) in &output.structure_offsets {
            let safe_struct = Self::sanitize_rust_name(struct_name);
            code.push_str(&format!("\n    pub mod {} {{\n", safe_struct.to_lowercase()));
            code.push_str(&format!("        pub const SIZE: usize = {};\n", structure.size));
            code.push_str(&format!("        pub const ALIGNMENT: usize = {};\n", structure.alignment));

            let mut fields: Vec<_> = structure.fields.iter().collect();
            fields.sort_by_key(|(_, f)| f.offset);

            for (field_name, field) in fields {
                let safe_field = Self::sanitize_rust_name(field_name);
                code.push_str(&format!("        pub const {}: usize = 0x{:x};\n", safe_field, field.offset));
            }
            code.push_str("    }\n");
        }
        code.push_str("}\n\n");

        code.push_str("pub mod classes {\n");
        for class in &output.classes {
            let safe_class = Self::sanitize_rust_name(&class.name);
            code.push_str(&format!("\n    pub mod {} {{\n", safe_class.to_lowercase()));
            if let Some(vtable) = class.vtable_address {
                code.push_str(&format!("        pub const VTABLE: usize = 0x{:x};\n", vtable));
            }
            code.push_str(&format!("        pub const SIZE: usize = {};\n", class.size));
            code.push_str("    }\n");
        }
        code.push_str("}\n");

        code
    }

    fn export_lua_table(&self, output: &OffsetOutput) -> String {
        let mut code = String::new();

        if self.include_comments {
            code.push_str(&format!("-- Generated: {}\n", output.generated_at));
            code.push_str(&format!("-- Target: {}\n", output.target.name));
            code.push_str(&format!("-- Architecture: {}\n\n", output.target.architecture));
        }

        code.push_str("local Offsets = {\n");
        code.push_str(&format!("    BASE_ADDRESS = 0x{:x},\n\n", output.target.base_address));

        code.push_str("    Functions = {\n");
        let mut sorted: Vec<_> = output.functions.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        for (name, func) in &sorted {
            code.push_str(&format!("        [\"{}\"] = 0x{:x},\n", name, func.address));
        }
        code.push_str("    },\n\n");

        code.push_str("    Structures = {\n");
        for (struct_name, structure) in &output.structure_offsets {
            code.push_str(&format!("        [\"{}\"] = {{\n", struct_name));
            code.push_str(&format!("            SIZE = {},\n", structure.size));

            let mut fields: Vec<_> = structure.fields.iter().collect();
            fields.sort_by_key(|(_, f)| f.offset);

            for (field_name, field) in fields {
                code.push_str(&format!("            [\"{}\"] = 0x{:x},\n", field_name, field.offset));
            }
            code.push_str("        },\n");
        }
        code.push_str("    },\n\n");

        code.push_str("    Classes = {\n");
        for class in &output.classes {
            code.push_str(&format!("        [\"{}\"] = {{\n", class.name));
            if let Some(vtable) = class.vtable_address {
                code.push_str(&format!("            VTABLE = 0x{:x},\n", vtable));
            }
            code.push_str(&format!("            SIZE = {},\n", class.size));
            code.push_str("        },\n");
        }
        code.push_str("    },\n");

        code.push_str("}\n\nreturn Offsets\n");
        code
    }

    fn export_python_dict(&self, output: &OffsetOutput) -> String {
        let mut code = String::new();

        if self.include_comments {
            code.push_str(&format!("# Generated: {}\n", output.generated_at));
            code.push_str(&format!("# Target: {}\n", output.target.name));
            code.push_str(&format!("# Architecture: {}\n\n", output.target.architecture));
        }

        code.push_str("OFFSETS = {\n");
        code.push_str(&format!("    'base_address': 0x{:x},\n\n", output.target.base_address));

        code.push_str("    'functions': {\n");
        let mut sorted: Vec<_> = output.functions.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        for (name, func) in &sorted {
            code.push_str(&format!("        '{}': 0x{:x},\n", name, func.address));
        }
        code.push_str("    },\n\n");

        code.push_str("    'structures': {\n");
        for (struct_name, structure) in &output.structure_offsets {
            code.push_str(&format!("        '{}': {{\n", struct_name));
            code.push_str(&format!("            'size': {},\n", structure.size));
            code.push_str("            'fields': {\n");

            let mut fields: Vec<_> = structure.fields.iter().collect();
            fields.sort_by_key(|(_, f)| f.offset);

            for (field_name, field) in fields {
                code.push_str(&format!("                '{}': 0x{:x},\n", field_name, field.offset));
            }
            code.push_str("            },\n");
            code.push_str("        },\n");
        }
        code.push_str("    },\n\n");

        code.push_str("    'classes': {\n");
        for class in &output.classes {
            code.push_str(&format!("        '{}': {{\n", class.name));
            if let Some(vtable) = class.vtable_address {
                code.push_str(&format!("            'vtable': 0x{:x},\n", vtable));
            }
            code.push_str(&format!("            'size': {},\n", class.size));
            code.push_str("        },\n");
        }
        code.push_str("    },\n");

        code.push_str("}\n");
        code
    }

    fn export_javascript_module(&self, output: &OffsetOutput) -> String {
        let mut code = String::new();

        if self.include_comments {
            code.push_str(&format!("// Generated: {}\n", output.generated_at));
            code.push_str(&format!("// Target: {}\n", output.target.name));
            code.push_str(&format!("// Architecture: {}\n\n", output.target.architecture));
        }

        code.push_str("export const OFFSETS = {\n");
        code.push_str(&format!("    baseAddress: 0x{:x}n,\n\n", output.target.base_address));

        code.push_str("    functions: {\n");
        let mut sorted: Vec<_> = output.functions.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        for (name, func) in &sorted {
            let safe_name = Self::sanitize_js_name(name);
            code.push_str(&format!("        {}: 0x{:x}n,\n", safe_name, func.address));
        }
        code.push_str("    },\n\n");

        code.push_str("    structures: {\n");
        for (struct_name, structure) in &output.structure_offsets {
            let safe_struct = Self::sanitize_js_name(struct_name);
            code.push_str(&format!("        {}: {{\n", safe_struct));
            code.push_str(&format!("            size: {},\n", structure.size));

            let mut fields: Vec<_> = structure.fields.iter().collect();
            fields.sort_by_key(|(_, f)| f.offset);

            for (field_name, field) in fields {
                let safe_field = Self::sanitize_js_name(field_name);
                code.push_str(&format!("            {}: 0x{:x},\n", safe_field, field.offset));
            }
            code.push_str("        },\n");
        }
        code.push_str("    },\n");

        code.push_str("};\n\nexport default OFFSETS;\n");
        code
    }

    fn export_ida_script(&self, output: &OffsetOutput) -> String {
        let mut code = String::new();

        code.push_str("import idaapi\nimport idc\n\n");

        if self.include_comments {
            code.push_str(&format!("# Generated: {}\n", output.generated_at));
            code.push_str(&format!("# Target: {}\n\n", output.target.name));
        }

        code.push_str("def apply_offsets():\n");
        code.push_str(&format!("    base = 0x{:x}\n\n", output.target.base_address));

        code.push_str("    # Apply function names\n");
        for (name, func) in &output.functions {
            code.push_str(&format!("    idc.set_name(0x{:x}, \"{}\", idc.SN_NOWARN)\n", func.address, name));
        }

        code.push_str("\n    print(\"Offsets applied successfully!\")\n\n");
        code.push_str("if __name__ == \"__main__\":\n");
        code.push_str("    apply_offsets()\n");

        code
    }

    fn export_ghidra_script(&self, output: &OffsetOutput) -> String {
        let mut code = String::new();

        code.push_str("# Ghidra script to apply offsets\n");
        code.push_str("# @category: Analysis\n\n");

        if self.include_comments {
            code.push_str(&format!("# Generated: {}\n", output.generated_at));
            code.push_str(&format!("# Target: {}\n\n", output.target.name));
        }

        code.push_str("from ghidra.program.model.symbol import SourceType\n\n");

        code.push_str("def run():\n");
        code.push_str("    program = getCurrentProgram()\n");
        code.push_str("    symbolTable = program.getSymbolTable()\n");
        code.push_str("    addressFactory = program.getAddressFactory()\n\n");

        for (name, func) in &output.functions {
            code.push_str(&format!(
                "    addr = addressFactory.getAddress(\"0x{:x}\")\n    symbolTable.createLabel(addr, \"{}\", SourceType.USER_DEFINED)\n",
                func.address, name
            ));
        }

        code.push_str("\n    print(\"Offsets applied!\")\n\n");
        code.push_str("run()\n");

        code
    }

    fn export_cheat_engine(&self, output: &OffsetOutput) -> String {
        let mut code = String::new();

        code.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        code.push_str("<CheatTable>\n");
        code.push_str("  <CheatEntries>\n");

        for (name, func) in &output.functions {
            code.push_str("    <CheatEntry>\n");
            code.push_str(&format!("      <Description>{}</Description>\n", name));
            code.push_str(&format!("      <Address>{:x}</Address>\n", func.address));
            code.push_str("      <VariableType>Auto Assembler Script</VariableType>\n");
            code.push_str("    </CheatEntry>\n");
        }

        code.push_str("  </CheatEntries>\n");
        code.push_str("</CheatTable>\n");

        code
    }

    fn export_frida_script(&self, output: &OffsetOutput) -> String {
        let mut code = String::new();

        if self.include_comments {
            code.push_str(&format!("// Generated: {}\n", output.generated_at));
            code.push_str(&format!("// Target: {}\n\n", output.target.name));
        }

        code.push_str("const Offsets = {\n");
        code.push_str(&format!("    baseAddress: ptr('0x{:x}'),\n\n", output.target.base_address));

        code.push_str("    functions: {\n");
        for (name, func) in &output.functions {
            let safe_name = Self::sanitize_js_name(name);
            code.push_str(&format!("        {}: ptr('0x{:x}'),\n", safe_name, func.address));
        }
        code.push_str("    },\n");
        code.push_str("};\n\n");

        code.push_str("function hookFunction(name) {\n");
        code.push_str("    const addr = Offsets.functions[name];\n");
        code.push_str("    if (!addr) return;\n\n");
        code.push_str("    Interceptor.attach(addr, {\n");
        code.push_str("        onEnter: function(args) {\n");
        code.push_str("            console.log(`${name} called`);\n");
        code.push_str("        },\n");
        code.push_str("        onLeave: function(retval) {\n");
        code.push_str("            console.log(`${name} returned: ${retval}`);\n");
        code.push_str("        }\n");
        code.push_str("    });\n");
        code.push_str("}\n\n");

        code.push_str("module.exports = { Offsets, hookFunction };\n");

        code
    }

    fn sanitize_cpp_name(name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect()
    }

    fn sanitize_rust_name(name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() { c.to_ascii_uppercase() } else { '_' })
            .collect()
    }

    fn sanitize_js_name(name: &str) -> String {
        let result: String = name.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();

        if result.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
            format!("_{}", result)
        } else {
            result
        }
    }
}

impl Default for OffsetExporter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn export_to_cpp(output: &OffsetOutput) -> String {
    OffsetExporter::new().export(output, ExportFormat::CppHeader)
}

pub fn export_to_rust(output: &OffsetOutput) -> String {
    OffsetExporter::new().export(output, ExportFormat::RustModule)
}

pub fn export_to_lua(output: &OffsetOutput) -> String {
    OffsetExporter::new().export(output, ExportFormat::LuaTable)
}

pub fn export_to_python(output: &OffsetOutput) -> String {
    OffsetExporter::new().export(output, ExportFormat::PythonDict)
}

pub fn export_to_frida(output: &OffsetOutput) -> String {
    OffsetExporter::new().export(output, ExportFormat::FridaScript)
}
