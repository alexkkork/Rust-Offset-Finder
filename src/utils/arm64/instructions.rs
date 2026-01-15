// Tue Jan 13 2026 - Alex

use super::{Register, Operand};

#[derive(Debug, Clone)]
pub struct InstructionInfo {
    pub mnemonic: String,
    pub operands: Vec<Operand>,
    pub size: usize,
    pub encoding: u32,
}

impl InstructionInfo {
    pub fn new(mnemonic: &str, encoding: u32) -> Self {
        Self {
            mnemonic: mnemonic.to_string(),
            operands: Vec::new(),
            size: 4,
            encoding,
        }
    }

    pub fn with_operands(mut self, operands: Vec<Operand>) -> Self {
        self.operands = operands;
        self
    }

    pub fn is_branch(&self) -> bool {
        matches!(self.mnemonic.as_str(),
            "b" | "bl" | "br" | "blr" | "ret" |
            "b.eq" | "b.ne" | "b.cs" | "b.cc" |
            "b.mi" | "b.pl" | "b.vs" | "b.vc" |
            "b.hi" | "b.ls" | "b.ge" | "b.lt" |
            "b.gt" | "b.le" | "b.al" |
            "cbz" | "cbnz" | "tbz" | "tbnz"
        )
    }

    pub fn is_call(&self) -> bool {
        matches!(self.mnemonic.as_str(), "bl" | "blr")
    }

    pub fn is_return(&self) -> bool {
        self.mnemonic == "ret"
    }

    pub fn is_conditional_branch(&self) -> bool {
        self.mnemonic.starts_with("b.") ||
        matches!(self.mnemonic.as_str(), "cbz" | "cbnz" | "tbz" | "tbnz")
    }

    pub fn is_unconditional_branch(&self) -> bool {
        matches!(self.mnemonic.as_str(), "b" | "bl" | "br" | "blr" | "ret")
    }

    pub fn is_load(&self) -> bool {
        self.mnemonic.starts_with("ldr") ||
        self.mnemonic.starts_with("ldp") ||
        self.mnemonic.starts_with("ldur") ||
        matches!(self.mnemonic.as_str(), "ldrb" | "ldrh" | "ldrsb" | "ldrsh" | "ldrsw")
    }

    pub fn is_store(&self) -> bool {
        self.mnemonic.starts_with("str") ||
        self.mnemonic.starts_with("stp") ||
        self.mnemonic.starts_with("stur") ||
        matches!(self.mnemonic.as_str(), "strb" | "strh")
    }

    pub fn is_memory_access(&self) -> bool {
        self.is_load() || self.is_store()
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(self.mnemonic.as_str(),
            "add" | "adds" | "sub" | "subs" |
            "adc" | "adcs" | "sbc" | "sbcs" |
            "neg" | "negs" | "ngc" | "ngcs" |
            "mul" | "mneg" | "smull" | "smulh" |
            "umull" | "umulh" | "madd" | "msub" |
            "smaddl" | "smsubl" | "umaddl" | "umsubl" |
            "sdiv" | "udiv"
        )
    }

    pub fn is_logical(&self) -> bool {
        matches!(self.mnemonic.as_str(),
            "and" | "ands" | "orr" | "eor" |
            "bic" | "bics" | "orn" | "eon" |
            "mvn" | "tst"
        )
    }

    pub fn is_compare(&self) -> bool {
        matches!(self.mnemonic.as_str(),
            "cmp" | "cmn" | "tst" | "ccmp" | "ccmn"
        )
    }

    pub fn is_move(&self) -> bool {
        matches!(self.mnemonic.as_str(),
            "mov" | "movz" | "movn" | "movk" |
            "mvn" | "adr" | "adrp"
        )
    }

    pub fn is_nop(&self) -> bool {
        self.mnemonic == "nop" || self.encoding == 0xD503201F
    }

    pub fn get_destination_register(&self) -> Option<&Register> {
        if self.is_store() || self.is_compare() || self.is_branch() {
            return None;
        }

        match self.operands.first() {
            Some(Operand::Register(reg)) => Some(reg),
            _ => None,
        }
    }

    pub fn get_source_registers(&self) -> Vec<&Register> {
        let skip = if self.is_store() || self.is_compare() { 0 } else { 1 };

        self.operands.iter().skip(skip).filter_map(|op| {
            match op {
                Operand::Register(reg) => Some(reg),
                Operand::Memory { .. } => None,
                _ => None,
            }
        }).collect()
    }

    pub fn get_immediate(&self) -> Option<i64> {
        for op in &self.operands {
            if let Operand::Immediate(imm) = op {
                return Some(*imm);
            }
        }
        None
    }

    pub fn get_memory_base(&self) -> Option<u8> {
        for op in &self.operands {
            if let Operand::Memory { base, .. } = op {
                return Some(*base);
            }
        }
        None
    }

    pub fn get_memory_offset(&self) -> Option<i64> {
        for op in &self.operands {
            if let Operand::Memory { offset, .. } = op {
                return Some(*offset);
            }
        }
        None
    }

    pub fn get_branch_target(&self, current_address: u64) -> Option<u64> {
        if !self.is_branch() {
            return None;
        }

        self.get_immediate().map(|offset| {
            ((current_address as i64) + offset) as u64
        })
    }

    pub fn disassemble(&self) -> String {
        let ops: Vec<String> = self.operands.iter().map(|op| op.to_string()).collect();

        if ops.is_empty() {
            self.mnemonic.clone()
        } else {
            format!("{} {}", self.mnemonic, ops.join(", "))
        }
    }
}

impl std::fmt::Display for InstructionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.disassemble())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionCategory {
    Branch,
    Load,
    Store,
    Arithmetic,
    Logical,
    Compare,
    Move,
    System,
    Simd,
    Unknown,
}

impl InstructionInfo {
    pub fn category(&self) -> InstructionCategory {
        if self.is_branch() {
            InstructionCategory::Branch
        } else if self.is_load() {
            InstructionCategory::Load
        } else if self.is_store() {
            InstructionCategory::Store
        } else if self.is_arithmetic() {
            InstructionCategory::Arithmetic
        } else if self.is_logical() {
            InstructionCategory::Logical
        } else if self.is_compare() {
            InstructionCategory::Compare
        } else if self.is_move() {
            InstructionCategory::Move
        } else {
            InstructionCategory::Unknown
        }
    }
}
