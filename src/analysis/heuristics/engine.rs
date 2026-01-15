// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::heuristics::patterns::HeuristicPattern;
use crate::analysis::heuristics::rules::HeuristicRule;
use crate::analysis::heuristics::scoring::HeuristicScorer;
use crate::analysis::heuristics::detector::OffsetDetector;
use crate::finders::result::FinderResults;
use std::sync::Arc;
use std::collections::HashMap;

pub struct HeuristicsEngine {
    reader: Arc<dyn MemoryReader>,
    patterns: Vec<HeuristicPattern>,
    rules: Vec<Box<dyn HeuristicRule>>,
    scorer: HeuristicScorer,
    detector: OffsetDetector,
    config: HeuristicsConfig,
}

impl HeuristicsEngine {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader: reader.clone(),
            patterns: Vec::new(),
            rules: Self::default_rules(),
            scorer: HeuristicScorer::new(),
            detector: OffsetDetector::new(reader),
            config: HeuristicsConfig::default(),
        }
    }

    fn default_rules() -> Vec<Box<dyn HeuristicRule>> {
        vec![
            Box::new(FunctionPrologueRule::new()),
            Box::new(StackAccessRule::new()),
            Box::new(GlobalAccessRule::new()),
            Box::new(StringReferenceRule::new()),
            Box::new(VTableAccessRule::new()),
        ]
    }

    pub fn add_pattern(&mut self, pattern: HeuristicPattern) {
        self.patterns.push(pattern);
    }

    pub fn add_rule(&mut self, rule: Box<dyn HeuristicRule>) {
        self.rules.push(rule);
    }

    pub fn analyze(&self, start: Address, end: Address) -> Result<FinderResults, MemoryError> {
        let mut results = FinderResults::new();

        let functions = self.detect_functions(start, end)?;
        for (name, addr) in functions {
            results.functions.insert(name, addr);
        }

        let offsets = self.detect_structure_offsets(start, end)?;
        for (struct_name, fields) in offsets {
            for (field, offset) in fields {
                results.structure_offsets
                    .entry(struct_name.clone())
                    .or_default()
                    .insert(field, offset);
            }
        }

        Ok(results)
    }

    fn detect_functions(&self, start: Address, end: Address) -> Result<HashMap<String, Address>, MemoryError> {
        let mut functions = HashMap::new();
        let step = 4;
        let mut current = start;

        while current < end {
            if self.is_likely_function_start(current)? {
                let name = format!("sub_{:X}", current.as_u64());
                functions.insert(name, current);
            }
            current = current + step;
        }

        Ok(functions)
    }

    fn is_likely_function_start(&self, addr: Address) -> Result<bool, MemoryError> {
        let bytes = self.reader.read_bytes(addr, 16)?;

        let inst0 = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if (inst0 & 0xFFC003E0) == 0xA9800000 {
            return Ok(true);
        }

        if (inst0 & 0xFF0003E0) == 0xD10003E0 {
            return Ok(true);
        }

        if inst0 == 0xD503237F {
            return Ok(true);
        }

        Ok(false)
    }

    fn detect_structure_offsets(&self, start: Address, end: Address) -> Result<HashMap<String, HashMap<String, u64>>, MemoryError> {
        let mut offsets: HashMap<String, HashMap<String, u64>> = HashMap::new();

        let detected = self.detector.detect_offsets(start, end)?;

        for (category, fields) in detected {
            for (field_name, offset) in fields {
                offsets.entry(category.clone())
                    .or_default()
                    .insert(field_name, offset);
            }
        }

        Ok(offsets)
    }

    pub fn analyze_function(&self, addr: Address) -> Result<FunctionAnalysis, MemoryError> {
        let mut analysis = FunctionAnalysis::new(addr);

        let prologue_info = self.analyze_prologue(addr)?;
        analysis.stack_frame_size = prologue_info.stack_size;
        analysis.saved_registers = prologue_info.saved_registers;

        let body_info = self.analyze_function_body(addr, 1000)?;
        analysis.called_functions = body_info.called_functions;
        analysis.accessed_globals = body_info.accessed_globals;
        analysis.string_references = body_info.string_references;

        analysis.estimated_complexity = self.estimate_complexity(&analysis);

        Ok(analysis)
    }

    fn analyze_prologue(&self, addr: Address) -> Result<PrologueInfo, MemoryError> {
        let mut info = PrologueInfo::new();
        let bytes = self.reader.read_bytes(addr, 32)?;

        for i in (0..32).step_by(4) {
            if i + 4 > bytes.len() {
                break;
            }
            let inst = u32::from_le_bytes([bytes[i], bytes[i+1], bytes[i+2], bytes[i+3]]);

            if (inst & 0xFFC003E0) == 0xA9800000 {
                let imm = ((inst >> 15) & 0x7F) as i32;
                info.stack_size += (imm * 8) as u64;
                let rt = inst & 0x1F;
                let rt2 = (inst >> 10) & 0x1F;
                info.saved_registers.push(rt as u8);
                info.saved_registers.push(rt2 as u8);
            }

            if (inst & 0xFF0003E0) == 0xD10003E0 {
                let imm = ((inst >> 10) & 0xFFF) as u64;
                info.stack_size = imm;
            }
        }

        Ok(info)
    }

    fn analyze_function_body(&self, addr: Address, max_size: usize) -> Result<BodyInfo, MemoryError> {
        let mut info = BodyInfo::new();
        let bytes = self.reader.read_bytes(addr, max_size)?;

        for i in (0..bytes.len()).step_by(4) {
            if i + 4 > bytes.len() {
                break;
            }
            let inst = u32::from_le_bytes([bytes[i], bytes[i+1], bytes[i+2], bytes[i+3]]);

            if (inst >> 26) == 0x25 {
                let offset = (inst & 0x03FFFFFF) as i32;
                let offset = if offset & 0x02000000 != 0 {
                    offset | !0x03FFFFFF
                } else {
                    offset
                };
                let target = (addr.as_u64() as i64 + (i as i64) + (offset as i64 * 4)) as u64;
                info.called_functions.push(Address::new(target));
            }

            if (inst >> 24) == 0x90 {
                let rd = inst & 0x1F;
                let immhi = ((inst >> 5) & 0x7FFFF) as i64;
                let immlo = ((inst >> 29) & 0x3) as i64;
                let page = ((immhi << 2) | immlo) << 12;
                info.accessed_globals.push(page as u64);
            }
        }

        Ok(info)
    }

    fn estimate_complexity(&self, analysis: &FunctionAnalysis) -> u32 {
        let mut complexity = 1;

        complexity += analysis.called_functions.len() as u32;
        complexity += (analysis.accessed_globals.len() / 2) as u32;
        complexity += analysis.string_references.len() as u32;

        if analysis.stack_frame_size > 256 {
            complexity += 2;
        }

        complexity
    }

    pub fn configure(&mut self, config: HeuristicsConfig) {
        self.config = config;
    }
}

