// Tue Jan 15 2026 - Alex

use crate::symbol::{Symbol, SymbolType};
use std::io::Write;

/// Export format for symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// IDA Pro script format
    Ida,
    /// Ghidra script format
    Ghidra,
    /// Binary Ninja script
    BinaryNinja,
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// C header file
    CHeader,
    /// LLDB commands
    Lldb,
    /// GDB commands
    Gdb,
    /// Symbol map format
    SymbolMap,
}

/// Symbol exporter
pub struct SymbolExporter {
    symbols: Vec<ExportableSymbol>,
    base_address: Option<u64>,
    include_types: bool,
    include_sizes: bool,
}

impl SymbolExporter {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            base_address: None,
            include_types: true,
            include_sizes: true,
        }
    }

    pub fn with_base_address(mut self, base: u64) -> Self {
        self.base_address = Some(base);
        self
    }

    pub fn without_types(mut self) -> Self {
        self.include_types = false;
        self
    }

    pub fn without_sizes(mut self) -> Self {
        self.include_sizes = false;
        self
    }

    pub fn add_symbol(&mut self, symbol: ExportableSymbol) {
        self.symbols.push(symbol);
    }

    pub fn add_symbols(&mut self, symbols: impl Iterator<Item = ExportableSymbol>) {
        self.symbols.extend(symbols);
    }

    pub fn from_symbols(symbols: &[Symbol]) -> Self {
        let mut exporter = Self::new();
        for sym in symbols {
            exporter.add_symbol(ExportableSymbol::from_symbol(sym));
        }
        exporter
    }

    /// Export to specified format
    pub fn export(&self, format: ExportFormat) -> String {
        match format {
            ExportFormat::Ida => self.to_ida(),
            ExportFormat::Ghidra => self.to_ghidra(),
            ExportFormat::BinaryNinja => self.to_binary_ninja(),
            ExportFormat::Json => self.to_json(),
            ExportFormat::Csv => self.to_csv(),
            ExportFormat::CHeader => self.to_c_header(),
            ExportFormat::Lldb => self.to_lldb(),
            ExportFormat::Gdb => self.to_gdb(),
            ExportFormat::SymbolMap => self.to_symbol_map(),
        }
    }

    /// Export to IDA Pro Python script
    fn to_ida(&self) -> String {
        let mut script = String::new();
        
        script.push_str("# IDA Pro symbol import script\n");
        script.push_str("# Auto-generated\n\n");
        script.push_str("import idaapi\n");
        script.push_str("import idc\n\n");

        if let Some(base) = self.base_address {
            script.push_str(&format!("base_addr = 0x{:X}\n\n", base));
        }

        script.push_str("def import_symbols():\n");

        for sym in &self.symbols {
            let addr = if self.base_address.is_some() {
                format!("base_addr + 0x{:X}", sym.address)
            } else {
                format!("0x{:X}", sym.address)
            };

            // Set name
            script.push_str(&format!(
                "    idc.set_name({}, \"{}\", idc.SN_NOWARN)\n",
                addr, escape_string(&sym.name)
            ));

            // Set type if function
            if sym.symbol_type == ExportSymbolType::Function {
                script.push_str(&format!(
                    "    idc.create_insn({})\n",
                    addr
                ));

                if let Some(size) = sym.size {
                    script.push_str(&format!(
                        "    idc.add_func({}, {} + 0x{:X})\n",
                        addr, addr, size
                    ));
                } else {
                    script.push_str(&format!(
                        "    idc.add_func({})\n",
                        addr
                    ));
                }
            }
        }

        script.push_str("\nimport_symbols()\n");
        script.push_str("print(\"Imported {} symbols\")\n");

        script
    }

    /// Export to Ghidra Python script
    fn to_ghidra(&self) -> String {
        let mut script = String::new();
        
        script.push_str("# Ghidra symbol import script\n");
        script.push_str("# @category: Symbols\n\n");
        script.push_str("from ghidra.program.model.symbol import SourceType\n\n");

        script.push_str("def run():\n");
        script.push_str("    sm = currentProgram.getSymbolTable()\n");
        script.push_str("    fm = currentProgram.getFunctionManager()\n\n");

        for sym in &self.symbols {
            let addr = format!("toAddr(0x{:X})", sym.address);

            if sym.symbol_type == ExportSymbolType::Function {
                script.push_str(&format!(
                    "    createFunction({}, \"{}\")\n",
                    addr, escape_string(&sym.name)
                ));
            } else {
                script.push_str(&format!(
                    "    createLabel({}, \"{}\", True)\n",
                    addr, escape_string(&sym.name)
                ));
            }
        }

        script.push_str("\nrun()\n");

        script
    }

    /// Export to Binary Ninja Python script
    fn to_binary_ninja(&self) -> String {
        let mut script = String::new();
        
        script.push_str("# Binary Ninja symbol import script\n\n");

        script.push_str("def import_symbols(bv):\n");

        for sym in &self.symbols {
            match sym.symbol_type {
                ExportSymbolType::Function => {
                    script.push_str(&format!(
                        "    bv.add_function(0x{:X})\n",
                        sym.address
                    ));
                    script.push_str(&format!(
                        "    func = bv.get_function_at(0x{:X})\n",
                        sym.address
                    ));
                    script.push_str(&format!(
                        "    if func: func.name = \"{}\"\n",
                        escape_string(&sym.name)
                    ));
                }
                _ => {
                    script.push_str(&format!(
                        "    bv.define_user_symbol(Symbol(SymbolType.DataSymbol, 0x{:X}, \"{}\"))\n",
                        sym.address, escape_string(&sym.name)
                    ));
                }
            }
        }

        script.push_str("\nimport_symbols(bv)\n");

        script
    }

    /// Export to JSON format
    fn to_json(&self) -> String {
        let mut json = String::new();
        
        json.push_str("{\n");
        if let Some(base) = self.base_address {
            json.push_str(&format!("  \"base_address\": \"0x{:X}\",\n", base));
        }
        json.push_str("  \"symbols\": [\n");

        for (i, sym) in self.symbols.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"name\": \"{}\",\n", escape_json(&sym.name)));
            json.push_str(&format!("      \"address\": \"0x{:X}\",\n", sym.address));
            json.push_str(&format!("      \"type\": \"{:?}\"", sym.symbol_type));
            
            if self.include_sizes {
                if let Some(size) = sym.size {
                    json.push_str(&format!(",\n      \"size\": {}", size));
                }
            }
            
            json.push_str("\n    }");
            if i < self.symbols.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }

        json.push_str("  ]\n");
        json.push_str("}\n");

        json
    }

    /// Export to CSV format
    fn to_csv(&self) -> String {
        let mut csv = String::new();
        
        csv.push_str("address,name,type");
        if self.include_sizes {
            csv.push_str(",size");
        }
        csv.push('\n');

        for sym in &self.symbols {
            csv.push_str(&format!(
                "0x{:X},{},{:?}",
                sym.address,
                escape_csv(&sym.name),
                sym.symbol_type
            ));
            
            if self.include_sizes {
                if let Some(size) = sym.size {
                    csv.push_str(&format!(",{}", size));
                } else {
                    csv.push(',');
                }
            }
            
            csv.push('\n');
        }

        csv
    }

    /// Export to C header file
    fn to_c_header(&self) -> String {
        let mut header = String::new();
        
        header.push_str("// Auto-generated symbol definitions\n");
        header.push_str("#ifndef SYMBOLS_H\n");
        header.push_str("#define SYMBOLS_H\n\n");
        header.push_str("#include <stdint.h>\n\n");

        // Generate address defines
        header.push_str("// Symbol addresses\n");
        for sym in &self.symbols {
            let c_name = to_c_identifier(&sym.name);
            header.push_str(&format!(
                "#define {}_ADDR 0x{:X}ULL\n",
                c_name.to_uppercase(),
                sym.address
            ));
        }

        header.push_str("\n// Function pointer types\n");
        for sym in &self.symbols {
            if sym.symbol_type == ExportSymbolType::Function {
                let c_name = to_c_identifier(&sym.name);
                header.push_str(&format!(
                    "typedef void (*{}_t)(void);\n",
                    c_name
                ));
            }
        }

        header.push_str("\n// Function pointers\n");
        for sym in &self.symbols {
            if sym.symbol_type == ExportSymbolType::Function {
                let c_name = to_c_identifier(&sym.name);
                header.push_str(&format!(
                    "#define {} (({}_t){}_ADDR)\n",
                    c_name, c_name, c_name.to_uppercase()
                ));
            }
        }

        header.push_str("\n#endif // SYMBOLS_H\n");

        header
    }

    /// Export to LLDB commands
    fn to_lldb(&self) -> String {
        let mut commands = String::new();
        
        commands.push_str("# LLDB symbol import commands\n\n");

        for sym in &self.symbols {
            commands.push_str(&format!(
                "image lookup -a 0x{:X} # {}\n",
                sym.address, sym.name
            ));
            
            // Add breakpoint for functions
            if sym.symbol_type == ExportSymbolType::Function {
                commands.push_str(&format!(
                    "# br set -a 0x{:X} -N {}\n",
                    sym.address, sym.name
                ));
            }
        }

        commands
    }

    /// Export to GDB commands
    fn to_gdb(&self) -> String {
        let mut commands = String::new();
        
        commands.push_str("# GDB symbol import commands\n\n");

        for sym in &self.symbols {
            // Add symbol
            commands.push_str(&format!(
                "add-symbol-file-from-memory 0x{:X} # {}\n",
                sym.address, sym.name
            ));
            
            // Set convenience variable
            let gdb_name = to_c_identifier(&sym.name);
            commands.push_str(&format!(
                "set ${} = (void*)0x{:X}\n",
                gdb_name, sym.address
            ));
        }

        commands
    }

    /// Export to symbol map format
    fn to_symbol_map(&self) -> String {
        let mut map = String::new();
        
        map.push_str("; Symbol Map\n");
        map.push_str("; Format: address type name [size]\n\n");

        // Sort by address
        let mut sorted: Vec<_> = self.symbols.iter().collect();
        sorted.sort_by_key(|s| s.address);

        for sym in sorted {
            let type_char = match sym.symbol_type {
                ExportSymbolType::Function => 'F',
                ExportSymbolType::Data => 'D',
                ExportSymbolType::Object => 'O',
                ExportSymbolType::Unknown => '?',
            };

            map.push_str(&format!(
                "{:016X} {} {}",
                sym.address, type_char, sym.name
            ));
            
            if let Some(size) = sym.size {
                map.push_str(&format!(" {}", size));
            }
            
            map.push('\n');
        }

        map
    }

    /// Write export to file
    pub fn export_to_file(&self, format: ExportFormat, path: &str) -> std::io::Result<()> {
        let content = self.export(format);
        let mut file = std::fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }
}

