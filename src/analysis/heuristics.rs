// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::disassembler::{Disassembler, DisassembledInstruction};
use crate::analysis::function::AnalyzedFunction;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

pub struct HeuristicAnalyzer {
    reader: Arc<dyn MemoryReader>,
    disassembler: Arc<Disassembler>,
    confidence_thresholds: ConfidenceThresholds,
}

#[derive(Debug, Clone)]
pub struct ConfidenceThresholds {
    pub minimum_confidence: f64,
    pub high_confidence: f64,
    pub function_detection: f64,
    pub offset_validation: f64,
}

impl Default for ConfidenceThresholds {
    fn default() -> Self {
        Self {
            minimum_confidence: 0.3,
            high_confidence: 0.8,
            function_detection: 0.6,
            offset_validation: 0.7,
        }
    }
}

impl HeuristicAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>, disassembler: Arc<Disassembler>) -> Self {
        Self {
            reader,
            disassembler,
            confidence_thresholds: ConfidenceThresholds::default(),
        }
    }

    pub fn with_thresholds(mut self, thresholds: ConfidenceThresholds) -> Self {
        self.confidence_thresholds = thresholds;
        self
    }

    pub fn is_function_entry(&self, addr: Address) -> Result<HeuristicResult, MemoryError> {
        let mut confidence = 0.0;
        let mut evidence = Vec::new();

        let instructions = self.disassembler.disassemble_function(addr, 32)?;

        if instructions.is_empty() {
            return Ok(HeuristicResult {
                confidence: 0.0,
                evidence,
                conclusion: HeuristicConclusion::Unlikely,
            });
        }

        if let Some(first) = instructions.first() {
            if first.mnemonic == "STP" && first.op_str.contains("X29") && first.op_str.contains("X30") {
                confidence += 0.35;
                evidence.push("Standard function prologue (STP X29, X30)".to_string());
            }

            if first.mnemonic == "SUB" && first.op_str.contains("SP") {
                confidence += 0.15;
                evidence.push("Stack allocation at function start".to_string());
            }

            if first.mnemonic == "STP" || first.mnemonic == "STR" {
                if first.op_str.contains("X19") || first.op_str.contains("X20") ||
                   first.op_str.contains("X21") || first.op_str.contains("X22") {
                    confidence += 0.1;
                    evidence.push("Callee-saved register preservation".to_string());
                }
            }
        }

        if let Some(second) = instructions.get(1) {
            if second.mnemonic == "MOV" && second.op_str.contains("X29") && second.op_str.contains("SP") {
                confidence += 0.2;
                evidence.push("Frame pointer setup (MOV X29, SP)".to_string());
            }
        }

        let has_ret = instructions.iter().any(|i| i.mnemonic == "RET");
        if has_ret {
            confidence += 0.1;
            evidence.push("Contains RET instruction".to_string());
        }

        let aligned = addr.as_u64() % 4 == 0;
        if aligned {
            confidence += 0.05;
            evidence.push("Address is 4-byte aligned".to_string());
        }

        if addr.as_u64() % 16 == 0 {
            confidence += 0.05;
            evidence.push("Address is 16-byte aligned".to_string());
        }

        let conclusion = if confidence >= self.confidence_thresholds.high_confidence {
            HeuristicConclusion::HighlyLikely
        } else if confidence >= self.confidence_thresholds.function_detection {
            HeuristicConclusion::Likely
        } else if confidence >= self.confidence_thresholds.minimum_confidence {
            HeuristicConclusion::Possible
        } else {
            HeuristicConclusion::Unlikely
        };

        Ok(HeuristicResult {
            confidence: confidence.min(1.0),
            evidence,
            conclusion,
        })
    }

    pub fn is_lua_state_pointer(&self, value: u64) -> Result<HeuristicResult, MemoryError> {
        let mut confidence = 0.0;
        let mut evidence = Vec::new();

        if value == 0 || value % 8 != 0 {
            return Ok(HeuristicResult {
                confidence: 0.0,
                evidence: vec!["Invalid pointer value".to_string()],
                conclusion: HeuristicConclusion::Unlikely,
            });
        }

        let addr = Address::new(value);

        let header_bytes = self.reader.read_bytes(addr, 64)?;

        let potential_top = u64::from_le_bytes(header_bytes[0x10..0x18].try_into().unwrap_or([0; 8]));
        let potential_stack = u64::from_le_bytes(header_bytes[0x18..0x20].try_into().unwrap_or([0; 8]));

        if potential_top > potential_stack && potential_top - potential_stack < 0x100000 {
            confidence += 0.2;
            evidence.push("Stack/top relationship looks valid".to_string());
        }

        if potential_top % 16 == 0 && potential_stack % 16 == 0 {
            confidence += 0.1;
            evidence.push("Stack pointers are 16-byte aligned".to_string());
        }

        let gcheader = u64::from_le_bytes(header_bytes[0..8].try_into().unwrap_or([0; 8]));
        if gcheader != 0 && gcheader % 8 == 0 {
            confidence += 0.1;
            evidence.push("GC header pointer looks valid".to_string());
        }

        let type_tag = header_bytes[8];
        if type_tag == 8 {
            confidence += 0.3;
            evidence.push("Type tag matches LUA_TTHREAD".to_string());
        }

        let conclusion = if confidence >= self.confidence_thresholds.high_confidence {
            HeuristicConclusion::HighlyLikely
        } else if confidence >= self.confidence_thresholds.offset_validation {
            HeuristicConclusion::Likely
        } else if confidence >= self.confidence_thresholds.minimum_confidence {
            HeuristicConclusion::Possible
        } else {
            HeuristicConclusion::Unlikely
        };

        Ok(HeuristicResult {
            confidence: confidence.min(1.0),
            evidence,
            conclusion,
        })
    }

    pub fn is_vtable_pointer(&self, value: u64) -> Result<HeuristicResult, MemoryError> {
        let mut confidence = 0.0;
        let mut evidence = Vec::new();

        if value == 0 || value % 8 != 0 {
            return Ok(HeuristicResult {
                confidence: 0.0,
                evidence: vec!["Invalid pointer value".to_string()],
                conclusion: HeuristicConclusion::Unlikely,
            });
        }

        let addr = Address::new(value);

        let vtable_bytes = self.reader.read_bytes(addr, 64)?;

        let mut valid_pointers = 0;
        for i in 0..8 {
            let entry = u64::from_le_bytes(vtable_bytes[i*8..(i+1)*8].try_into().unwrap_or([0; 8]));

            if entry != 0 && entry % 4 == 0 && entry > 0x100000000 && entry < 0x800000000000 {
                valid_pointers += 1;
            }
        }

        if valid_pointers >= 4 {
            confidence += 0.4;
            evidence.push(format!("{}/8 vtable entries look like valid function pointers", valid_pointers));
        } else if valid_pointers >= 2 {
            confidence += 0.2;
            evidence.push(format!("{}/8 vtable entries look like valid function pointers", valid_pointers));
        }

        let first_entry = u64::from_le_bytes(vtable_bytes[0..8].try_into().unwrap_or([0; 8]));
        if first_entry != 0 {
            let first_entry_addr = Address::new(first_entry);
            if let Ok(result) = self.is_function_entry(first_entry_addr) {
                if result.conclusion == HeuristicConclusion::HighlyLikely ||
                   result.conclusion == HeuristicConclusion::Likely {
                    confidence += 0.3;
                    evidence.push("First vtable entry points to valid function".to_string());
                }
            }
        }

        let conclusion = if confidence >= self.confidence_thresholds.high_confidence {
            HeuristicConclusion::HighlyLikely
        } else if confidence >= self.confidence_thresholds.offset_validation {
            HeuristicConclusion::Likely
        } else if confidence >= self.confidence_thresholds.minimum_confidence {
            HeuristicConclusion::Possible
        } else {
            HeuristicConclusion::Unlikely
        };

        Ok(HeuristicResult {
            confidence: confidence.min(1.0),
            evidence,
            conclusion,
        })
    }

    pub fn identify_function_purpose(&self, function: &AnalyzedFunction) -> FunctionPurpose {
        let mut scores: HashMap<FunctionPurposeType, f64> = HashMap::new();

        if function.is_leaf() && function.stack_size == 0 {
            *scores.entry(FunctionPurposeType::Getter).or_insert(0.0) += 0.3;
        }

        if function.called_functions.len() == 1 && function.is_leaf() {
            *scores.entry(FunctionPurposeType::Wrapper).or_insert(0.0) += 0.4;
        }

        if function.block_count() == 1 && function.instruction_count() < 10 {
            *scores.entry(FunctionPurposeType::Getter).or_insert(0.0) += 0.2;
            *scores.entry(FunctionPurposeType::Setter).or_insert(0.0) += 0.2;
        }

        if function.block_count() > 10 {
            *scores.entry(FunctionPurposeType::Complex).or_insert(0.0) += 0.3;
        }

        if function.data_references.len() > 5 {
            *scores.entry(FunctionPurposeType::DataProcessor).or_insert(0.0) += 0.3;
        }

        if function.called_functions.len() > 10 {
            *scores.entry(FunctionPurposeType::Dispatcher).or_insert(0.0) += 0.3;
        }

        let (purpose_type, confidence) = scores.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((FunctionPurposeType::Unknown, 0.0));

        FunctionPurpose {
            purpose_type,
            confidence,
        }
    }

    pub fn estimate_struct_size(&self, base_addr: Address, max_size: usize) -> Result<StructSizeEstimate, MemoryError> {
        let data = self.reader.read_bytes(base_addr, max_size)?;

        let mut last_non_zero = 0;
        for (i, byte) in data.iter().enumerate().rev() {
            if *byte != 0 {
                last_non_zero = i + 1;
                break;
            }
        }

        let aligned_size = ((last_non_zero + 7) / 8) * 8;

        let mut null_runs = 0;
        let mut current_run = 0;

        for byte in &data[..aligned_size.min(data.len())] {
            if *byte == 0 {
                current_run += 1;
            } else {
                if current_run >= 8 {
                    null_runs += 1;
                }
                current_run = 0;
            }
        }

        let confidence = if null_runs > 3 {
            0.3
        } else if last_non_zero == 0 {
            0.1
        } else {
            0.6
        };

        Ok(StructSizeEstimate {
            estimated_size: aligned_size,
            confidence,
            padding_detected: null_runs > 0,
        })
    }

    pub fn detect_calling_convention(&self, function: &AnalyzedFunction) -> CallingConvention {
        let uses_x0_x7 = function.register_uses.keys()
            .any(|r| r.starts_with("X") || r.starts_with("W"));

        let saves_callee = function.saved_registers.iter()
            .any(|r| r.starts_with("X19") || r.starts_with("X20"));

        if uses_x0_x7 && saves_callee {
            CallingConvention::Arm64Aapcs
        } else if function.has_frame_pointer {
            CallingConvention::Arm64Aapcs
        } else {
            CallingConvention::Unknown
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeuristicResult {
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub conclusion: HeuristicConclusion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeuristicConclusion {
    HighlyLikely,
    Likely,
    Possible,
    Unlikely,
}

impl HeuristicConclusion {
    pub fn is_positive(&self) -> bool {
        matches!(self, HeuristicConclusion::HighlyLikely | HeuristicConclusion::Likely)
    }
}

#[derive(Debug, Clone)]
pub struct FunctionPurpose {
    pub purpose_type: FunctionPurposeType,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionPurposeType {
    Getter,
    Setter,
    Constructor,
    Destructor,
    Wrapper,
    Dispatcher,
    DataProcessor,
    Complex,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct StructSizeEstimate {
    pub estimated_size: usize,
    pub confidence: f64,
    pub padding_detected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    Arm64Aapcs,
    Arm64AapcsVfp,
    Unknown,
}

pub struct HeuristicCache {
    function_entries: HashMap<u64, HeuristicResult>,
    vtable_pointers: HashMap<u64, HeuristicResult>,
    lua_state_pointers: HashMap<u64, HeuristicResult>,
}

impl HeuristicCache {
    pub fn new() -> Self {
        Self {
            function_entries: HashMap::new(),
            vtable_pointers: HashMap::new(),
            lua_state_pointers: HashMap::new(),
        }
    }

    pub fn get_function_entry(&self, addr: u64) -> Option<&HeuristicResult> {
        self.function_entries.get(&addr)
    }

    pub fn cache_function_entry(&mut self, addr: u64, result: HeuristicResult) {
        self.function_entries.insert(addr, result);
    }

    pub fn get_vtable(&self, addr: u64) -> Option<&HeuristicResult> {
        self.vtable_pointers.get(&addr)
    }

    pub fn cache_vtable(&mut self, addr: u64, result: HeuristicResult) {
        self.vtable_pointers.insert(addr, result);
    }

    pub fn get_lua_state(&self, addr: u64) -> Option<&HeuristicResult> {
        self.lua_state_pointers.get(&addr)
    }

    pub fn cache_lua_state(&mut self, addr: u64, result: HeuristicResult) {
        self.lua_state_pointers.insert(addr, result);
    }

    pub fn clear(&mut self) {
        self.function_entries.clear();
        self.vtable_pointers.clear();
        self.lua_state_pointers.clear();
    }

    pub fn stats(&self) -> HeuristicCacheStats {
        HeuristicCacheStats {
            function_entries: self.function_entries.len(),
            vtable_pointers: self.vtable_pointers.len(),
            lua_state_pointers: self.lua_state_pointers.len(),
        }
    }
}

impl Default for HeuristicCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct HeuristicCacheStats {
    pub function_entries: usize,
    pub vtable_pointers: usize,
    pub lua_state_pointers: usize,
}

impl HeuristicCacheStats {
    pub fn total(&self) -> usize {
        self.function_entries + self.vtable_pointers + self.lua_state_pointers
    }
}
