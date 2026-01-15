// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::analysis::arm64::{Arm64Instruction, Opcode, OperandType};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Instruction {
    address: Address,
    raw: u32,
    mnemonic: String,
    operands_str: String,
    opcode: InstructionOpcode,
    size: u8,
    is_branch: bool,
    is_call: bool,
    is_return: bool,
    is_conditional: bool,
    branch_target: Option<Address>,
    source_regs: Vec<u8>,
    dest_reg: Option<u8>,
    memory_operand: Option<MemoryOperand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionOpcode {
    Unknown,
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    Not,
    Shift,
    Move,
    Load,
    Store,
    Compare,
    Branch,
    Call,
    Return,
    Push,
    Pop,
    Nop,
    System,
    Float,
    Simd,
}

#[derive(Debug, Clone)]
pub struct MemoryOperand {
    pub base: Option<u8>,
    pub index: Option<u8>,
    pub offset: i64,
    pub scale: u8,
    pub size: u8,
}

impl Instruction {
    pub fn new(address: Address, raw: u32, mnemonic: &str) -> Self {
        Self {
            address,
            raw,
            mnemonic: mnemonic.to_string(),
            operands_str: String::new(),
            opcode: InstructionOpcode::Unknown,
            size: 4,
            is_branch: false,
            is_call: false,
            is_return: false,
            is_conditional: false,
            branch_target: None,
            source_regs: Vec::new(),
            dest_reg: None,
            memory_operand: None,
        }
    }