struct FunctionPrologueRule;

impl FunctionPrologueRule {
    fn new() -> Self {
        Self
    }
}

impl HeuristicRule for FunctionPrologueRule {
    fn name(&self) -> &str {
        "FunctionPrologue"
    }

    fn check(&self, data: &[u8], addr: Address) -> Option<HeuristicMatch> {
        if data.len() < 4 {
            return None;
        }

        let inst = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        if (inst & 0xFFC003E0) == 0xA9800000 {
            return Some(HeuristicMatch {
                rule: self.name().to_string(),
                address: addr,
                confidence: 0.9,
                description: "STP prologue detected".to_string(),
            });
        }

        if (inst & 0xFF0003E0) == 0xD10003E0 {
            return Some(HeuristicMatch {
                rule: self.name().to_string(),
                address: addr,
                confidence: 0.8,
                description: "SUB SP prologue detected".to_string(),
            });
        }

        None
    }
}

struct StackAccessRule;

impl StackAccessRule {
    fn new() -> Self {
        Self
    }
}

impl HeuristicRule for StackAccessRule {
    fn name(&self) -> &str {
        "StackAccess"
    }

    fn check(&self, data: &[u8], addr: Address) -> Option<HeuristicMatch> {
        if data.len() < 4 {
            return None;
        }

        let inst = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        if (inst >> 22) == 0x3E5 || (inst >> 22) == 0x3E4 {
            let rn = (inst >> 5) & 0x1F;
            if rn == 31 {
                return Some(HeuristicMatch {
                    rule: self.name().to_string(),
                    address: addr,
                    confidence: 0.7,
                    description: "Stack access detected".to_string(),
                });
            }
        }

        None
    }
}

