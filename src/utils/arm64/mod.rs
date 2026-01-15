// Tue Jan 13 2026 - Alex

pub mod decoder;
pub mod encoding;
pub mod registers;
pub mod instructions;
pub mod operands;

pub use decoder::InstructionDecoder;
pub use encoding::InstructionEncoder;
pub use registers::Register;
pub use instructions::InstructionInfo;
pub use operands::Operand;

pub struct Arm64Utils;

impl Arm64Utils {
    pub fn decode_instruction(bytes: &[u8]) -> Option<InstructionInfo> {
        if bytes.len() < 4 {
            return None;
        }

        let insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        InstructionDecoder::decode(insn)
    }

    pub fn is_branch(insn: u32) -> bool {
        let op0 = (insn >> 25) & 0xF;
        matches!(op0, 0b0101 | 0b0111)
    }

    pub fn is_call(insn: u32) -> bool {
        let op = (insn >> 26) & 0x3F;
        op == 0b100101
    }

    pub fn is_return(insn: u32) -> bool {
        (insn & 0xFFFFFC1F) == 0xD65F0000
    }

    pub fn is_nop(insn: u32) -> bool {
        insn == 0xD503201F
    }

    pub fn is_load(insn: u32) -> bool {
        let op0 = (insn >> 25) & 0xF;
        op0 == 0b1100 || op0 == 0b1101
    }

    pub fn is_store(insn: u32) -> bool {
        let op0 = (insn >> 25) & 0xF;
        op0 == 0b1100 || op0 == 0b1101
    }

    pub fn get_branch_target(insn: u32, address: u64) -> Option<u64> {
        if Self::is_branch(insn) {
            let imm26 = (insn & 0x03FFFFFF) as i32;
            let offset = ((imm26 << 6) >> 4) as i64;
            Some((address as i64 + offset) as u64)
        } else {
            None
        }
    }

    pub fn get_conditional_branch_target(insn: u32, address: u64) -> Option<u64> {
        if (insn >> 25) & 0x7F == 0b0101010 {
            let imm19 = ((insn >> 5) & 0x7FFFF) as i32;
            let offset = ((imm19 << 13) >> 11) as i64;
            Some((address as i64 + offset) as u64)
        } else {
            None
        }
    }

    pub fn get_adrp_value(insn: u32, address: u64) -> Option<u64> {
        if (insn & 0x9F000000) == 0x90000000 {
            let immlo = ((insn >> 29) & 0x3) as i64;
            let immhi = ((insn >> 5) & 0x7FFFF) as i64;
            let imm = ((immhi << 2) | immlo) << 12;
            let page_addr = address & !0xFFF;
            Some(((page_addr as i64) + imm) as u64)
        } else {
            None
        }
    }

    pub fn get_add_imm(insn: u32) -> Option<u64> {
        if (insn & 0x7F800000) == 0x11000000 {
            let imm12 = ((insn >> 10) & 0xFFF) as u64;
            let shift = ((insn >> 22) & 0x3) as u64;
            Some(imm12 << (shift * 12))
        } else {
            None
        }
    }

    pub fn get_ldr_str_offset(insn: u32) -> Option<i64> {
        let opc = (insn >> 22) & 0x3;
        let size = (insn >> 30) & 0x3;

        if (insn & 0x3B000000) == 0x39000000 {
            let imm12 = ((insn >> 10) & 0xFFF) as i64;
            let scale = 1 << size;
            Some(imm12 * scale)
        } else if (insn & 0x3B200000) == 0x38000000 {
            let imm9 = ((insn >> 12) & 0x1FF) as i32;
            let imm9 = (imm9 << 23) >> 23;
            Some(imm9 as i64)
        } else {
            None
        }
    }

    pub fn get_register_from_instruction(insn: u32, field: RegisterField) -> u8 {
        match field {
            RegisterField::Rd => (insn & 0x1F) as u8,
            RegisterField::Rn => ((insn >> 5) & 0x1F) as u8,
            RegisterField::Rm => ((insn >> 16) & 0x1F) as u8,
            RegisterField::Ra => ((insn >> 10) & 0x1F) as u8,
            RegisterField::Rt => (insn & 0x1F) as u8,
            RegisterField::Rt2 => ((insn >> 10) & 0x1F) as u8,
        }
    }

    pub fn make_b_instruction(target: u64, current: u64) -> u32 {
        let offset = ((target as i64) - (current as i64)) >> 2;
        let imm26 = (offset as u32) & 0x03FFFFFF;
        0x14000000 | imm26
    }

    pub fn make_bl_instruction(target: u64, current: u64) -> u32 {
        let offset = ((target as i64) - (current as i64)) >> 2;
        let imm26 = (offset as u32) & 0x03FFFFFF;
        0x94000000 | imm26
    }

    pub fn make_ret_instruction() -> u32 {
        0xD65F03C0
    }

    pub fn make_nop_instruction() -> u32 {
        0xD503201F
    }

    pub fn make_mov_imm(rd: u8, imm16: u16) -> u32 {
        let rd = (rd & 0x1F) as u32;
        0xD2800000 | ((imm16 as u32) << 5) | rd
    }

    pub fn disassemble(insn: u32) -> String {
        if Self::is_nop(insn) {
            return "nop".to_string();
        }

        if Self::is_return(insn) {
            return "ret".to_string();
        }

        if Self::is_call(insn) {
            let imm26 = (insn & 0x03FFFFFF) as i32;
            let offset = ((imm26 << 6) >> 4) as i64;
            return format!("bl #{:+x}", offset);
        }

        if (insn & 0xFC000000) == 0x14000000 {
            let imm26 = (insn & 0x03FFFFFF) as i32;
            let offset = ((imm26 << 6) >> 4) as i64;
            return format!("b #{:+x}", offset);
        }

        format!(".word 0x{:08x}", insn)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterField {
    Rd,
    Rn,
    Rm,
    Ra,
    Rt,
    Rt2,
}

pub fn decode(bytes: &[u8]) -> Option<InstructionInfo> {
    Arm64Utils::decode_instruction(bytes)
}

pub fn is_branch(insn: u32) -> bool {
    Arm64Utils::is_branch(insn)
}

pub fn is_call(insn: u32) -> bool {
    Arm64Utils::is_call(insn)
}

pub fn is_return(insn: u32) -> bool {
    Arm64Utils::is_return(insn)
}

pub fn disassemble(insn: u32) -> String {
    Arm64Utils::disassemble(insn)
}
