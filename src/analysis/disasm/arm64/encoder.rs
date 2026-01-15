// Wed Jan 15 2026 - Alex

use super::{Arm64Register, Arm64Condition, Arm64Shift};
use crate::memory::Address;

pub struct Arm64Encoder;

impl Arm64Encoder {
    pub fn encode_b(target: Address, from: Address) -> u32 {
        let offset = (target.as_u64() as i64 - from.as_u64() as i64) / 4;
        let imm26 = (offset as u32) & 0x03FFFFFF;
        0x14000000 | imm26
    }

    pub fn encode_bl(target: Address, from: Address) -> u32 {
        let offset = (target.as_u64() as i64 - from.as_u64() as i64) / 4;
        let imm26 = (offset as u32) & 0x03FFFFFF;
        0x94000000 | imm26
    }

    pub fn encode_b_cond(target: Address, from: Address, cond: Arm64Condition) -> u32 {
        let offset = (target.as_u64() as i64 - from.as_u64() as i64) / 4;
        let imm19 = ((offset as u32) & 0x7FFFF) << 5;
        0x54000000 | imm19 | (cond.encoding() as u32)
    }

    pub fn encode_cbz(reg: Arm64Register, target: Address, from: Address) -> u32 {
        let offset = (target.as_u64() as i64 - from.as_u64() as i64) / 4;
        let imm19 = ((offset as u32) & 0x7FFFF) << 5;
        let sf = if reg.is_64bit() { 1 << 31 } else { 0 };
        sf | 0x34000000 | imm19 | (reg.encoding() as u32)
    }

    pub fn encode_cbnz(reg: Arm64Register, target: Address, from: Address) -> u32 {
        let offset = (target.as_u64() as i64 - from.as_u64() as i64) / 4;
        let imm19 = ((offset as u32) & 0x7FFFF) << 5;
        let sf = if reg.is_64bit() { 1 << 31 } else { 0 };
        sf | 0x35000000 | imm19 | (reg.encoding() as u32)
    }

    pub fn encode_br(reg: Arm64Register) -> u32 {
        0xD61F0000 | ((reg.encoding() as u32) << 5)
    }

    pub fn encode_blr(reg: Arm64Register) -> u32 {
        0xD63F0000 | ((reg.encoding() as u32) << 5)
    }

    pub fn encode_ret(reg: Arm64Register) -> u32 {
        0xD65F0000 | ((reg.encoding() as u32) << 5)
    }

