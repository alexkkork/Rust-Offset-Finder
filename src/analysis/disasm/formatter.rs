// Wed Jan 15 2026 - Alex

use crate::memory::Address;
use crate::analysis::disasm::{DecodedInstruction, InstructionCategory};
use std::collections::HashMap;

pub struct InstructionFormatter {
    config: FormatterConfig,
    symbol_map: HashMap<u64, String>,
}

impl InstructionFormatter {
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
            symbol_map: HashMap::new(),
        }
    }

    pub fn with_config(config: FormatterConfig) -> Self {
        Self {
            config,
            symbol_map: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, addr: u64, name: String) {
        self.symbol_map.insert(addr, name);
    }

    pub fn load_symbols(&mut self, symbols: HashMap<u64, String>) {
        self.symbol_map.extend(symbols);
    }

    pub fn format(&self, instr: &DecodedInstruction) -> String {
        let mut parts = Vec::new();

        if self.config.show_address {
            parts.push(format!("{:016X}", instr.address.as_u64()));
        }

        if self.config.show_bytes {
            let bytes_str: String = instr.bytes.iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            parts.push(format!("{:12}", bytes_str));
        }

        let mnemonic = if self.config.uppercase_mnemonics {
            instr.mnemonic.to_uppercase()
        } else {
            instr.mnemonic.to_lowercase()
        };

        let mut line = format!("{:<8}", mnemonic);

        let operands = if self.config.resolve_symbols {
            self.resolve_operands(instr)
        } else {
            instr.operand_str.clone()
        };

        line.push_str(&operands);
        parts.push(line);

        if self.config.show_category {
            parts.push(format!("[{}]", instr.category.name()));
        }

        parts.join("  ")
    }

    fn resolve_operands(&self, instr: &DecodedInstruction) -> String {
        let mut result = instr.operand_str.clone();

        if let Some(target) = instr.get_branch_target() {
            if let Some(symbol) = self.symbol_map.get(&target.as_u64()) {
                result = result.replace(
                    &format!("0x{:X}", target.as_u64()),
                    &format!("<{}>", symbol)
                );
            }
        }

        result
    }

    pub fn format_block(&self, instructions: &[DecodedInstruction]) -> String {
        let mut lines = Vec::new();

        for instr in instructions {
            lines.push(self.format(instr));
        }

        lines.join("\n")
    }

    pub fn format_function(&self, name: &str, instructions: &[DecodedInstruction]) -> String {
        let mut output = String::new();

        output.push_str(&format!("; Function: {}\n", name));
        if let Some(first) = instructions.first() {
            output.push_str(&format!("; Address: 0x{:X}\n", first.address.as_u64()));
        }
        output.push_str(&format!("; Size: {} instructions\n", instructions.len()));
        output.push_str("\n");

        output.push_str(&self.format_block(instructions));
        output.push_str("\n");

        output
    }

    pub fn format_with_annotations(&self, instr: &DecodedInstruction, annotations: &[String]) -> String {
        let mut line = self.format(instr);

        if !annotations.is_empty() {
            line.push_str(" ; ");
            line.push_str(&annotations.join(", "));
        }

        line
    }
}

impl Default for InstructionFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FormatterConfig {
    pub show_address: bool,
    pub show_bytes: bool,
    pub show_category: bool,
    pub uppercase_mnemonics: bool,
    pub resolve_symbols: bool,
    pub indent_width: usize,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            show_address: true,
            show_bytes: true,
            show_category: false,
            uppercase_mnemonics: true,
            resolve_symbols: true,
            indent_width: 4,
        }
    }
}

pub struct HexDumper {
    bytes_per_line: usize,
    show_ascii: bool,
}

impl HexDumper {
    pub fn new() -> Self {
        Self {
            bytes_per_line: 16,
            show_ascii: true,
        }
    }

    pub fn dump(&self, addr: Address, data: &[u8]) -> String {
        let mut output = String::new();
        let mut offset = 0;

        while offset < data.len() {
            let line_addr = addr.as_u64() + offset as u64;
            output.push_str(&format!("{:016X}  ", line_addr));

            let line_end = (offset + self.bytes_per_line).min(data.len());
            let line_data = &data[offset..line_end];

            for (i, byte) in line_data.iter().enumerate() {
                output.push_str(&format!("{:02X} ", byte));
                if i == 7 {
                    output.push(' ');
                }
            }

            for _ in line_data.len()..self.bytes_per_line {
                output.push_str("   ");
            }

            if self.show_ascii {
                output.push_str(" |");
                for byte in line_data {
                    let c = if *byte >= 0x20 && *byte < 0x7F {
                        *byte as char
                    } else {
                        '.'
                    };
                    output.push(c);
                }
                output.push('|');
            }

            output.push('\n');
            offset += self.bytes_per_line;
        }

        output
    }
}

impl Default for HexDumper {
    fn default() -> Self {
        Self::new()
    }
}