struct GlobalAccessRule;

impl GlobalAccessRule {
    fn new() -> Self {
        Self
    }
}

impl HeuristicRule for GlobalAccessRule {
    fn name(&self) -> &str {
        "GlobalAccess"
    }

    fn check(&self, data: &[u8], addr: Address) -> Option<HeuristicMatch> {
        if data.len() < 8 {
            return None;
        }

        let inst0 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let inst1 = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

        if (inst0 >> 24) == 0x90 && ((inst1 >> 22) == 0x3E5 || (inst1 >> 24) == 0x91) {
            return Some(HeuristicMatch {
                rule: self.name().to_string(),
                address: addr,
                confidence: 0.85,
                description: "ADRP+ADD/LDR global access detected".to_string(),
            });
        }

        None
    }
}

struct StringReferenceRule;

impl StringReferenceRule {
    fn new() -> Self {
        Self
    }
}

impl HeuristicRule for StringReferenceRule {
    fn name(&self) -> &str {
        "StringReference"
    }

    fn check(&self, _data: &[u8], _addr: Address) -> Option<HeuristicMatch> {
        None
    }
}

struct VTableAccessRule;

impl VTableAccessRule {
    fn new() -> Self {
        Self
    }
}

impl HeuristicRule for VTableAccessRule {
    fn name(&self) -> &str {
        "VTableAccess"
    }

    fn check(&self, data: &[u8], addr: Address) -> Option<HeuristicMatch> {
        if data.len() < 12 {
            return None;
        }

        let inst0 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let inst1 = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let inst2 = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);

        if (inst0 >> 22) == 0x3E5 {
            if (inst1 >> 22) == 0x3E5 {
                if (inst2 >> 26) == 0x1A8 {
                    return Some(HeuristicMatch {
                        rule: self.name().to_string(),
                        address: addr,
                        confidence: 0.75,
                        description: "Potential vtable call detected".to_string(),
                    });
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct FunctionAnalysis {
    pub address: Address,
    pub stack_frame_size: u64,
    pub saved_registers: Vec<u8>,
    pub called_functions: Vec<Address>,
    pub accessed_globals: Vec<u64>,
    pub string_references: Vec<Address>,
    pub estimated_complexity: u32,
}

impl FunctionAnalysis {
    fn new(address: Address) -> Self {
        Self {
            address,
            stack_frame_size: 0,
            saved_registers: Vec::new(),
            called_functions: Vec::new(),
            accessed_globals: Vec::new(),
            string_references: Vec::new(),
            estimated_complexity: 0,
        }
    }
}

struct PrologueInfo {
    stack_size: u64,
    saved_registers: Vec<u8>,
}

impl PrologueInfo {
    fn new() -> Self {
        Self {
            stack_size: 0,
            saved_registers: Vec::new(),
        }
    }
}

struct BodyInfo {
    called_functions: Vec<Address>,
    accessed_globals: Vec<u64>,
    string_references: Vec<Address>,
}

impl BodyInfo {
    fn new() -> Self {
        Self {
            called_functions: Vec::new(),
            accessed_globals: Vec::new(),
            string_references: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeuristicMatch {
    pub rule: String,
    pub address: Address,
    pub confidence: f64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct HeuristicsConfig {
    pub min_confidence: f64,
    pub max_scan_size: usize,
    pub enable_learning: bool,
    pub parallel_scan: bool,
}

impl Default for HeuristicsConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            max_scan_size: 0x1000000,
            enable_learning: false,
            parallel_scan: true,
        }
    }
}