    pub fn encode_add_imm(rd: Arm64Register, rn: Arm64Register, imm12: u16, shift: bool) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        let sh = if shift { 1 << 22 } else { 0 };
        sf | 0x11000000 | sh | ((imm12 as u32) << 10) | ((rn.encoding() as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_sub_imm(rd: Arm64Register, rn: Arm64Register, imm12: u16, shift: bool) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        let sh = if shift { 1 << 22 } else { 0 };
        sf | 0x51000000 | sh | ((imm12 as u32) << 10) | ((rn.encoding() as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_add_reg(rd: Arm64Register, rn: Arm64Register, rm: Arm64Register) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        sf | 0x0B000000 | ((rm.encoding() as u32) << 16) | ((rn.encoding() as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_sub_reg(rd: Arm64Register, rn: Arm64Register, rm: Arm64Register) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        sf | 0x4B000000 | ((rm.encoding() as u32) << 16) | ((rn.encoding() as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_mov_reg(rd: Arm64Register, rm: Arm64Register) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        sf | 0x2A0003E0 | ((rm.encoding() as u32) << 16) | (rd.encoding() as u32)
    }

    pub fn encode_movz(rd: Arm64Register, imm16: u16, shift: u8) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        let hw = ((shift / 16) as u32) << 21;
        sf | 0x52800000 | hw | ((imm16 as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_movk(rd: Arm64Register, imm16: u16, shift: u8) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        let hw = ((shift / 16) as u32) << 21;
        sf | 0x72800000 | hw | ((imm16 as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_ldr_imm(rt: Arm64Register, rn: Arm64Register, offset: i16) -> u32 {
        let sf = if rt.is_64bit() { 1 << 30 } else { 0 };
        let scale = if rt.is_64bit() { 3 } else { 2 };
        let imm12 = ((offset >> scale) as u32) & 0xFFF;
        sf | 0x39400000 | (imm12 << 10) | ((rn.encoding() as u32) << 5) | (rt.encoding() as u32)
    }

    pub fn encode_str_imm(rt: Arm64Register, rn: Arm64Register, offset: i16) -> u32 {
        let sf = if rt.is_64bit() { 1 << 30 } else { 0 };
        let scale = if rt.is_64bit() { 3 } else { 2 };
        let imm12 = ((offset >> scale) as u32) & 0xFFF;
        sf | 0x39000000 | (imm12 << 10) | ((rn.encoding() as u32) << 5) | (rt.encoding() as u32)
    }

    pub fn encode_stp_pre(rt1: Arm64Register, rt2: Arm64Register, rn: Arm64Register, offset: i16) -> u32 {
        let sf = if rt1.is_64bit() { 1 << 31 } else { 0 };
        let imm7 = ((offset / 8) as u32) & 0x7F;
        sf | 0x29800000 | (imm7 << 15) | ((rt2.encoding() as u32) << 10) | ((rn.encoding() as u32) << 5) | (rt1.encoding() as u32)
    }

    pub fn encode_ldp_post(rt1: Arm64Register, rt2: Arm64Register, rn: Arm64Register, offset: i16) -> u32 {
        let sf = if rt1.is_64bit() { 1 << 31 } else { 0 };
        let imm7 = ((offset / 8) as u32) & 0x7F;
        sf | 0x28C00000 | (imm7 << 15) | ((rt2.encoding() as u32) << 10) | ((rn.encoding() as u32) << 5) | (rt1.encoding() as u32)
    }

    pub fn encode_cmp_imm(rn: Arm64Register, imm12: u16) -> u32 {
        let sf = if rn.is_64bit() { 1 << 31 } else { 0 };
        sf | 0x7100001F | ((imm12 as u32) << 10) | ((rn.encoding() as u32) << 5)
    }

    pub fn encode_cmp_reg(rn: Arm64Register, rm: Arm64Register) -> u32 {
        let sf = if rn.is_64bit() { 1 << 31 } else { 0 };
        sf | 0x6B00001F | ((rm.encoding() as u32) << 16) | ((rn.encoding() as u32) << 5)
    }

    pub fn encode_tst_imm(rn: Arm64Register, imm: u64) -> u32 {
        let sf = if rn.is_64bit() { 1 << 31 } else { 0 };
        sf | 0x7200001F | ((rn.encoding() as u32) << 5)
    }

    pub fn encode_and_reg(rd: Arm64Register, rn: Arm64Register, rm: Arm64Register) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        sf | 0x0A000000 | ((rm.encoding() as u32) << 16) | ((rn.encoding() as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_orr_reg(rd: Arm64Register, rn: Arm64Register, rm: Arm64Register) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        sf | 0x2A000000 | ((rm.encoding() as u32) << 16) | ((rn.encoding() as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_eor_reg(rd: Arm64Register, rn: Arm64Register, rm: Arm64Register) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        sf | 0x4A000000 | ((rm.encoding() as u32) << 16) | ((rn.encoding() as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_lsl_imm(rd: Arm64Register, rn: Arm64Register, shift: u8) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        let bits = if rd.is_64bit() { 64 } else { 32 };
        let immr = (bits - shift as u32) % bits;
        let imms = bits - 1 - shift as u32;
        sf | 0x53000000 | (immr << 16) | (imms << 10) | ((rn.encoding() as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_lsr_imm(rd: Arm64Register, rn: Arm64Register, shift: u8) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        let bits = if rd.is_64bit() { 63 } else { 31 };
        sf | 0x53000000 | ((shift as u32) << 16) | (bits << 10) | ((rn.encoding() as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_asr_imm(rd: Arm64Register, rn: Arm64Register, shift: u8) -> u32 {
        let sf = if rd.is_64bit() { 1 << 31 } else { 0 };
        let bits = if rd.is_64bit() { 63 } else { 31 };
        sf | 0x13000000 | ((shift as u32) << 16) | (bits << 10) | ((rn.encoding() as u32) << 5) | (rd.encoding() as u32)
    }

    pub fn encode_nop() -> u32 {
        0xD503201F
    }

    pub fn encode_brk(imm16: u16) -> u32 {
        0xD4200000 | ((imm16 as u32) << 5)
    }

    pub fn encode_svc(imm16: u16) -> u32 {
        0xD4000001 | ((imm16 as u32) << 5)
    }
}
