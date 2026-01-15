// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::bytecode::decoder::{BytecodeDecoder, LuauInstruction, LuauOpcode};
use std::sync::Arc;
use std::collections::HashMap;

pub struct BytecodeAnalyzer {
    reader: Arc<dyn MemoryReader>,
    decoder: BytecodeDecoder,
}

impl BytecodeAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            decoder: BytecodeDecoder::new(reader.clone()),
            reader,
        }
    }

    pub fn analyze_function(&self, addr: Address, instruction_count: usize) -> FunctionAnalysis {
        let instructions = self.decoder.decode_function(addr, instruction_count);

        let mut analysis = FunctionAnalysis {
            address: addr,
            instruction_count: instructions.len(),
            opcode_histogram: HashMap::new(),
            call_targets: Vec::new(),
            jump_targets: Vec::new(),
            uses_upvalues: false,
            uses_varargs: false,
            has_loops: false,
            max_stack_size: 0,
            complexity_score: 0.0,
        };

        for insn in &instructions {
            *analysis.opcode_histogram.entry(insn.opcode).or_insert(0) += 1;

            match insn.opcode {
                LuauOpcode::GetUpval | LuauOpcode::SetUpval | LuauOpcode::CloseUpvals => {
                    analysis.uses_upvalues = true;
                }
                LuauOpcode::GetVarargs | LuauOpcode::PrepVarargs => {
                    analysis.uses_varargs = true;
                }
                LuauOpcode::JumpBack | LuauOpcode::ForNLoop | LuauOpcode::ForGLoop
                | LuauOpcode::ForGLoopINext | LuauOpcode::ForGLoopNext => {
                    analysis.has_loops = true;
                }
                LuauOpcode::Call | LuauOpcode::FastCall | LuauOpcode::FastCall1
                | LuauOpcode::FastCall2 | LuauOpcode::FastCall2K | LuauOpcode::FastCall3 => {
                    analysis.call_targets.push(insn.clone());
                }
                LuauOpcode::Jump | LuauOpcode::JumpX | LuauOpcode::JumpIf | LuauOpcode::JumpIfNot
                | LuauOpcode::JumpIfEq | LuauOpcode::JumpIfLe | LuauOpcode::JumpIfLt
                | LuauOpcode::JumpIfNotEq | LuauOpcode::JumpIfNotLe | LuauOpcode::JumpIfNotLt => {
                    analysis.jump_targets.push(insn.sbx() as i64);
                }
                _ => {}
            }

            let reg = insn.a as usize;
            if reg > analysis.max_stack_size {
                analysis.max_stack_size = reg;
            }
        }

        analysis.complexity_score = self.calculate_complexity(&analysis);

        analysis
    }

    fn calculate_complexity(&self, analysis: &FunctionAnalysis) -> f64 {
        let mut score = 0.0;

        score += analysis.instruction_count as f64 * 0.1;

        let unique_opcodes = analysis.opcode_histogram.len();
        score += unique_opcodes as f64 * 0.5;

        score += analysis.call_targets.len() as f64 * 1.0;

        score += analysis.jump_targets.len() as f64 * 0.3;

        if analysis.has_loops {
            score += 5.0;
        }

        if analysis.uses_upvalues {
            score += 2.0;
        }

        if analysis.uses_varargs {
            score += 1.5;
        }

        score
    }

    pub fn find_patterns(&self, addr: Address, instruction_count: usize) -> Vec<BytecodePattern> {
        let instructions = self.decoder.decode_function(addr, instruction_count);
        let mut patterns = Vec::new();

        for window_size in 2..=5 {
            for i in 0..instructions.len().saturating_sub(window_size - 1) {
                let window: Vec<_> = instructions[i..i + window_size].iter().collect();

                if let Some(pattern) = self.identify_pattern(&window) {
                    patterns.push(pattern);
                }
            }
        }

        patterns
    }

    fn identify_pattern(&self, window: &[&LuauInstruction]) -> Option<BytecodePattern> {
        if window.len() >= 2 {
            if matches!(window[0].opcode, LuauOpcode::NameCall)
                && matches!(window[1].opcode, LuauOpcode::Call)
            {
                return Some(BytecodePattern::MethodCall);
            }

            if matches!(window[0].opcode, LuauOpcode::GetGlobal)
                && matches!(window[1].opcode, LuauOpcode::Call)
            {
                return Some(BytecodePattern::GlobalCall);
            }
        }

        if window.len() >= 3 {
            if matches!(window[0].opcode, LuauOpcode::GetTable)
                && matches!(window[1].opcode, LuauOpcode::GetTable)
                && matches!(window[2].opcode, LuauOpcode::Call)
            {
                return Some(BytecodePattern::ChainedTableAccess);
            }

            if matches!(window[0].opcode, LuauOpcode::LoadN)
                && matches!(window[1].opcode, LuauOpcode::LoadN)
                && matches!(window[2].opcode, LuauOpcode::Add | LuauOpcode::Sub | LuauOpcode::Mul | LuauOpcode::Div)
            {
                return Some(BytecodePattern::ConstantArithmetic);
            }
        }

        if window.len() >= 2 {
            if matches!(window[0].opcode, LuauOpcode::ForNPrep)
                || matches!(window[0].opcode, LuauOpcode::ForGPrep)
            {
                return Some(BytecodePattern::LoopSetup);
            }
        }

        None
    }

    pub fn detect_obfuscation(&self, addr: Address, instruction_count: usize) -> ObfuscationIndicators {
        let instructions = self.decoder.decode_function(addr, instruction_count);

        let mut indicators = ObfuscationIndicators {
            nop_density: 0.0,
            jump_density: 0.0,
            dead_code_suspected: false,
            unusual_patterns: Vec::new(),
            obfuscation_score: 0.0,
        };

        if instructions.is_empty() {
            return indicators;
        }

        let nop_count = instructions.iter()
            .filter(|i| matches!(i.opcode, LuauOpcode::Nop))
            .count();
        indicators.nop_density = nop_count as f64 / instructions.len() as f64;

        let jump_count = instructions.iter()
            .filter(|i| matches!(
                i.opcode,
                LuauOpcode::Jump | LuauOpcode::JumpX | LuauOpcode::JumpBack
            ))
            .count();
        indicators.jump_density = jump_count as f64 / instructions.len() as f64;

        let mut consecutive_nops = 0;
        for insn in &instructions {
            if matches!(insn.opcode, LuauOpcode::Nop) {
                consecutive_nops += 1;
                if consecutive_nops >= 3 {
                    indicators.dead_code_suspected = true;
                    indicators.unusual_patterns.push("consecutive_nops".to_string());
                }
            } else {
                consecutive_nops = 0;
            }
        }

        for i in 0..instructions.len().saturating_sub(1) {
            if matches!(instructions[i].opcode, LuauOpcode::Jump) {
                let target = instructions[i].sbx();
                if target == 0 || target == 1 {
                    indicators.unusual_patterns.push("short_unconditional_jump".to_string());
                }
            }
        }

        indicators.obfuscation_score = indicators.nop_density * 10.0
            + indicators.jump_density * 5.0
            + if indicators.dead_code_suspected { 3.0 } else { 0.0 }
            + indicators.unusual_patterns.len() as f64 * 0.5;

        indicators
    }
}

#[derive(Debug, Clone)]
pub struct FunctionAnalysis {
    pub address: Address,
    pub instruction_count: usize,
    pub opcode_histogram: HashMap<LuauOpcode, usize>,
    pub call_targets: Vec<LuauInstruction>,
    pub jump_targets: Vec<i64>,
    pub uses_upvalues: bool,
    pub uses_varargs: bool,
    pub has_loops: bool,
    pub max_stack_size: usize,
    pub complexity_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BytecodePattern {
    MethodCall,
    GlobalCall,
    ChainedTableAccess,
    ConstantArithmetic,
    LoopSetup,
    TableConstruction,
    StringConcatenation,
    ClosureCapture,
}

#[derive(Debug, Clone)]
pub struct ObfuscationIndicators {
    pub nop_density: f64,
    pub jump_density: f64,
    pub dead_code_suspected: bool,
    pub unusual_patterns: Vec<String>,
    pub obfuscation_score: f64,
}

impl ObfuscationIndicators {
    pub fn is_likely_obfuscated(&self) -> bool {
        self.obfuscation_score >= 5.0
    }
}