    pub fn from_arm64(insn: &Arm64Instruction) -> Self {
        let mut inst = Self::new(insn.address, insn.raw, insn.mnemonic());

        inst.is_branch = insn.is_branch();
        inst.is_call = insn.is_call();
        inst.is_return = insn.is_return();
        inst.is_conditional = insn.is_conditional_branch();
        inst.branch_target = insn.branch_target();

        inst.opcode = match insn.opcode {
            Opcode::ADD | Opcode::ADDS | Opcode::ADC | Opcode::ADCS => InstructionOpcode::Add,
            Opcode::SUB | Opcode::SUBS | Opcode::SBC | Opcode::SBCS | Opcode::NEG | Opcode::NEGS => InstructionOpcode::Sub,
            Opcode::MUL | Opcode::MADD | Opcode::MSUB | Opcode::SMULL | Opcode::UMULL => InstructionOpcode::Mul,
            Opcode::SDIV | Opcode::UDIV => InstructionOpcode::Div,
            Opcode::AND | Opcode::ANDS | Opcode::BIC | Opcode::BICS => InstructionOpcode::And,
            Opcode::ORR | Opcode::ORN => InstructionOpcode::Or,
            Opcode::EOR | Opcode::EON => InstructionOpcode::Xor,
            Opcode::MVN => InstructionOpcode::Not,
            Opcode::LSL | Opcode::LSR | Opcode::ASR | Opcode::ROR => InstructionOpcode::Shift,
            Opcode::MOV | Opcode::MOVZ | Opcode::MOVN | Opcode::MOVK => InstructionOpcode::Move,
            Opcode::LDR | Opcode::LDRB | Opcode::LDRH | Opcode::LDRSB | Opcode::LDRSH |
            Opcode::LDRSW | Opcode::LDP | Opcode::LDXR | Opcode::LDAR | Opcode::LDAXR => InstructionOpcode::Load,
            Opcode::STR | Opcode::STRB | Opcode::STRH | Opcode::STP |
            Opcode::STXR | Opcode::STLR | Opcode::STLXR => InstructionOpcode::Store,
            Opcode::CMP | Opcode::CMN | Opcode::TST | Opcode::CCMP | Opcode::CCMN => InstructionOpcode::Compare,
            Opcode::B | Opcode::Bcc | Opcode::BR | Opcode::CBZ | Opcode::CBNZ |
            Opcode::TBZ | Opcode::TBNZ => InstructionOpcode::Branch,
            Opcode::BL | Opcode::BLR => InstructionOpcode::Call,
            Opcode::RET => InstructionOpcode::Return,
            Opcode::NOP => InstructionOpcode::Nop,
            Opcode::SVC | Opcode::BRK | Opcode::HLT | Opcode::MSR | Opcode::MRS => InstructionOpcode::System,
            Opcode::SIMD(_) => InstructionOpcode::Simd,
            Opcode::FMOV | Opcode::FADD | Opcode::FSUB | Opcode::FMUL | Opcode::FDIV |
            Opcode::FCMP | Opcode::FCVT | Opcode::FCVTZS | Opcode::FCVTZU |
            Opcode::SCVTF | Opcode::UCVTF => InstructionOpcode::Float,
            _ => InstructionOpcode::Unknown,
        };

        for operand in &insn.operands {
            match &operand.op_type {
                OperandType::Register(reg) => {
                    inst.source_regs.push(reg.index());
                }
                OperandType::Memory { base, index, offset, scale, .. } => {
                    inst.memory_operand = Some(MemoryOperand {
                        base: base.map(|r| r.index()),
                        index: index.map(|r| r.index()),
                        offset: *offset,
                        scale: *scale,
                        size: operand.size,
                    });
                    if let Some(b) = base {
                        inst.source_regs.push(b.index());
                    }
                    if let Some(i) = index {
                        inst.source_regs.push(i.index());
                    }
                }
                _ => {}
            }
        }

        if let Some(dest) = insn.get_destination_register() {
            inst.dest_reg = Some(dest.index());
        }

        let mut operands = Vec::new();
        for op in &insn.operands {
            operands.push(format!("{}", op));
        }
        inst.operands_str = operands.join(", ");

        inst
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn raw(&self) -> u32 {
        self.raw
    }

    pub fn mnemonic(&self) -> &str {
        &self.mnemonic
    }

    pub fn operands_str(&self) -> &str {
        &self.operands_str
    }

    pub fn opcode(&self) -> InstructionOpcode {
        self.opcode
    }

    pub fn size(&self) -> u8 {
        self.size
    }

    pub fn is_branch(&self) -> bool {
        self.is_branch
    }

    pub fn is_call(&self) -> bool {
        self.is_call
    }

    pub fn is_return(&self) -> bool {
        self.is_return
    }

    pub fn is_conditional_branch(&self) -> bool {
        self.is_conditional
    }

    pub fn is_unconditional_branch(&self) -> bool {
        self.is_branch && !self.is_conditional && !self.is_call
    }

    pub fn branch_target(&self) -> Option<Address> {
        self.branch_target
    }

    pub fn branch_targets(&self) -> Vec<Address> {
        self.branch_target.into_iter().collect()
    }

    pub fn source_registers(&self) -> &[u8] {
        &self.source_regs
    }

    pub fn destination_register(&self) -> Option<u8> {
        self.dest_reg
    }

    pub fn memory_operand(&self) -> Option<&MemoryOperand> {
        self.memory_operand.as_ref()
    }

    pub fn is_memory_access(&self) -> bool {
        self.memory_operand.is_some()
    }

    pub fn is_load(&self) -> bool {
        self.opcode == InstructionOpcode::Load
    }

    pub fn is_store(&self) -> bool {
        self.opcode == InstructionOpcode::Store
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(self.opcode, InstructionOpcode::Add | InstructionOpcode::Sub |
            InstructionOpcode::Mul | InstructionOpcode::Div)
    }

    pub fn is_logical(&self) -> bool {
        matches!(self.opcode, InstructionOpcode::And | InstructionOpcode::Or |
            InstructionOpcode::Xor | InstructionOpcode::Not)
    }

    pub fn is_compare(&self) -> bool {
        self.opcode == InstructionOpcode::Compare
    }

    pub fn is_move(&self) -> bool {
        self.opcode == InstructionOpcode::Move
    }

    pub fn is_nop(&self) -> bool {
        self.opcode == InstructionOpcode::Nop
    }

    pub fn is_system(&self) -> bool {
        self.opcode == InstructionOpcode::System
    }

    pub fn is_float(&self) -> bool {
        self.opcode == InstructionOpcode::Float
    }

    pub fn is_simd(&self) -> bool {
        self.opcode == InstructionOpcode::Simd
    }

    pub fn uses_register(&self, reg: u8) -> bool {
        self.source_regs.contains(&reg)
    }

    pub fn defines_register(&self, reg: u8) -> bool {
        self.dest_reg == Some(reg)
    }

    pub fn reads_memory(&self) -> bool {
        self.is_load()
    }

    pub fn writes_memory(&self) -> bool {
        self.is_store()
    }

    pub fn has_side_effects(&self) -> bool {
        self.is_call() || self.is_system() || self.writes_memory()
    }

    pub fn next_address(&self) -> Address {
        self.address + self.size as u64
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}: {:08x}  {} {}",
            self.address.as_u64(),
            self.raw,
            self.mnemonic,
            self.operands_str)
    }
}

