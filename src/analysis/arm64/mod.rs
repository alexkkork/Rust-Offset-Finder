// Tue Jan 13 2026 - Alex

pub mod decoder;
pub mod operand;
pub mod register;
pub mod condition;
pub mod encoding;
pub mod opcodes;

pub use decoder::Arm64Decoder;
pub use operand::{Operand, OperandType, ShiftType, ExtendType};
pub use register::{Register, RegisterBank, RegisterSize};
pub use condition::Condition;
pub use encoding::{InstructionEncoding, EncodingClass};
pub use opcodes::{Opcode, OpcodeClass};

use crate::memory::Address;

#[derive(Debug, Clone)]
pub struct Arm64Instruction {
    pub address: Address,
    pub raw: u32,
    pub opcode: Opcode,
    pub operands: Vec<Operand>,
    pub size: u8,
    pub condition: Option<Condition>,
}

impl Arm64Instruction {
    pub fn new(address: Address, raw: u32) -> Self {
        Self {
            address,
            raw,
            opcode: Opcode::Unknown,
            operands: Vec::new(),
            size: 4,
            condition: None,
        }
    }

    pub fn with_opcode(mut self, opcode: Opcode) -> Self {
        self.opcode = opcode;
        self
    }

    pub fn with_operands(mut self, operands: Vec<Operand>) -> Self {
        self.operands = operands;
        self
    }

    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn is_branch(&self) -> bool {
        matches!(
            self.opcode,
            Opcode::B | Opcode::BL | Opcode::BR | Opcode::BLR | Opcode::RET |
            Opcode::CBZ | Opcode::CBNZ | Opcode::TBZ | Opcode::TBNZ |
            Opcode::Bcc
        )
    }

    pub fn is_unconditional_branch(&self) -> bool {
        matches!(self.opcode, Opcode::B | Opcode::BR | Opcode::RET)
    }

    pub fn is_conditional_branch(&self) -> bool {
        matches!(
            self.opcode,
            Opcode::Bcc | Opcode::CBZ | Opcode::CBNZ | Opcode::TBZ | Opcode::TBNZ
        )
    }

    pub fn is_call(&self) -> bool {
        matches!(self.opcode, Opcode::BL | Opcode::BLR)
    }

    pub fn is_return(&self) -> bool {
        matches!(self.opcode, Opcode::RET)
    }

    pub fn is_load(&self) -> bool {
        matches!(
            self.opcode,
            Opcode::LDR | Opcode::LDRB | Opcode::LDRH | Opcode::LDRSB |
            Opcode::LDRSH | Opcode::LDRSW | Opcode::LDP | Opcode::LDXR |
            Opcode::LDAR | Opcode::LDAXR
        )
    }

    pub fn is_store(&self) -> bool {
        matches!(
            self.opcode,
            Opcode::STR | Opcode::STRB | Opcode::STRH | Opcode::STP |
            Opcode::STXR | Opcode::STLR | Opcode::STLXR
        )
    }

    pub fn is_memory_access(&self) -> bool {
        self.is_load() || self.is_store()
    }

    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self.opcode,
            Opcode::ADD | Opcode::ADDS | Opcode::SUB | Opcode::SUBS |
            Opcode::MUL | Opcode::SDIV | Opcode::UDIV | Opcode::NEG |
            Opcode::ADC | Opcode::SBC | Opcode::MADD | Opcode::MSUB
        )
    }

    pub fn is_logical(&self) -> bool {
        matches!(
            self.opcode,
            Opcode::AND | Opcode::ANDS | Opcode::ORR | Opcode::EOR |
            Opcode::BIC | Opcode::ORN | Opcode::EON | Opcode::MVN
        )
    }

    pub fn is_compare(&self) -> bool {
        matches!(self.opcode, Opcode::CMP | Opcode::CMN | Opcode::TST)
    }

    pub fn is_move(&self) -> bool {
        matches!(
            self.opcode,
            Opcode::MOV | Opcode::MOVZ | Opcode::MOVN | Opcode::MOVK
        )
    }

    pub fn is_shift(&self) -> bool {
        matches!(
            self.opcode,
            Opcode::LSL | Opcode::LSR | Opcode::ASR | Opcode::ROR
        )
    }

    pub fn is_simd(&self) -> bool {
        matches!(self.opcode, Opcode::SIMD(_))
    }

    pub fn is_system(&self) -> bool {
        matches!(
            self.opcode,
            Opcode::SVC | Opcode::HVC | Opcode::SMC | Opcode::BRK |
            Opcode::HLT | Opcode::NOP | Opcode::MSR | Opcode::MRS
        )
    }

    pub fn branch_target(&self) -> Option<Address> {
        if !self.is_branch() {
            return None;
        }

        for operand in &self.operands {
            if let OperandType::Immediate(imm) = operand.op_type {
                return Some(Address::new(imm as u64));
            }
            if let OperandType::PCRelative(offset) = operand.op_type {
                return Some(self.address + offset as u64);
            }
        }
        None
    }

    pub fn uses_register(&self, reg: Register) -> bool {
        for operand in &self.operands {
            if let OperandType::Register(r) = operand.op_type {
                if r == reg {
                    return true;
                }
            }
            if let OperandType::Memory { base, index, .. } = operand.op_type {
                if base == Some(reg) || index == Some(reg) {
                    return true;
                }
            }
        }
        false
    }

    pub fn defines_register(&self, reg: Register) -> bool {
        if let Some(first) = self.operands.first() {
            if let OperandType::Register(r) = first.op_type {
                return r == reg;
            }
        }
        false
    }

    pub fn get_source_registers(&self) -> Vec<Register> {
        let mut regs = Vec::new();
        for (i, operand) in self.operands.iter().enumerate() {
            if i == 0 && !self.is_compare() && !self.is_store() {
                continue;
            }
            if let OperandType::Register(r) = operand.op_type {
                regs.push(r);
            }
            if let OperandType::Memory { base, index, .. } = operand.op_type {
                if let Some(b) = base {
                    regs.push(b);
                }
                if let Some(idx) = index {
                    regs.push(idx);
                }
            }
        }
        regs
    }

    pub fn get_destination_register(&self) -> Option<Register> {
        if self.is_compare() || self.is_store() {
            return None;
        }
        if let Some(first) = self.operands.first() {
            if let OperandType::Register(r) = first.op_type {
                return Some(r);
            }
        }
        None
    }

    pub fn mnemonic(&self) -> &'static str {
        self.opcode.mnemonic()
    }
}

impl std::fmt::Display for Arm64Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}: {:08x}  {}", self.address.as_u64(), self.raw, self.mnemonic())?;
        for (i, op) in self.operands.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, " {}", op)?;
        }
        Ok(())
    }
}

pub fn decode_instruction(address: Address, raw: u32) -> Arm64Instruction {
    let decoder = Arm64Decoder::new();
    decoder.decode(address, raw)
}

pub fn is_valid_instruction(raw: u32) -> bool {
    let decoder = Arm64Decoder::new();
    decoder.is_valid(raw)
}
