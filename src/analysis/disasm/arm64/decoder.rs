// Wed Jan 15 2026 - Alex

use super::{Arm64Instruction, Arm64Operand, Arm64Register, Arm64Condition, Arm64Shift, Arm64Extend};
use crate::memory::Address;

pub struct Arm64Decoder;

impl Arm64Decoder {
    pub fn decode(raw: u32, addr: Address) -> Arm64Instruction {
        let op0 = (raw >> 25) & 0xF;

        match op0 {
            0b0000 | 0b0001 | 0b0010 | 0b0011 => Self::decode_unallocated(raw, addr),
            0b1000 | 0b1001 => Self::decode_data_processing_imm(raw, addr),
            0b1010 | 0b1011 => Self::decode_branch(raw, addr),
            0b0100 | 0b0110 | 0b1100 | 0b1110 => Self::decode_load_store(raw, addr),
            0b0101 | 0b1101 => Self::decode_data_processing_reg(raw, addr),
            0b0111 | 0b1111 => Self::decode_simd_fp(raw, addr),
            _ => Self::decode_unknown(raw, addr),
        }
    }

    fn decode_unallocated(raw: u32, addr: Address) -> Arm64Instruction {
        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: "UDF".to_string(),
            operands: vec![Arm64Operand::Immediate((raw & 0xFFFF) as i64)],
            writes_flags: false,
            reads_flags: false,
        }
    }

    fn decode_data_processing_imm(raw: u32, addr: Address) -> Arm64Instruction {
        let op0 = (raw >> 23) & 0x7;

        match op0 {
            0b000 | 0b001 => Self::decode_add_sub_imm(raw, addr),
            0b010 => Self::decode_logical_imm(raw, addr),
            0b011 => Self::decode_move_wide(raw, addr),
            0b100 => Self::decode_bitfield(raw, addr),
            0b101 => Self::decode_extract(raw, addr),
            _ => Self::decode_unknown(raw, addr),
        }
    }

    fn decode_add_sub_imm(raw: u32, addr: Address) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let op = (raw >> 30) & 1;
        let s = (raw >> 29) & 1;
        let sh = (raw >> 22) & 1;
        let imm12 = ((raw >> 10) & 0xFFF) as i64;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        let imm = if sh == 1 { imm12 << 12 } else { imm12 };
        let is_64bit = sf == 1;

        let (mnemonic, writes_flags) = match (op, s) {
            (0, 0) => ("ADD", false),
            (0, 1) => ("ADDS", true),
            (1, 0) => ("SUB", false),
            (1, 1) => ("SUBS", true),
        };

        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Arm64Operand::Register(Arm64Register::from_encoding(rd, is_64bit)),
                Arm64Operand::Register(Arm64Register::from_encoding(rn, is_64bit)),
                Arm64Operand::Immediate(imm),
            ],
            writes_flags,
            reads_flags: false,
        }
    }

    fn decode_logical_imm(raw: u32, addr: Address) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let opc = (raw >> 29) & 0x3;
        let _n = (raw >> 22) & 1;
        let _immr = ((raw >> 16) & 0x3F) as u8;
        let _imms = ((raw >> 10) & 0x3F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        let is_64bit = sf == 1;

        let (mnemonic, writes_flags) = match opc {
            0b00 => ("AND", false),
            0b01 => ("ORR", false),
            0b10 => ("EOR", false),
            0b11 => ("ANDS", true),
            _ => unreachable!(),
        };

        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Arm64Operand::Register(Arm64Register::from_encoding(rd, is_64bit)),
                Arm64Operand::Register(Arm64Register::from_encoding(rn, is_64bit)),
                Arm64Operand::Immediate(0),
            ],
            writes_flags,
            reads_flags: false,
        }
    }

    fn decode_move_wide(raw: u32, addr: Address) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let opc = (raw >> 29) & 0x3;
        let hw = ((raw >> 21) & 0x3) as u8;
        let imm16 = ((raw >> 5) & 0xFFFF) as i64;
        let rd = (raw & 0x1F) as u8;

        let is_64bit = sf == 1;
        let shift = (hw as i64) * 16;

        let mnemonic = match opc {
            0b00 => "MOVN",
            0b10 => "MOVZ",
            0b11 => "MOVK",
            _ => "UNKNOWN",
        };

        let mut operands = vec![
            Arm64Operand::Register(Arm64Register::from_encoding(rd, is_64bit)),
            Arm64Operand::Immediate(imm16),
        ];

        if shift != 0 {
            operands.push(Arm64Operand::ShiftedReg {
                reg: Arm64Register::from_encoding(rd, is_64bit),
                shift: Arm64Shift::Lsl,
                amount: shift as u8,
            });
        }

        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: mnemonic.to_string(),
            operands,
            writes_flags: false,
            reads_flags: false,
        }
    }

    fn decode_bitfield(raw: u32, addr: Address) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let opc = (raw >> 29) & 0x3;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;
        let is_64bit = sf == 1;

        let mnemonic = match opc {
            0b00 => "SBFM",
            0b01 => "BFM",
            0b10 => "UBFM",
            _ => "UNKNOWN",
        };

        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Arm64Operand::Register(Arm64Register::from_encoding(rd, is_64bit)),
                Arm64Operand::Register(Arm64Register::from_encoding(rn, is_64bit)),
            ],
            writes_flags: false,
            reads_flags: false,
        }
    }

    fn decode_extract(raw: u32, addr: Address) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let rm = ((raw >> 16) & 0x1F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;
        let is_64bit = sf == 1;

        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: "EXTR".to_string(),
            operands: vec![
                Arm64Operand::Register(Arm64Register::from_encoding(rd, is_64bit)),
                Arm64Operand::Register(Arm64Register::from_encoding(rn, is_64bit)),
                Arm64Operand::Register(Arm64Register::from_encoding(rm, is_64bit)),
            ],
            writes_flags: false,
            reads_flags: false,
        }
    }

    fn decode_branch(raw: u32, addr: Address) -> Arm64Instruction {
        let op0 = (raw >> 29) & 0x7;

        match op0 {
            0b000 | 0b100 => Self::decode_unconditional_branch(raw, addr),
            0b001 | 0b101 => Self::decode_compare_branch(raw, addr),
            0b010 | 0b110 => Self::decode_conditional_branch(raw, addr),
            0b011 | 0b111 => Self::decode_test_branch(raw, addr),
            _ => Self::decode_unknown(raw, addr),
        }
    }

    fn decode_unconditional_branch(raw: u32, addr: Address) -> Arm64Instruction {
        let op = (raw >> 31) & 1;
        let imm26 = raw & 0x03FFFFFF;

        let offset = if imm26 & 0x02000000 != 0 {
            ((imm26 | 0xFC000000) as i32) * 4
        } else {
            (imm26 as i32) * 4
        };

        let target = (addr.as_u64() as i64 + offset as i64) as u64;

        let mnemonic = if op == 1 { "BL" } else { "B" };

        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: mnemonic.to_string(),
            operands: vec![Arm64Operand::Address(Address::new(target))],
            writes_flags: false,
            reads_flags: false,
        }
    }

    fn decode_compare_branch(raw: u32, addr: Address) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let op = (raw >> 24) & 1;
        let imm19 = (raw >> 5) & 0x7FFFF;
        let rt = (raw & 0x1F) as u8;

        let offset = if imm19 & 0x40000 != 0 {
            ((imm19 | 0xFFF80000) as i32) * 4
        } else {
            (imm19 as i32) * 4
        };

        let target = (addr.as_u64() as i64 + offset as i64) as u64;
        let is_64bit = sf == 1;

        let mnemonic = if op == 0 { "CBZ" } else { "CBNZ" };

        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Arm64Operand::Register(Arm64Register::from_encoding(rt, is_64bit)),
                Arm64Operand::Address(Address::new(target)),
            ],
            writes_flags: false,
            reads_flags: false,
        }
    }

    fn decode_conditional_branch(raw: u32, addr: Address) -> Arm64Instruction {
        let imm19 = (raw >> 5) & 0x7FFFF;
        let cond = (raw & 0xF) as u8;

        let offset = if imm19 & 0x40000 != 0 {
            ((imm19 | 0xFFF80000) as i32) * 4
        } else {
            (imm19 as i32) * 4
        };

        let target = (addr.as_u64() as i64 + offset as i64) as u64;
        let condition = Arm64Condition::from_encoding(cond);

        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: format!("B.{}", condition.name()),
            operands: vec![
                Arm64Operand::Condition(condition),
                Arm64Operand::Address(Address::new(target)),
            ],
            writes_flags: false,
            reads_flags: true,
        }
    }

    fn decode_test_branch(raw: u32, addr: Address) -> Arm64Instruction {
        let b5 = (raw >> 31) & 1;
        let op = (raw >> 24) & 1;
        let b40 = ((raw >> 19) & 0x1F) as u8;
        let imm14 = (raw >> 5) & 0x3FFF;
        let rt = (raw & 0x1F) as u8;

        let bit_pos = (b5 << 5) as u8 | b40;
        let offset = if imm14 & 0x2000 != 0 {
            ((imm14 | 0xFFFFC000) as i32) * 4
        } else {
            (imm14 as i32) * 4
        };

        let target = (addr.as_u64() as i64 + offset as i64) as u64;
        let mnemonic = if op == 0 { "TBZ" } else { "TBNZ" };

        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Arm64Operand::Register(Arm64Register::X(rt)),
                Arm64Operand::Immediate(bit_pos as i64),
                Arm64Operand::Address(Address::new(target)),
            ],
            writes_flags: false,
            reads_flags: false,
        }
    }

    fn decode_load_store(raw: u32, addr: Address) -> Arm64Instruction {
        let size = (raw >> 30) & 0x3;
        let v = (raw >> 26) & 1;
        let opc = (raw >> 22) & 0x3;
        let rt = (raw & 0x1F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;

        let is_load = opc & 1 == 1;
        let is_64bit = size == 3;

        let mnemonic = match (v, is_load, size) {
            (0, false, _) => "STR",
            (0, true, _) => "LDR",
            (1, false, _) => "STR",
            (1, true, _) => "LDR",
        };

        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Arm64Operand::Register(Arm64Register::from_encoding(rt, is_64bit)),
                Arm64Operand::Memory {
                    base: Arm64Register::X(rn),
                    offset: 0,
                    pre_index: false,
                    post_index: false,
                },
            ],
            writes_flags: false,
            reads_flags: false,
        }
    }

    fn decode_data_processing_reg(raw: u32, addr: Address) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let opc = (raw >> 29) & 0x3;
        let rm = ((raw >> 16) & 0x1F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;
        let is_64bit = sf == 1;

        let mnemonic = match opc {
            0b00 => "AND",
            0b01 => "ORR",
            0b10 => "EOR",
            0b11 => "ANDS",
            _ => "UNKNOWN",
        };

        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Arm64Operand::Register(Arm64Register::from_encoding(rd, is_64bit)),
                Arm64Operand::Register(Arm64Register::from_encoding(rn, is_64bit)),
                Arm64Operand::Register(Arm64Register::from_encoding(rm, is_64bit)),
            ],
            writes_flags: opc == 0b11,
            reads_flags: false,
        }
    }

    fn decode_simd_fp(raw: u32, addr: Address) -> Arm64Instruction {
        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: "SIMD".to_string(),
            operands: vec![],
            writes_flags: false,
            reads_flags: false,
        }
    }

    fn decode_unknown(raw: u32, addr: Address) -> Arm64Instruction {
        Arm64Instruction {
            address: addr,
            raw,
            mnemonic: "UNKNOWN".to_string(),
            operands: vec![Arm64Operand::Immediate(raw as i64)],
            writes_flags: false,
            reads_flags: false,
        }
    }
}