impl MemoryOperand {
    pub fn effective_address(&self, base_value: u64, index_value: u64) -> u64 {
        let mut addr = 0u64;
        if self.base.is_some() {
            addr = base_value;
        }
        if self.index.is_some() {
            addr = addr.wrapping_add(index_value.wrapping_mul(self.scale as u64));
        }
        addr.wrapping_add(self.offset as u64)
    }

    pub fn is_pc_relative(&self) -> bool {
        self.base == Some(32)
    }

    pub fn is_stack_access(&self) -> bool {
        self.base == Some(31)
    }

    pub fn access_size(&self) -> u8 {
        self.size
    }
}

pub fn categorize_instruction(mnemonic: &str) -> InstructionOpcode {
    let m = mnemonic.to_lowercase();

    if m.starts_with("add") || m.starts_with("adc") { return InstructionOpcode::Add; }
    if m.starts_with("sub") || m.starts_with("sbc") || m.starts_with("neg") { return InstructionOpcode::Sub; }
    if m.starts_with("mul") || m.starts_with("madd") || m.starts_with("msub") { return InstructionOpcode::Mul; }
    if m.starts_with("sdiv") || m.starts_with("udiv") { return InstructionOpcode::Div; }
    if m.starts_with("and") || m.starts_with("bic") { return InstructionOpcode::And; }
    if m.starts_with("orr") || m.starts_with("orn") { return InstructionOpcode::Or; }
    if m.starts_with("eor") || m.starts_with("eon") { return InstructionOpcode::Xor; }
    if m.starts_with("mvn") { return InstructionOpcode::Not; }
    if m.starts_with("lsl") || m.starts_with("lsr") || m.starts_with("asr") || m.starts_with("ror") {
        return InstructionOpcode::Shift;
    }
    if m.starts_with("mov") { return InstructionOpcode::Move; }
    if m.starts_with("ldr") || m.starts_with("ldp") || m.starts_with("ldx") || m.starts_with("lda") {
        return InstructionOpcode::Load;
    }
    if m.starts_with("str") || m.starts_with("stp") || m.starts_with("stx") || m.starts_with("stl") {
        return InstructionOpcode::Store;
    }
    if m.starts_with("cmp") || m.starts_with("cmn") || m.starts_with("tst") || m.starts_with("ccm") {
        return InstructionOpcode::Compare;
    }
    if m == "b" || m.starts_with("b.") || m.starts_with("cbz") || m.starts_with("cbnz") ||
       m.starts_with("tbz") || m.starts_with("tbnz") || m == "br" {
        return InstructionOpcode::Branch;
    }
    if m == "bl" || m == "blr" { return InstructionOpcode::Call; }
    if m == "ret" { return InstructionOpcode::Return; }
    if m == "nop" { return InstructionOpcode::Nop; }
    if m.starts_with("svc") || m.starts_with("brk") || m.starts_with("msr") || m.starts_with("mrs") {
        return InstructionOpcode::System;
    }
    if m.starts_with("f") { return InstructionOpcode::Float; }

    InstructionOpcode::Unknown
}
