// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::disassembler::{Disassembler, DisassembledInstruction};
use crate::analysis::function::AnalyzedFunction;
use std::sync::Arc;
use std::collections::HashMap;

pub struct SignatureAnalyzer {
    reader: Arc<dyn MemoryReader>,
    disassembler: Arc<Disassembler>,
}

impl SignatureAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>, disassembler: Arc<Disassembler>) -> Self {
        Self { reader, disassembler }
    }

    pub fn generate_signature(&self, addr: Address, max_bytes: usize) -> Result<FunctionSignature, MemoryError> {
        let instructions = self.disassembler.disassemble_function(addr, max_bytes)?;

        if instructions.is_empty() {
            return Ok(FunctionSignature {
                pattern: Vec::new(),
                mask: Vec::new(),
                name: None,
            });
        }

        let mut pattern = Vec::new();
        let mut mask = Vec::new();

        for instr in instructions.iter().take(16) {
            let bytes = instr.bytes.clone();

            for (idx, &byte) in bytes.iter().enumerate() {
                if self.is_stable_byte(&instr.mnemonic, idx, &instr.op_str) {
                    pattern.push(byte);
                    mask.push(0xFF);
                } else {
                    pattern.push(0x00);
                    mask.push(0x00);
                }
            }
        }

        Ok(FunctionSignature {
            pattern,
            mask,
            name: None,
        })
    }

    pub fn generate_unique_signature(&self, addr: Address, all_functions: &[Address]) -> Result<FunctionSignature, MemoryError> {
        let mut sig = self.generate_signature(addr, 64)?;

        let mut is_unique = false;
        let mut sig_len = 8;

        while !is_unique && sig_len <= sig.pattern.len() {
            let test_pattern = &sig.pattern[..sig_len];
            let test_mask = &sig.mask[..sig_len];

            let mut matches = 0;
            for &func_addr in all_functions {
                if func_addr == addr {
                    continue;
                }

                if let Ok(other_sig) = self.generate_signature(func_addr, sig_len) {
                    if self.patterns_match(test_pattern, test_mask, &other_sig.pattern, &other_sig.mask) {
                        matches += 1;
                    }
                }
            }

            if matches == 0 {
                is_unique = true;
                sig.pattern.truncate(sig_len);
                sig.mask.truncate(sig_len);
            } else {
                sig_len += 4;
            }
        }

        Ok(sig)
    }

    pub fn match_signature(&self, sig: &FunctionSignature, addr: Address) -> Result<bool, MemoryError> {
        if sig.pattern.is_empty() {
            return Ok(false);
        }

        let data = self.reader.read_bytes(addr, sig.pattern.len())?;

        Ok(self.patterns_match(&sig.pattern, &sig.mask, &data, &vec![0xFF; data.len()]))
    }

    pub fn find_signature(&self, sig: &FunctionSignature) -> Result<Vec<Address>, MemoryError> {
        let mut results = Vec::new();
        let regions = self.reader.get_regions()?;

        for region in &regions {
            if !region.protection.is_executable() {
                continue;
            }

            let data = self.reader.read_bytes(region.range.start, region.range.size() as usize)?;

            for offset in 0..data.len().saturating_sub(sig.pattern.len() - 1) {
                let window = &data[offset..offset + sig.pattern.len()];

                if self.patterns_match(&sig.pattern, &sig.mask, window, &vec![0xFF; window.len()]) {
                    results.push(Address::new(region.range.start.as_u64() + offset as u64));
                }
            }
        }

        Ok(results)
    }

    pub fn infer_argument_count(&self, function: &AnalyzedFunction) -> ArgumentInfo {
        let mut arg_registers_used = Vec::new();

        for (reg, uses) in &function.register_uses {
            if reg.starts_with("X0") || reg.starts_with("W0") {
                arg_registers_used.push(0);
            } else if reg.starts_with("X1") || reg.starts_with("W1") {
                arg_registers_used.push(1);
            } else if reg.starts_with("X2") || reg.starts_with("W2") {
                arg_registers_used.push(2);
            } else if reg.starts_with("X3") || reg.starts_with("W3") {
                arg_registers_used.push(3);
            } else if reg.starts_with("X4") || reg.starts_with("W4") {
                arg_registers_used.push(4);
            } else if reg.starts_with("X5") || reg.starts_with("W5") {
                arg_registers_used.push(5);
            } else if reg.starts_with("X6") || reg.starts_with("W6") {
                arg_registers_used.push(6);
            } else if reg.starts_with("X7") || reg.starts_with("W7") {
                arg_registers_used.push(7);
            }
        }

        arg_registers_used.sort();
        arg_registers_used.dedup();

        let count = if arg_registers_used.is_empty() {
            0
        } else {
            *arg_registers_used.last().unwrap() + 1
        };

        let mut stack_args = 0;
        for access in &function.stack_accesses {
            if access.offset >= 0 && access.read_count > 0 && access.write_count == 0 {
                stack_args += 1;
            }
        }

        ArgumentInfo {
            register_count: count,
            stack_count: stack_args,
            total: count + stack_args,
            confidence: if arg_registers_used.len() == count { 0.8 } else { 0.5 },
        }
    }

    pub fn infer_return_type(&self, function: &AnalyzedFunction) -> ReturnTypeInfo {
        let uses_x0_for_return = function.register_definitions.contains_key("X0") ||
                                 function.register_definitions.contains_key("W0");

        let uses_d0_for_return = function.register_definitions.contains_key("D0") ||
                                 function.register_definitions.contains_key("S0");

        if uses_d0_for_return {
            ReturnTypeInfo {
                inferred_type: InferredType::FloatingPoint,
                size_bytes: 8,
                confidence: 0.6,
            }
        } else if uses_x0_for_return {
            ReturnTypeInfo {
                inferred_type: InferredType::Integer,
                size_bytes: 8,
                confidence: 0.7,
            }
        } else {
            ReturnTypeInfo {
                inferred_type: InferredType::Void,
                size_bytes: 0,
                confidence: 0.5,
            }
        }
    }

    fn is_stable_byte(&self, mnemonic: &str, byte_idx: usize, op_str: &str) -> bool {
        match mnemonic {
            "B" | "BL" | "CBZ" | "CBNZ" | "TBZ" | "TBNZ" => {
                byte_idx == 0
            }
            "ADRP" | "ADR" => {
                byte_idx == 0 || byte_idx == 3
            }
            "LDR" | "STR" | "LDRSW" => {
                if op_str.contains('[') {
                    byte_idx == 0 || byte_idx == 3
                } else {
                    byte_idx < 2
                }
            }
            "MOV" | "MOVZ" | "MOVK" | "MOVN" => {
                byte_idx < 2
            }
            _ => {
                true
            }
        }
    }

    fn patterns_match(&self, pattern1: &[u8], mask1: &[u8], pattern2: &[u8], mask2: &[u8]) -> bool {
        if pattern1.len() != pattern2.len() {
            return false;
        }

        for i in 0..pattern1.len() {
            let effective_mask = mask1.get(i).copied().unwrap_or(0xFF) & mask2.get(i).copied().unwrap_or(0xFF);
            if (pattern1[i] & effective_mask) != (pattern2[i] & effective_mask) {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub pattern: Vec<u8>,
    pub mask: Vec<u8>,
    pub name: Option<String>,
}

impl FunctionSignature {
    pub fn new() -> Self {
        Self {
            pattern: Vec::new(),
            mask: Vec::new(),
            name: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn from_hex_string(pattern_str: &str) -> Result<Self, String> {
        let mut pattern = Vec::new();
        let mut mask = Vec::new();

        for part in pattern_str.split_whitespace() {
            if part == "??" || part == "?" {
                pattern.push(0x00);
                mask.push(0x00);
            } else {
                let byte = u8::from_str_radix(part, 16)
                    .map_err(|e| format!("Invalid hex byte '{}': {}", part, e))?;
                pattern.push(byte);
                mask.push(0xFF);
            }
        }

        Ok(Self {
            pattern,
            mask,
            name: None,
        })
    }

    pub fn to_hex_string(&self) -> String {
        let mut result = String::new();

        for i in 0..self.pattern.len() {
            if i > 0 {
                result.push(' ');
            }

            if self.mask.get(i).copied().unwrap_or(0xFF) == 0x00 {
                result.push_str("??");
            } else {
                result.push_str(&format!("{:02X}", self.pattern[i]));
            }
        }

        result
    }

    pub fn len(&self) -> usize {
        self.pattern.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pattern.is_empty()
    }

    pub fn specificity(&self) -> f64 {
        if self.mask.is_empty() {
            return 0.0;
        }

        let fixed_bytes = self.mask.iter().filter(|&&m| m == 0xFF).count();
        fixed_bytes as f64 / self.mask.len() as f64
    }
}

impl Default for FunctionSignature {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ArgumentInfo {
    pub register_count: usize,
    pub stack_count: usize,
    pub total: usize,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ReturnTypeInfo {
    pub inferred_type: InferredType,
    pub size_bytes: usize,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferredType {
    Void,
    Integer,
    FloatingPoint,
    Pointer,
    Struct,
    Unknown,
}

pub struct SignatureDatabase {
    signatures: HashMap<String, FunctionSignature>,
}

impl SignatureDatabase {
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, sig: FunctionSignature) {
        self.signatures.insert(name, sig);
    }

    pub fn get(&self, name: &str) -> Option<&FunctionSignature> {
        self.signatures.get(name)
    }

    pub fn find_matching(&self, analyzer: &SignatureAnalyzer, addr: Address) -> Result<Vec<String>, MemoryError> {
        let mut matches = Vec::new();

        for (name, sig) in &self.signatures {
            if analyzer.match_signature(sig, addr)? {
                matches.push(name.clone());
            }
        }

        Ok(matches)
    }

    pub fn len(&self) -> usize {
        self.signatures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &FunctionSignature)> {
        self.signatures.iter()
    }
}

impl Default for SignatureDatabase {
    fn default() -> Self {
        Self::new()
    }
}
