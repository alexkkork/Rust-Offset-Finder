// Tue Jan 13 2026 - Alex

pub struct InstructionEncoder;

impl InstructionEncoder {
    pub fn encode_b(offset: i64) -> u32 {
        let imm26 = ((offset >> 2) as u32) & 0x03FFFFFF;
        0x14000000 | imm26
    }

    pub fn encode_bl(offset: i64) -> u32 {
        let imm26 = ((offset >> 2) as u32) & 0x03FFFFFF;
        0x94000000 | imm26
    }

    pub fn encode_br(rn: u8) -> u32 {
        let rn = (rn & 0x1F) as u32;
        0xD61F0000 | (rn << 5)
    }

    pub fn encode_blr(rn: u8) -> u32 {
        let rn = (rn & 0x1F) as u32;
        0xD63F0000 | (rn << 5)
    }

    pub fn encode_ret(rn: u8) -> u32 {
        let rn = (rn & 0x1F) as u32;
        0xD65F0000 | (rn << 5)
    }

    pub fn encode_cbz(rt: u8, offset: i64, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let rt = (rt & 0x1F) as u32;
        let imm19 = ((offset >> 2) as u32) & 0x7FFFF;
        (sf << 31) | 0x34000000 | (imm19 << 5) | rt
    }

    pub fn encode_cbnz(rt: u8, offset: i64, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let rt = (rt & 0x1F) as u32;
        let imm19 = ((offset >> 2) as u32) & 0x7FFFF;
        (sf << 31) | 0x35000000 | (imm19 << 5) | rt
    }

    pub fn encode_b_cond(cond: u8, offset: i64) -> u32 {
        let cond = (cond & 0xF) as u32;
        let imm19 = ((offset >> 2) as u32) & 0x7FFFF;
        0x54000000 | (imm19 << 5) | cond
    }

    pub fn encode_adr(rd: u8, offset: i64) -> u32 {
        let rd = (rd & 0x1F) as u32;
        let immlo = (offset as u32) & 0x3;
        let immhi = ((offset >> 2) as u32) & 0x7FFFF;
        (immlo << 29) | 0x10000000 | (immhi << 5) | rd
    }

    pub fn encode_adrp(rd: u8, offset: i64) -> u32 {
        let rd = (rd & 0x1F) as u32;
        let immlo = ((offset >> 12) as u32) & 0x3;
        let immhi = ((offset >> 14) as u32) & 0x7FFFF;
        (immlo << 29) | 0x90000000 | (immhi << 5) | rd
    }

    pub fn encode_add_imm(rd: u8, rn: u8, imm12: u16, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let rd = (rd & 0x1F) as u32;
        let rn = (rn & 0x1F) as u32;
        let imm = (imm12 & 0xFFF) as u32;
        (sf << 31) | 0x11000000 | (imm << 10) | (rn << 5) | rd
    }

    pub fn encode_sub_imm(rd: u8, rn: u8, imm12: u16, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let rd = (rd & 0x1F) as u32;
        let rn = (rn & 0x1F) as u32;
        let imm = (imm12 & 0xFFF) as u32;
        (sf << 31) | 0x51000000 | (imm << 10) | (rn << 5) | rd
    }

    pub fn encode_add_reg(rd: u8, rn: u8, rm: u8, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let rd = (rd & 0x1F) as u32;
        let rn = (rn & 0x1F) as u32;
        let rm = (rm & 0x1F) as u32;
        (sf << 31) | 0x0B000000 | (rm << 16) | (rn << 5) | rd
    }

    pub fn encode_sub_reg(rd: u8, rn: u8, rm: u8, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let rd = (rd & 0x1F) as u32;
        let rn = (rn & 0x1F) as u32;
        let rm = (rm & 0x1F) as u32;
        (sf << 31) | 0x4B000000 | (rm << 16) | (rn << 5) | rd
    }

    pub fn encode_mov_reg(rd: u8, rm: u8, is_64bit: bool) -> u32 {
        Self::encode_orr_reg(rd, 31, rm, is_64bit)
    }

    pub fn encode_movz(rd: u8, imm16: u16, shift: u8, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let hw = ((shift / 16) & 0x3) as u32;
        let rd = (rd & 0x1F) as u32;
        let imm = imm16 as u32;
        (sf << 31) | 0x52800000 | (hw << 21) | (imm << 5) | rd
    }

    pub fn encode_movn(rd: u8, imm16: u16, shift: u8, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let hw = ((shift / 16) & 0x3) as u32;
        let rd = (rd & 0x1F) as u32;
        let imm = imm16 as u32;
        (sf << 31) | 0x12800000 | (hw << 21) | (imm << 5) | rd
    }

    pub fn encode_movk(rd: u8, imm16: u16, shift: u8, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let hw = ((shift / 16) & 0x3) as u32;
        let rd = (rd & 0x1F) as u32;
        let imm = imm16 as u32;
        (sf << 31) | 0x72800000 | (hw << 21) | (imm << 5) | rd
    }

