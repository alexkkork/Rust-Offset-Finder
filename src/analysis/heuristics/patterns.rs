// Tue Jan 13 2026 - Alex

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HeuristicPattern {
    pub name: String,
    pub description: String,
    pub pattern_type: PatternType,
    pub instructions: Vec<InstructionPattern>,
    pub confidence_base: f64,
    pub metadata: PatternMetadata,
}

impl HeuristicPattern {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            pattern_type: PatternType::Generic,
            instructions: Vec::new(),
            confidence_base: 0.5,
            metadata: PatternMetadata::new(),
        }
    }

    pub fn with_type(mut self, pattern_type: PatternType) -> Self {
        self.pattern_type = pattern_type;
        self
    }

    pub fn with_instruction(mut self, instruction: InstructionPattern) -> Self {
        self.instructions.push(instruction);
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence_base = confidence;
        self
    }

    pub fn matches(&self, data: &[u8]) -> Option<PatternMatch> {
        if data.len() < self.min_size() {
            return None;
        }

        let mut offset = 0;
        let mut matched_instructions = Vec::new();

        for instr_pattern in &self.instructions {
            if offset + 4 > data.len() {
                return None;
            }

            let inst = u32::from_le_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
            ]);

            if !instr_pattern.matches(inst) {
                return None;
            }

            matched_instructions.push(MatchedInstruction {
                offset,
                instruction: inst,
                pattern: instr_pattern.clone(),
            });

            offset += 4;
        }

        Some(PatternMatch {
            pattern_name: self.name.clone(),
            confidence: self.calculate_confidence(&matched_instructions),
            matched_instructions,
            captured_values: HashMap::new(),
        })
    }

    fn min_size(&self) -> usize {
        self.instructions.len() * 4
    }

    fn calculate_confidence(&self, matches: &[MatchedInstruction]) -> f64 {
        let mut confidence = self.confidence_base;

        for matched in matches {
            confidence += matched.pattern.specificity * 0.1;
        }

        confidence.min(1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    Generic,
    FunctionPrologue,
    FunctionEpilogue,
    MethodCall,
    PropertyAccess,
    GlobalAccess,
    StringReference,
    VTableCall,
    LuaApiCall,
    StackManipulation,
    BranchPattern,
    LoopPattern,
}

impl PatternType {
    pub fn name(&self) -> &'static str {
        match self {
            PatternType::Generic => "Generic",
            PatternType::FunctionPrologue => "Function Prologue",
            PatternType::FunctionEpilogue => "Function Epilogue",
            PatternType::MethodCall => "Method Call",
            PatternType::PropertyAccess => "Property Access",
            PatternType::GlobalAccess => "Global Access",
            PatternType::StringReference => "String Reference",
            PatternType::VTableCall => "VTable Call",
            PatternType::LuaApiCall => "Lua API Call",
            PatternType::StackManipulation => "Stack Manipulation",
            PatternType::BranchPattern => "Branch Pattern",
            PatternType::LoopPattern => "Loop Pattern",
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstructionPattern {
    pub mask: u32,
    pub value: u32,
    pub name: String,
    pub specificity: f64,
    pub capture_groups: Vec<CaptureGroup>,
}

impl InstructionPattern {
    pub fn new(name: &str, mask: u32, value: u32) -> Self {
        Self {
            mask,
            value,
            name: name.to_string(),
            specificity: Self::calculate_specificity(mask),
            capture_groups: Vec::new(),
        }
    }

    pub fn with_capture(mut self, name: &str, bit_start: u8, bit_length: u8) -> Self {
        self.capture_groups.push(CaptureGroup {
            name: name.to_string(),
            bit_start,
            bit_length,
        });
        self
    }

    pub fn matches(&self, instruction: u32) -> bool {
        (instruction & self.mask) == self.value
    }

    pub fn extract_captures(&self, instruction: u32) -> HashMap<String, u64> {
        let mut captures = HashMap::new();

        for group in &self.capture_groups {
            let mask = ((1u64 << group.bit_length) - 1) as u32;
            let value = (instruction >> group.bit_start) & mask;
            captures.insert(group.name.clone(), value as u64);
        }

        captures
    }

    fn calculate_specificity(mask: u32) -> f64 {
        let bit_count = mask.count_ones();
        bit_count as f64 / 32.0
    }

    pub fn stp_prologue() -> Self {
        Self::new("STP Prologue", 0xFFC003E0, 0xA9800000)
            .with_capture("rt", 0, 5)
            .with_capture("rt2", 10, 5)
            .with_capture("imm", 15, 7)
    }

    pub fn sub_sp() -> Self {
        Self::new("SUB SP", 0xFF0003FF, 0xD10003FF)
            .with_capture("imm", 10, 12)
    }

    pub fn add_sp() -> Self {
        Self::new("ADD SP", 0xFF0003FF, 0x910003FF)
            .with_capture("imm", 10, 12)
    }

    pub fn bl() -> Self {
        Self::new("BL", 0xFC000000, 0x94000000)
            .with_capture("imm26", 0, 26)
    }

    pub fn blr() -> Self {
        Self::new("BLR", 0xFFFFFC1F, 0xD63F0000)
            .with_capture("rn", 5, 5)
    }

    pub fn ret() -> Self {
        Self::new("RET", 0xFFFFFFFF, 0xD65F03C0)
    }

    pub fn adrp() -> Self {
        Self::new("ADRP", 0x9F000000, 0x90000000)
            .with_capture("rd", 0, 5)
            .with_capture("immlo", 29, 2)
            .with_capture("immhi", 5, 19)
    }

    pub fn ldr_imm() -> Self {
        Self::new("LDR Imm", 0xFFC00000, 0xF9400000)
            .with_capture("rt", 0, 5)
            .with_capture("rn", 5, 5)
            .with_capture("imm12", 10, 12)
    }

    pub fn str_imm() -> Self {
        Self::new("STR Imm", 0xFFC00000, 0xF9000000)
            .with_capture("rt", 0, 5)
            .with_capture("rn", 5, 5)
            .with_capture("imm12", 10, 12)
    }

    pub fn mov_reg() -> Self {
        Self::new("MOV Reg", 0xFFE0FFE0, 0xAA0003E0)
            .with_capture("rd", 0, 5)
            .with_capture("rm", 16, 5)
    }

    pub fn cbz() -> Self {
        Self::new("CBZ", 0xFF000000, 0xB4000000)
            .with_capture("rt", 0, 5)
            .with_capture("imm19", 5, 19)
    }

    pub fn cbnz() -> Self {
        Self::new("CBNZ", 0xFF000000, 0xB5000000)
            .with_capture("rt", 0, 5)
            .with_capture("imm19", 5, 19)
    }

    pub fn b_cond() -> Self {
        Self::new("B.cond", 0xFF000010, 0x54000000)
            .with_capture("cond", 0, 4)
            .with_capture("imm19", 5, 19)
    }
}

#[derive(Debug, Clone)]
pub struct CaptureGroup {
    pub name: String,
    pub bit_start: u8,
    pub bit_length: u8,
}

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub confidence: f64,
    pub matched_instructions: Vec<MatchedInstruction>,
    pub captured_values: HashMap<String, u64>,
}

impl PatternMatch {
    pub fn get_capture(&self, name: &str) -> Option<u64> {
        self.captured_values.get(name).copied()
    }

    pub fn instruction_count(&self) -> usize {
        self.matched_instructions.len()
    }
}

#[derive(Debug, Clone)]
pub struct MatchedInstruction {
    pub offset: usize,
    pub instruction: u32,
    pub pattern: InstructionPattern,
}

#[derive(Debug, Clone, Default)]
pub struct PatternMetadata {
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub related_patterns: Vec<String>,
}

impl PatternMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn with_related(mut self, pattern: &str) -> Self {
        self.related_patterns.push(pattern.to_string());
        self
    }
}

pub struct PatternLibrary {
    patterns: HashMap<String, HeuristicPattern>,
}

impl PatternLibrary {
    pub fn new() -> Self {
        let mut library = Self {
            patterns: HashMap::new(),
        };
        library.load_default_patterns();
        library
    }

    fn load_default_patterns(&mut self) {
        self.add(self.create_function_prologue_pattern());
        self.add(self.create_function_epilogue_pattern());
        self.add(self.create_global_access_pattern());
        self.add(self.create_vtable_call_pattern());
        self.add(self.create_method_call_pattern());
    }

    fn create_function_prologue_pattern(&self) -> HeuristicPattern {
        HeuristicPattern::new("arm64_prologue", "Standard ARM64 function prologue")
            .with_type(PatternType::FunctionPrologue)
            .with_instruction(InstructionPattern::stp_prologue())
            .with_instruction(InstructionPattern::sub_sp())
            .with_confidence(0.9)
    }

    fn create_function_epilogue_pattern(&self) -> HeuristicPattern {
        HeuristicPattern::new("arm64_epilogue", "Standard ARM64 function epilogue")
            .with_type(PatternType::FunctionEpilogue)
            .with_instruction(InstructionPattern::add_sp())
            .with_instruction(InstructionPattern::ret())
            .with_confidence(0.85)
    }

    fn create_global_access_pattern(&self) -> HeuristicPattern {
        HeuristicPattern::new("global_access", "ADRP + LDR/STR global data access")
            .with_type(PatternType::GlobalAccess)
            .with_instruction(InstructionPattern::adrp())
            .with_instruction(InstructionPattern::ldr_imm())
            .with_confidence(0.8)
    }

    fn create_vtable_call_pattern(&self) -> HeuristicPattern {
        HeuristicPattern::new("vtable_call", "Virtual table method call")
            .with_type(PatternType::VTableCall)
            .with_instruction(InstructionPattern::ldr_imm())
            .with_instruction(InstructionPattern::ldr_imm())
            .with_instruction(InstructionPattern::blr())
            .with_confidence(0.75)
    }

    fn create_method_call_pattern(&self) -> HeuristicPattern {
        HeuristicPattern::new("method_call", "Direct method/function call")
            .with_type(PatternType::MethodCall)
            .with_instruction(InstructionPattern::bl())
            .with_confidence(0.7)
    }

    pub fn add(&mut self, pattern: HeuristicPattern) {
        self.patterns.insert(pattern.name.clone(), pattern);
    }

    pub fn get(&self, name: &str) -> Option<&HeuristicPattern> {
        self.patterns.get(name)
    }

    pub fn find_matches(&self, data: &[u8]) -> Vec<(usize, PatternMatch)> {
        let mut matches = Vec::new();

        for offset in (0..data.len()).step_by(4) {
            let remaining = &data[offset..];

            for pattern in self.patterns.values() {
                if let Some(m) = pattern.matches(remaining) {
                    matches.push((offset, m));
                }
            }
        }

        matches
    }

    pub fn patterns_by_type(&self, pattern_type: PatternType) -> Vec<&HeuristicPattern> {
        self.patterns.values()
            .filter(|p| p.pattern_type == pattern_type)
            .collect()
    }

    pub fn all_patterns(&self) -> impl Iterator<Item = &HeuristicPattern> {
        self.patterns.values()
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

impl Default for PatternLibrary {
    fn default() -> Self {
        Self::new()
    }
}