impl Default for SymbolExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Symbol prepared for export
#[derive(Debug, Clone)]
pub struct ExportableSymbol {
    pub name: String,
    pub address: u64,
    pub symbol_type: ExportSymbolType,
    pub size: Option<u64>,
    pub demangle_name: Option<String>,
    pub module: Option<String>,
}

impl ExportableSymbol {
    pub fn new(name: &str, address: u64) -> Self {
        Self {
            name: name.to_string(),
            address,
            symbol_type: ExportSymbolType::Unknown,
            size: None,
            demangle_name: None,
            module: None,
        }
    }

    pub fn function(name: &str, address: u64) -> Self {
        Self {
            name: name.to_string(),
            address,
            symbol_type: ExportSymbolType::Function,
            size: None,
            demangle_name: None,
            module: None,
        }
    }

    pub fn data(name: &str, address: u64) -> Self {
        Self {
            name: name.to_string(),
            address,
            symbol_type: ExportSymbolType::Data,
            size: None,
            demangle_name: None,
            module: None,
        }
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_type(mut self, typ: ExportSymbolType) -> Self {
        self.symbol_type = typ;
        self
    }

    pub fn with_demangled(mut self, name: &str) -> Self {
        self.demangle_name = Some(name.to_string());
        self
    }

    pub fn from_symbol(sym: &Symbol) -> Self {
        Self {
            name: sym.name.clone(),
            address: sym.address.as_u64(),
            symbol_type: match sym.symbol_type {
                SymbolType::Function => ExportSymbolType::Function,
                SymbolType::Data | SymbolType::BSS => ExportSymbolType::Data,
                SymbolType::External | SymbolType::Undefined | 
                SymbolType::Section | SymbolType::Unknown => ExportSymbolType::Unknown,
            },
            size: sym.size,
            demangle_name: sym.demangled_name.clone(),
            module: None,
        }
    }
}

/// Symbol types for export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportSymbolType {
    Function,
    Data,
    Object,
    Unknown,
}