    pub fn encode_and_reg(rd: u8, rn: u8, rm: u8, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let rd = (rd & 0x1F) as u32;
        let rn = (rn & 0x1F) as u32;
        let rm = (rm & 0x1F) as u32;
        (sf << 31) | 0x0A000000 | (rm << 16) | (rn << 5) | rd
    }

    pub fn encode_orr_reg(rd: u8, rn: u8, rm: u8, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let rd = (rd & 0x1F) as u32;
        let rn = (rn & 0x1F) as u32;
        let rm = (rm & 0x1F) as u32;
        (sf << 31) | 0x2A000000 | (rm << 16) | (rn << 5) | rd
    }

    pub fn encode_eor_reg(rd: u8, rn: u8, rm: u8, is_64bit: bool) -> u32 {
        let sf = if is_64bit { 1u32 } else { 0u32 };
        let rd = (rd & 0x1F) as u32;
        let rn = (rn & 0x1F) as u32;
        let rm = (rm & 0x1F) as u32;
        (sf << 31) | 0x4A000000 | (rm << 16) | (rn << 5) | rd
    }

    pub fn encode_ldr_imm_unsigned(rt: u8, rn: u8, offset: u16, size: u8) -> u32 {
        let size_bits = match size {
            1 => 0b00u32,
            2 => 0b01u32,
            4 => 0b10u32,
            8 => 0b11u32,
            _ => 0b11u32,
        };
        let rt = (rt & 0x1F) as u32;
        let rn = (rn & 0x1F) as u32;
        let imm12 = ((offset as u32) / size as u32) & 0xFFF;
        (size_bits << 30) | 0x39400000 | (imm12 << 10) | (rn << 5) | rt
    }

    pub fn encode_str_imm_unsigned(rt: u8, rn: u8, offset: u16, size: u8) -> u32 {
        let size_bits = match size {
            1 => 0b00u32,
            2 => 0b01u32,
            4 => 0b10u32,
            8 => 0b11u32,
            _ => 0b11u32,
        };
        let rt = (rt & 0x1F) as u32;
        let rn = (rn & 0x1F) as u32;
        let imm12 = ((offset as u32) / size as u32) & 0xFFF;
        (size_bits << 30) | 0x39000000 | (imm12 << 10) | (rn << 5) | rt
    }

    pub fn encode_ldr_literal(rt: u8, offset: i64, is_64bit: bool) -> u32 {
        let opc = if is_64bit { 0b01u32 } else { 0b00u32 };
        let rt = (rt & 0x1F) as u32;
        let imm19 = ((offset >> 2) as u32) & 0x7FFFF;
        (opc << 30) | 0x18000000 | (imm19 << 5) | rt
    }

    pub fn encode_stp_pre(rt: u8, rt2: u8, rn: u8, offset: i16, is_64bit: bool) -> u32 {
        let opc = if is_64bit { 0b10u32 } else { 0b00u32 };
        let rt = (rt & 0x1F) as u32;
        let rt2 = (rt2 & 0x1F) as u32;
        let rn = (rn & 0x1F) as u32;
        let scale = if is_64bit { 8 } else { 4 };
        let imm7 = ((offset / scale) as u32) & 0x7F;
        (opc << 30) | 0x29800000 | (imm7 << 15) | (rt2 << 10) | (rn << 5) | rt
    }

    pub fn encode_ldp_post(rt: u8, rt2: u8, rn: u8, offset: i16, is_64bit: bool) -> u32 {
        let opc = if is_64bit { 0b10u32 } else { 0b00u32 };
        let rt = (rt & 0x1F) as u32;
        let rt2 = (rt2 & 0x1F) as u32;
        let rn = (rn & 0x1F) as u32;
        let scale = if is_64bit { 8 } else { 4 };
        let imm7 = ((offset / scale) as u32) & 0x7F;
        (opc << 30) | 0x28C00000 | (imm7 << 15) | (rt2 << 10) | (rn << 5) | rt
    }

    pub fn encode_nop() -> u32 {
        0xD503201F
    }

    pub fn encode_brk(imm16: u16) -> u32 {
        let imm = imm16 as u32;
        0xD4200000 | (imm << 5)
    }

    pub fn encode_svc(imm16: u16) -> u32 {
        let imm = imm16 as u32;
        0xD4000001 | (imm << 5)
    }

    pub fn encode_cmp_imm(rn: u8, imm12: u16, is_64bit: bool) -> u32 {
        Self::encode_sub_imm(31, rn, imm12, is_64bit) | (1 << 29)
    }

    pub fn encode_cmp_reg(rn: u8, rm: u8, is_64bit: bool) -> u32 {
        Self::encode_sub_reg(31, rn, rm, is_64bit) | (1 << 29)
    }
}

pub fn encode_branch(offset: i64) -> u32 {
    InstructionEncoder::encode_b(offset)
}

pub fn encode_call(offset: i64) -> u32 {
    InstructionEncoder::encode_bl(offset)
}

pub fn encode_return() -> u32 {
    InstructionEncoder::encode_ret(30)
}

pub fn encode_nop() -> u32 {
    InstructionEncoder::encode_nop()
}
