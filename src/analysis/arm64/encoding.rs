// Tue Jan 13 2026 - Alex

use crate::analysis::arm64::{Opcode, Register};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingClass {
    Reserved,
    SME,
    SVE,
    DataProcessingImmediate,
    BranchExceptionSystem,
    LoadsStores,
    DataProcessingRegister,
    DataProcessingSimdFp,
}

#[derive(Debug, Clone)]
pub struct InstructionEncoding {
    pub class: EncodingClass,
    pub op0: u8,
    pub op1: u8,
    pub op2: u8,
    pub op3: u8,
    pub op4: u8,
}

impl InstructionEncoding {
    pub fn from_raw(raw: u32) -> Self {
        let op0 = ((raw >> 25) & 0xF) as u8;

        let class = match op0 {
            0b0000 => EncodingClass::Reserved,
            0b0001 => EncodingClass::Reserved,
            0b0010 => EncodingClass::SVE,
            0b0011 => EncodingClass::Reserved,
            0b1000 | 0b1001 => EncodingClass::DataProcessingImmediate,
            0b1010 | 0b1011 => EncodingClass::BranchExceptionSystem,
            0b0100 | 0b0110 | 0b1100 | 0b1110 => EncodingClass::LoadsStores,
            0b0101 | 0b1101 => EncodingClass::DataProcessingRegister,
            0b0111 | 0b1111 => EncodingClass::DataProcessingSimdFp,
            _ => EncodingClass::Reserved,
        };

        Self {
            class,
            op0,
            op1: ((raw >> 21) & 0xF) as u8,
            op2: ((raw >> 16) & 0x1F) as u8,
            op3: ((raw >> 10) & 0x3F) as u8,
            op4: (raw & 0x1F) as u8,
        }
    }

    pub fn is_valid(&self) -> bool {
        !matches!(self.class, EncodingClass::Reserved)
    }
}

pub fn encode_add_imm(rd: Register, rn: Register, imm: u16, shift: bool) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };
    let sh = if shift { 1u32 } else { 0u32 };

    (sf << 31) | (0b00100010 << 23) | (sh << 22) | ((imm as u32) << 10) |
    ((rn.index() as u32) << 5) | (rd.index() as u32)
}

pub fn encode_sub_imm(rd: Register, rn: Register, imm: u16, shift: bool) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };
    let sh = if shift { 1u32 } else { 0u32 };

    (sf << 31) | (0b10100010 << 23) | (sh << 22) | ((imm as u32) << 10) |
    ((rn.index() as u32) << 5) | (rd.index() as u32)
}

pub fn encode_mov_imm(rd: Register, imm: u16, shift: u8) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };
    let hw = (shift / 16) as u32;

    (sf << 31) | (0b10100101 << 23) | (hw << 21) | ((imm as u32) << 5) | (rd.index() as u32)
}

pub fn encode_movk(rd: Register, imm: u16, shift: u8) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };
    let hw = (shift / 16) as u32;

    (sf << 31) | (0b11100101 << 23) | (hw << 21) | ((imm as u32) << 5) | (rd.index() as u32)
}

pub fn encode_b(offset: i32) -> u32 {
    let imm26 = ((offset >> 2) & 0x3FFFFFF) as u32;
    (0b000101 << 26) | imm26
}

pub fn encode_bl(offset: i32) -> u32 {
    let imm26 = ((offset >> 2) & 0x3FFFFFF) as u32;
    (0b100101 << 26) | imm26
}

pub fn encode_br(rn: Register) -> u32 {
    (0b1101011000011111 << 16) | ((rn.index() as u32) << 5)
}

pub fn encode_blr(rn: Register) -> u32 {
    (0b1101011000111111 << 16) | ((rn.index() as u32) << 5)
}

pub fn encode_ret(rn: Register) -> u32 {
    (0b1101011001011111 << 16) | ((rn.index() as u32) << 5)
}

pub fn encode_cbz(rt: Register, offset: i32) -> u32 {
    let sf = if rt.size().bits() == 64 { 1u32 } else { 0u32 };
    let imm19 = ((offset >> 2) & 0x7FFFF) as u32;

    (sf << 31) | (0b0110100 << 24) | (imm19 << 5) | (rt.index() as u32)
}

pub fn encode_cbnz(rt: Register, offset: i32) -> u32 {
    let sf = if rt.size().bits() == 64 { 1u32 } else { 0u32 };
    let imm19 = ((offset >> 2) & 0x7FFFF) as u32;

    (sf << 31) | (0b0110101 << 24) | (imm19 << 5) | (rt.index() as u32)
}

pub fn encode_ldr_imm(rt: Register, rn: Register, offset: i64) -> u32 {
    let sf = if rt.size().bits() == 64 { 1u32 } else { 0u32 };
    let size = if sf == 1 { 0b11u32 } else { 0b10u32 };
    let scale = if sf == 1 { 3 } else { 2 };
    let imm12 = ((offset >> scale) & 0xFFF) as u32;

    (size << 30) | (0b11100101 << 22) | (imm12 << 10) |
    ((rn.index() as u32) << 5) | (rt.index() as u32)
}

pub fn encode_str_imm(rt: Register, rn: Register, offset: i64) -> u32 {
    let sf = if rt.size().bits() == 64 { 1u32 } else { 0u32 };
    let size = if sf == 1 { 0b11u32 } else { 0b10u32 };
    let scale = if sf == 1 { 3 } else { 2 };
    let imm12 = ((offset >> scale) & 0xFFF) as u32;

    (size << 30) | (0b11100100 << 22) | (imm12 << 10) |
    ((rn.index() as u32) << 5) | (rt.index() as u32)
}

