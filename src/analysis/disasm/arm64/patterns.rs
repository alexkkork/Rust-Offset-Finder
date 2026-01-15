// Wed Jan 15 2026 - Alex

use super::{Arm64Instruction, Arm64Operand, Arm64Register};
use crate::memory::Address;

pub struct Arm64Patterns;

impl Arm64Patterns {
    pub fn is_function_prologue(instructions: &[Arm64Instruction]) -> bool {
        if instructions.is_empty() {
            return false;
        }

        let first = &instructions[0];

        if first.mnemonic == "STP" {
            if let Some(Arm64Operand::Memory { base: Arm64Register::Sp, offset, pre_index: true, .. }) = instructions[0].operands.get(2) {
                if *offset < 0 {
                    return true;
                }
            }
        }

        if first.mnemonic == "SUB" {
            if let (Some(Arm64Operand::Register(Arm64Register::Sp)), Some(Arm64Operand::Register(Arm64Register::Sp))) = 
                (first.operands.get(0), first.operands.get(1)) {
                return true;
            }
        }

        if instructions.len() >= 2 {
            let second = &instructions[1];
            if first.mnemonic == "STP" && second.mnemonic == "ADD" {
                if let (Some(Arm64Operand::Register(Arm64Register::X(29))), Some(Arm64Operand::Register(Arm64Register::Sp))) =
                    (second.operands.get(0), second.operands.get(1)) {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_function_epilogue(instructions: &[Arm64Instruction]) -> bool {
        if instructions.is_empty() {
            return false;
        }

        let last = instructions.last().unwrap();

        if last.mnemonic == "RET" {
            return true;
        }

        if last.mnemonic == "B" {
            if instructions.len() >= 2 {
                let prev = &instructions[instructions.len() - 2];
                if prev.mnemonic == "LDP" {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_tail_call(instr: &Arm64Instruction) -> bool {
        instr.mnemonic == "B" && !instr.mnemonic.contains('.')
    }

    pub fn is_indirect_call(instr: &Arm64Instruction) -> bool {
        instr.mnemonic == "BLR"
    }

    pub fn is_direct_call(instr: &Arm64Instruction) -> bool {
        instr.mnemonic == "BL"
    }

    pub fn get_call_target(instr: &Arm64Instruction) -> Option<Address> {
        if !Self::is_direct_call(instr) {
            return None;
        }

        for operand in &instr.operands {
            if let Arm64Operand::Address(addr) = operand {
                return Some(*addr);
            }
        }

        None
    }

    pub fn is_adrp_sequence(instructions: &[Arm64Instruction]) -> Option<(Arm64Register, u64)> {
        if instructions.len() < 2 {
            return None;
        }

        let first = &instructions[0];
        let second = &instructions[1];

        if first.mnemonic != "ADRP" {
            return None;
        }

        let adrp_reg = match first.operands.get(0) {
            Some(Arm64Operand::Register(r)) => r.clone(),
            _ => return None,
        };

        let adrp_page = match first.operands.get(1) {
            Some(Arm64Operand::Immediate(imm)) => *imm as u64,
            _ => return None,
        };

        let page_base = (first.address.as_u64() & !0xFFF) + (adrp_page << 12);

        if second.mnemonic == "ADD" || second.mnemonic == "LDR" {
            if let Some(Arm64Operand::Register(r)) = second.operands.get(1) {
                if *r == adrp_reg {
                    let offset = match second.operands.get(2) {
                        Some(Arm64Operand::Immediate(imm)) => *imm as u64,
                        Some(Arm64Operand::Memory { offset, .. }) => *offset as u64,
                        _ => 0,
                    };
                    return Some((adrp_reg, page_base + offset));
                }
            }
        }

        None
    }

    pub fn is_stack_adjustment(instr: &Arm64Instruction) -> Option<i64> {
        if instr.mnemonic != "ADD" && instr.mnemonic != "SUB" {
            return None;
        }

        let is_sp_dst = matches!(instr.operands.get(0), Some(Arm64Operand::Register(Arm64Register::Sp)));
        let is_sp_src = matches!(instr.operands.get(1), Some(Arm64Operand::Register(Arm64Register::Sp)));

        if !is_sp_dst || !is_sp_src {
            return None;
        }

        let imm = match instr.operands.get(2) {
            Some(Arm64Operand::Immediate(i)) => *i,
            _ => return None,
        };

        if instr.mnemonic == "SUB" {
            Some(-imm)
        } else {
            Some(imm)
        }
    }

    pub fn is_frame_setup(instr: &Arm64Instruction) -> bool {
        if instr.mnemonic != "ADD" {
            return false;
        }

        let is_fp_dst = matches!(instr.operands.get(0), Some(Arm64Operand::Register(Arm64Register::X(29))));
        let is_sp_src = matches!(instr.operands.get(1), Some(Arm64Operand::Register(Arm64Register::Sp)));

        is_fp_dst && is_sp_src
    }

    pub fn extract_string_reference(instructions: &[Arm64Instruction]) -> Option<Address> {
        Self::is_adrp_sequence(instructions).map(|(_, addr)| Address::new(addr))
    }

    pub fn is_switch_table_load(instr: &Arm64Instruction) -> bool {
        if instr.mnemonic != "LDR" {
            return false;
        }

        if let Some(Arm64Operand::Memory { index: Some(_), .. }) = instr.operands.get(1) {
            return true;
        }

        false
    }

    pub fn is_computed_jump(instr: &Arm64Instruction) -> bool {
        instr.mnemonic == "BR"
    }

    pub fn is_conditional_select(instr: &Arm64Instruction) -> bool {
        matches!(instr.mnemonic.as_str(), "CSEL" | "CSINC" | "CSINV" | "CSNEG" | "CSET" | "CSETM")
    }

    pub fn get_memory_access_size(instr: &Arm64Instruction) -> Option<usize> {
        match instr.mnemonic.as_str() {
            "LDRB" | "STRB" | "LDRSB" => Some(1),
            "LDRH" | "STRH" | "LDRSH" => Some(2),
            "LDR" | "STR" if instr.mnemonic.contains('W') => Some(4),
            "LDR" | "STR" => Some(8),
            "LDP" | "STP" => Some(16),
            "LDUR" | "STUR" => Some(8),
            _ => None,
        }
    }
}

pub struct PatternMatcher {
    patterns: Vec<InstructionPattern>,
}

#[derive(Debug, Clone)]
pub struct InstructionPattern {
    pub name: String,
    pub mnemonics: Vec<String>,
    pub constraints: Vec<PatternConstraint>,
}

#[derive(Debug, Clone)]
pub enum PatternConstraint {
    RegisterMatch(usize, usize),
    ImmediateRange(usize, i64, i64),
    MemoryBase(usize, Arm64Register),
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    pub fn add_pattern(&mut self, pattern: InstructionPattern) {
        self.patterns.push(pattern);
    }

    pub fn find_matches(&self, instructions: &[Arm64Instruction]) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        for (i, _) in instructions.iter().enumerate() {
            for pattern in &self.patterns {
                if let Some(m) = self.try_match(instructions, i, pattern) {
                    matches.push(m);
                }
            }
        }

        matches
    }

    fn try_match(&self, instructions: &[Arm64Instruction], start: usize, pattern: &InstructionPattern) -> Option<PatternMatch> {
        if start + pattern.mnemonics.len() > instructions.len() {
            return None;
        }

        for (i, mnemonic) in pattern.mnemonics.iter().enumerate() {
            if instructions[start + i].mnemonic != *mnemonic {
                return None;
            }
        }

        Some(PatternMatch {
            pattern_name: pattern.name.clone(),
            start_address: instructions[start].address,
            instruction_count: pattern.mnemonics.len(),
        })
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_name: String,
    pub start_address: Address,
    pub instruction_count: usize,
}