// Helper functions

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
}

fn escape_json(s: &str) -> String {
    escape_string(s)
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn to_c_identifier(s: &str) -> String {
    let mut result = String::new();
    let mut prev_underscore = false;

    for c in s.chars() {
        if c.is_alphanumeric() {
            result.push(c);
            prev_underscore = false;
        } else if !prev_underscore {
            result.push('_');
            prev_underscore = true;
        }
    }

    // Ensure doesn't start with digit
    if result.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        result.insert(0, '_');
    }

    result
}

/// Import symbols from various formats
pub struct SymbolImporter;

impl SymbolImporter {
    /// Import from symbol map format
    pub fn from_symbol_map(content: &str) -> Vec<ExportableSymbol> {
        let mut symbols = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(addr) = u64::from_str_radix(parts[0].trim_start_matches("0x"), 16) {
                    let symbol_type = match parts[1] {
                        "F" => ExportSymbolType::Function,
                        "D" => ExportSymbolType::Data,
                        "O" => ExportSymbolType::Object,
                        _ => ExportSymbolType::Unknown,
                    };

                    let mut sym = ExportableSymbol::new(parts[2], addr);
                    sym.symbol_type = symbol_type;

                    if parts.len() >= 4 {
                        if let Ok(size) = parts[3].parse() {
                            sym.size = Some(size);
                        }
                    }

                    symbols.push(sym);
                }
            }
        }

        symbols
    }

    /// Import from CSV format
    pub fn from_csv(content: &str) -> Vec<ExportableSymbol> {
        let mut symbols = Vec::new();
        let mut lines = content.lines();
        
        // Skip header
        lines.next();

        for line in lines {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                let addr_str = parts[0].trim().trim_start_matches("0x");
                if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                    let name = parts[1].trim().trim_matches('"');
                    let mut sym = ExportableSymbol::new(name, addr);

                    let type_str = parts[2].trim();
                    sym.symbol_type = match type_str {
                        "Function" => ExportSymbolType::Function,
                        "Data" => ExportSymbolType::Data,
                        "Object" => ExportSymbolType::Object,
                        _ => ExportSymbolType::Unknown,
                    };

                    if parts.len() >= 4 {
                        if let Ok(size) = parts[3].trim().parse() {
                            sym.size = Some(size);
                        }
                    }

                    symbols.push(sym);
                }
            }
        }

        symbols
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_exporter_json() {
        let mut exporter = SymbolExporter::new();
        exporter.add_symbol(ExportableSymbol::function("test_func", 0x1000).with_size(100));
        
        let json = exporter.export(ExportFormat::Json);
        assert!(json.contains("test_func"));
        assert!(json.contains("0x1000"));
    }

    #[test]
    fn test_to_c_identifier() {
        assert_eq!(to_c_identifier("my_function"), "my_function");
        assert_eq!(to_c_identifier("my-function"), "my_function");
        assert_eq!(to_c_identifier("123start"), "_123start");
    }

    #[test]
    fn test_export_csv() {
        let mut exporter = SymbolExporter::new();
        exporter.add_symbol(ExportableSymbol::function("func1", 0x1000));
        exporter.add_symbol(ExportableSymbol::data("data1", 0x2000));
        
        let csv = exporter.export(ExportFormat::Csv);
        assert!(csv.contains("address,name,type"));
        assert!(csv.contains("func1"));
        assert!(csv.contains("data1"));
    }
}
