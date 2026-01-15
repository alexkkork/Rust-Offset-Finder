// Tue Jan 13 2026 - Alex

use super::{InstructionInfo, Register, Operand};

pub struct InstructionDecoder;

impl InstructionDecoder {
    pub fn decode(insn: u32) -> Option<InstructionInfo> {
        let op0 = (insn >> 25) & 0xF;

        match op0 {
            0b0000 | 0b0001 | 0b0010 | 0b0011 => Self::decode_unallocated(insn),
            0b1000 | 0b1001 => Self::decode_data_processing_imm(insn),
            0b1010 | 0b1011 => Self::decode_branch(insn),
            0b0100 | 0b0110 | 0b1100 | 0b1110 => Self::decode_load_store(insn),
            0b0101 | 0b1101 => Self::decode_data_processing_reg(insn),
            0b0111 | 0b1111 => Self::decode_data_processing_simd(insn),
            _ => None,
        }
    }

    fn decode_unallocated(_insn: u32) -> Option<InstructionInfo> {
        None
    }

    fn decode_data_processing_imm(insn: u32) -> Option<InstructionInfo> {
        let op0 = (insn >> 23) & 0x7;

        match op0 {
            0b000 | 0b001 => Self::decode_pc_rel(insn),
            0b010 | 0b011 => Self::decode_add_sub_imm(insn),
            0b100 => Self::decode_logical_imm(insn),
            0b101 => Self::decode_move_wide_imm(insn),
            0b110 => Self::decode_bitfield(insn),
            0b111 => Self::decode_extract(insn),
            _ => None,
        }
    }