pub fn encode_ldp(rt1: Register, rt2: Register, rn: Register, offset: i64) -> u32 {
    let sf = if rt1.size().bits() == 64 { 1u32 } else { 0u32 };
    let opc = if sf == 1 { 0b10u32 } else { 0b00u32 };
    let scale = if sf == 1 { 3 } else { 2 };
    let imm7 = ((offset >> scale) & 0x7F) as u32;

    (opc << 30) | (0b10100101 << 22) | (imm7 << 15) |
    ((rt2.index() as u32) << 10) | ((rn.index() as u32) << 5) | (rt1.index() as u32)
}

pub fn encode_stp(rt1: Register, rt2: Register, rn: Register, offset: i64) -> u32 {
    let sf = if rt1.size().bits() == 64 { 1u32 } else { 0u32 };
    let opc = if sf == 1 { 0b10u32 } else { 0b00u32 };
    let scale = if sf == 1 { 3 } else { 2 };
    let imm7 = ((offset >> scale) & 0x7F) as u32;

    (opc << 30) | (0b10100100 << 22) | (imm7 << 15) |
    ((rt2.index() as u32) << 10) | ((rn.index() as u32) << 5) | (rt1.index() as u32)
}

pub fn encode_add_reg(rd: Register, rn: Register, rm: Register, shift: u8, amount: u8) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };

    (sf << 31) | (0b0001011 << 24) | ((shift as u32) << 22) |
    ((rm.index() as u32) << 16) | ((amount as u32) << 10) |
    ((rn.index() as u32) << 5) | (rd.index() as u32)
}

pub fn encode_sub_reg(rd: Register, rn: Register, rm: Register, shift: u8, amount: u8) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };

    (sf << 31) | (0b1001011 << 24) | ((shift as u32) << 22) |
    ((rm.index() as u32) << 16) | ((amount as u32) << 10) |
    ((rn.index() as u32) << 5) | (rd.index() as u32)
}

pub fn encode_and_reg(rd: Register, rn: Register, rm: Register, shift: u8, amount: u8) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };

    (sf << 31) | (0b0001010 << 24) | ((shift as u32) << 22) |
    ((rm.index() as u32) << 16) | ((amount as u32) << 10) |
    ((rn.index() as u32) << 5) | (rd.index() as u32)
}

pub fn encode_orr_reg(rd: Register, rn: Register, rm: Register, shift: u8, amount: u8) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };

    (sf << 31) | (0b0101010 << 24) | ((shift as u32) << 22) |
    ((rm.index() as u32) << 16) | ((amount as u32) << 10) |
    ((rn.index() as u32) << 5) | (rd.index() as u32)
}

pub fn encode_eor_reg(rd: Register, rn: Register, rm: Register, shift: u8, amount: u8) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };

    (sf << 31) | (0b1001010 << 24) | ((shift as u32) << 22) |
    ((rm.index() as u32) << 16) | ((amount as u32) << 10) |
    ((rn.index() as u32) << 5) | (rd.index() as u32)
}

pub fn encode_mul(rd: Register, rn: Register, rm: Register) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };

    (sf << 31) | (0b0011011000 << 21) | ((rm.index() as u32) << 16) |
    (0b11111 << 10) | ((rn.index() as u32) << 5) | (rd.index() as u32)
}

pub fn encode_sdiv(rd: Register, rn: Register, rm: Register) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };

    (sf << 31) | (0b0011010110 << 21) | ((rm.index() as u32) << 16) |
    (0b000011 << 10) | ((rn.index() as u32) << 5) | (rd.index() as u32)
}

pub fn encode_udiv(rd: Register, rn: Register, rm: Register) -> u32 {
    let sf = if rd.size().bits() == 64 { 1u32 } else { 0u32 };

    (sf << 31) | (0b0011010110 << 21) | ((rm.index() as u32) << 16) |
    (0b000010 << 10) | ((rn.index() as u32) << 5) | (rd.index() as u32)
}

pub fn encode_nop() -> u32 {
    0xD503201F
}

pub fn encode_brk(imm: u16) -> u32 {
    (0b11010100001 << 21) | ((imm as u32) << 5)
}

pub fn encode_svc(imm: u16) -> u32 {
    (0b11010100000 << 21) | ((imm as u32) << 5) | 0b00001
}

pub fn decode_immediate(raw: u32, bits: u8, signed: bool) -> i64 {
    let mask = (1u32 << bits) - 1;
    let val = raw & mask;

    if signed && (val & (1 << (bits - 1))) != 0 {
        let sign_extend = !((1i64 << bits) - 1);
        (val as i64) | sign_extend
    } else {
        val as i64
    }
}

pub fn encode_immediate(val: i64, bits: u8) -> u32 {
    let mask = (1u32 << bits) - 1;
    (val as u32) & mask
}

pub fn extract_bits(raw: u32, start: u8, len: u8) -> u32 {
    (raw >> start) & ((1 << len) - 1)
}

pub fn insert_bits(val: u32, raw: u32, start: u8, len: u8) -> u32 {
    let mask = ((1u32 << len) - 1) << start;
    (raw & !mask) | ((val << start) & mask)
}

pub fn is_valid_immediate(val: i64, bits: u8, signed: bool) -> bool {
    if signed {
        let min = -(1i64 << (bits - 1));
        let max = (1i64 << (bits - 1)) - 1;
        val >= min && val <= max
    } else {
        val >= 0 && val < (1i64 << bits)
    }
}

pub fn align_down(val: u64, alignment: u64) -> u64 {
    val & !(alignment - 1)
}

pub fn align_up(val: u64, alignment: u64) -> u64 {
    (val + alignment - 1) & !(alignment - 1)
}