    fn decode_pc_rel(insn: u32) -> Option<InstructionInfo> {
        let op = (insn >> 31) & 1;
        let rd = (insn & 0x1F) as u8;
        let immlo = (insn >> 29) & 0x3;
        let immhi = (insn >> 5) & 0x7FFFF;

        let imm = if op == 0 {
            ((immhi << 2) | immlo) as i32
        } else {
            (((immhi << 2) | immlo) << 12) as i32
        };

        let mnemonic = if op == 0 { "adr" } else { "adrp" };

        Some(InstructionInfo {
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Operand::Register(Register::new_gpr(rd, true)),
                Operand::Immediate(imm as i64),
            ],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_add_sub_imm(insn: u32) -> Option<InstructionInfo> {
        let sf = (insn >> 31) & 1;
        let op = (insn >> 30) & 1;
        let s = (insn >> 29) & 1;
        let sh = (insn >> 22) & 1;
        let imm12 = ((insn >> 10) & 0xFFF) as u64;
        let rn = ((insn >> 5) & 0x1F) as u8;
        let rd = (insn & 0x1F) as u8;

        let imm = if sh == 1 { imm12 << 12 } else { imm12 };
        let is_64bit = sf == 1;

        let mnemonic = match (op, s) {
            (0, 0) => "add",
            (0, 1) => "adds",
            (1, 0) => "sub",
            (1, 1) => "subs",
            _ => return None,
        };

        Some(InstructionInfo {
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Operand::Register(Register::new_gpr(rd, is_64bit)),
                Operand::Register(Register::new_gpr(rn, is_64bit)),
                Operand::Immediate(imm as i64),
            ],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_logical_imm(insn: u32) -> Option<InstructionInfo> {
        let sf = (insn >> 31) & 1;
        let opc = (insn >> 29) & 0x3;
        let rd = (insn & 0x1F) as u8;
        let rn = ((insn >> 5) & 0x1F) as u8;

        let is_64bit = sf == 1;

        let mnemonic = match opc {
            0b00 => "and",
            0b01 => "orr",
            0b10 => "eor",
            0b11 => "ands",
            _ => return None,
        };

        Some(InstructionInfo {
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Operand::Register(Register::new_gpr(rd, is_64bit)),
                Operand::Register(Register::new_gpr(rn, is_64bit)),
                Operand::Immediate(0),
            ],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_move_wide_imm(insn: u32) -> Option<InstructionInfo> {
        let sf = (insn >> 31) & 1;
        let opc = (insn >> 29) & 0x3;
        let hw = ((insn >> 21) & 0x3) as u8;
        let imm16 = ((insn >> 5) & 0xFFFF) as u64;
        let rd = (insn & 0x1F) as u8;

        let is_64bit = sf == 1;

        let mnemonic = match opc {
            0b00 => "movn",
            0b10 => "movz",
            0b11 => "movk",
            _ => return None,
        };

        Some(InstructionInfo {
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Operand::Register(Register::new_gpr(rd, is_64bit)),
                Operand::Immediate(imm16 as i64),
                Operand::Shift(hw * 16),
            ],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_bitfield(insn: u32) -> Option<InstructionInfo> {
        let sf = (insn >> 31) & 1;
        let opc = (insn >> 29) & 0x3;
        let rd = (insn & 0x1F) as u8;
        let rn = ((insn >> 5) & 0x1F) as u8;

        let is_64bit = sf == 1;

        let mnemonic = match opc {
            0b00 => "sbfm",
            0b01 => "bfm",
            0b10 => "ubfm",
            _ => return None,
        };

        Some(InstructionInfo {
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Operand::Register(Register::new_gpr(rd, is_64bit)),
                Operand::Register(Register::new_gpr(rn, is_64bit)),
            ],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_extract(insn: u32) -> Option<InstructionInfo> {
        let sf = (insn >> 31) & 1;
        let rd = (insn & 0x1F) as u8;
        let rn = ((insn >> 5) & 0x1F) as u8;
        let rm = ((insn >> 16) & 0x1F) as u8;

        let is_64bit = sf == 1;

        Some(InstructionInfo {
            mnemonic: "extr".to_string(),
            operands: vec![
                Operand::Register(Register::new_gpr(rd, is_64bit)),
                Operand::Register(Register::new_gpr(rn, is_64bit)),
                Operand::Register(Register::new_gpr(rm, is_64bit)),
            ],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_branch(insn: u32) -> Option<InstructionInfo> {
        let op0 = (insn >> 29) & 0x7;

        match op0 {
            0b000 | 0b100 => Self::decode_conditional_branch(insn),
            0b001 | 0b101 => Self::decode_compare_branch(insn),
            0b010 | 0b110 => Self::decode_test_branch(insn),
            _ => Self::decode_unconditional_branch(insn),
        }
    }

    fn decode_conditional_branch(insn: u32) -> Option<InstructionInfo> {
        let cond = (insn & 0xF) as u8;
        let imm19 = ((insn >> 5) & 0x7FFFF) as i32;
        let offset = ((imm19 << 13) >> 11) as i64;

        let cond_str = match cond {
            0 => "eq", 1 => "ne", 2 => "cs", 3 => "cc",
            4 => "mi", 5 => "pl", 6 => "vs", 7 => "vc",
            8 => "hi", 9 => "ls", 10 => "ge", 11 => "lt",
            12 => "gt", 13 => "le", 14 => "al", _ => "nv",
        };

        Some(InstructionInfo {
            mnemonic: format!("b.{}", cond_str),
            operands: vec![Operand::Immediate(offset)],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_compare_branch(insn: u32) -> Option<InstructionInfo> {
        let sf = (insn >> 31) & 1;
        let op = (insn >> 24) & 1;
        let rt = (insn & 0x1F) as u8;
        let imm19 = ((insn >> 5) & 0x7FFFF) as i32;
        let offset = ((imm19 << 13) >> 11) as i64;

        let is_64bit = sf == 1;
        let mnemonic = if op == 0 { "cbz" } else { "cbnz" };

        Some(InstructionInfo {
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Operand::Register(Register::new_gpr(rt, is_64bit)),
                Operand::Immediate(offset),
            ],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_test_branch(insn: u32) -> Option<InstructionInfo> {
        let op = (insn >> 24) & 1;
        let rt = (insn & 0x1F) as u8;
        let imm14 = ((insn >> 5) & 0x3FFF) as i32;
        let offset = ((imm14 << 18) >> 16) as i64;
        let bit = ((insn >> 19) & 0x1F) | ((insn >> 26) & 0x20);

        let mnemonic = if op == 0 { "tbz" } else { "tbnz" };

        Some(InstructionInfo {
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Operand::Register(Register::new_gpr(rt, true)),
                Operand::Immediate(bit as i64),
                Operand::Immediate(offset),
            ],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_unconditional_branch(insn: u32) -> Option<InstructionInfo> {
        let op = (insn >> 31) & 1;

        if (insn & 0xFC000000) == 0x14000000 || (insn & 0xFC000000) == 0x94000000 {
            let imm26 = (insn & 0x03FFFFFF) as i32;
            let offset = ((imm26 << 6) >> 4) as i64;
            let mnemonic = if op == 0 { "b" } else { "bl" };

            return Some(InstructionInfo {
                mnemonic: mnemonic.to_string(),
                operands: vec![Operand::Immediate(offset)],
                size: 4,
                encoding: insn,
            });
        }

        if (insn & 0xFFFFFC1F) == 0xD65F0000 {
            return Some(InstructionInfo {
                mnemonic: "ret".to_string(),
                operands: vec![],
                size: 4,
                encoding: insn,
            });
        }

        if (insn & 0xFFFFFC1F) == 0xD63F0000 {
            let rn = ((insn >> 5) & 0x1F) as u8;
            return Some(InstructionInfo {
                mnemonic: "blr".to_string(),
                operands: vec![Operand::Register(Register::new_gpr(rn, true))],
                size: 4,
                encoding: insn,
            });
        }

        if (insn & 0xFFFFFC1F) == 0xD61F0000 {
            let rn = ((insn >> 5) & 0x1F) as u8;
            return Some(InstructionInfo {
                mnemonic: "br".to_string(),
                operands: vec![Operand::Register(Register::new_gpr(rn, true))],
                size: 4,
                encoding: insn,
            });
        }

        None
    }

    fn decode_load_store(insn: u32) -> Option<InstructionInfo> {
        let op0 = (insn >> 28) & 0xF;

        match op0 {
            0b0100 | 0b1100 | 0b0110 | 0b1110 => Self::decode_load_store_reg(insn),
            _ => None,
        }
    }

    fn decode_load_store_reg(insn: u32) -> Option<InstructionInfo> {
        let size = (insn >> 30) & 0x3;
        let v = (insn >> 26) & 1;
        let opc = (insn >> 22) & 0x3;
        let rt = (insn & 0x1F) as u8;
        let rn = ((insn >> 5) & 0x1F) as u8;

        let is_load = (opc & 1) == 1;
        let is_64bit = size == 0b11;

        let mnemonic = if v == 1 {
            if is_load { "ldr" } else { "str" }
        } else {
            match (size, opc) {
                (0b00, 0b00) => "strb",
                (0b00, 0b01) => "ldrb",
                (0b01, 0b00) => "strh",
                (0b01, 0b01) => "ldrh",
                (0b10, 0b00) => "str",
                (0b10, 0b01) => "ldr",
                (0b11, 0b00) => "str",
                (0b11, 0b01) => "ldr",
                _ => "unknown",
            }
        };

        Some(InstructionInfo {
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Operand::Register(Register::new_gpr(rt, is_64bit)),
                Operand::Memory { base: rn, offset: 0 },
            ],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_data_processing_reg(insn: u32) -> Option<InstructionInfo> {
        let op0 = (insn >> 30) & 1;
        let op1 = (insn >> 28) & 1;
        let op2 = (insn >> 21) & 0xF;

        if op1 == 0 && op2 == 0 {
            return Self::decode_logical_reg(insn);
        }

        if op1 == 0 && (op2 >> 1) == 0b0100 {
            return Self::decode_add_sub_shift(insn);
        }

        None
    }

    fn decode_logical_reg(insn: u32) -> Option<InstructionInfo> {
        let sf = (insn >> 31) & 1;
        let opc = (insn >> 29) & 0x3;
        let n = (insn >> 21) & 1;
        let rd = (insn & 0x1F) as u8;
        let rn = ((insn >> 5) & 0x1F) as u8;
        let rm = ((insn >> 16) & 0x1F) as u8;

        let is_64bit = sf == 1;

        let mnemonic = match (opc, n) {
            (0b00, 0) => "and",
            (0b00, 1) => "bic",
            (0b01, 0) => "orr",
            (0b01, 1) => "orn",
            (0b10, 0) => "eor",
            (0b10, 1) => "eon",
            (0b11, 0) => "ands",
            (0b11, 1) => "bics",
            _ => return None,
        };

        Some(InstructionInfo {
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Operand::Register(Register::new_gpr(rd, is_64bit)),
                Operand::Register(Register::new_gpr(rn, is_64bit)),
                Operand::Register(Register::new_gpr(rm, is_64bit)),
            ],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_add_sub_shift(insn: u32) -> Option<InstructionInfo> {
        let sf = (insn >> 31) & 1;
        let op = (insn >> 30) & 1;
        let s = (insn >> 29) & 1;
        let rd = (insn & 0x1F) as u8;
        let rn = ((insn >> 5) & 0x1F) as u8;
        let rm = ((insn >> 16) & 0x1F) as u8;

        let is_64bit = sf == 1;

        let mnemonic = match (op, s) {
            (0, 0) => "add",
            (0, 1) => "adds",
            (1, 0) => "sub",
            (1, 1) => "subs",
            _ => return None,
        };

        Some(InstructionInfo {
            mnemonic: mnemonic.to_string(),
            operands: vec![
                Operand::Register(Register::new_gpr(rd, is_64bit)),
                Operand::Register(Register::new_gpr(rn, is_64bit)),
                Operand::Register(Register::new_gpr(rm, is_64bit)),
            ],
            size: 4,
            encoding: insn,
        })
    }

    fn decode_data_processing_simd(_insn: u32) -> Option<InstructionInfo> {
        None
    }
}

pub fn decode(insn: u32) -> Option<InstructionInfo> {
    InstructionDecoder::decode(insn)
}
