// Tue Jan 13 2026 - Alex

use crate::memory::Address;
use crate::analysis::arm64::{
    Arm64Instruction, Opcode, Operand, OperandType, Register, RegisterSize,
    Condition, ShiftType, ExtendType,
};

pub struct Arm64Decoder {
    pc: Address,
}

impl Arm64Decoder {
    pub fn new() -> Self {
        Self {
            pc: Address::new(0),
        }
    }

    pub fn decode(&self, address: Address, raw: u32) -> Arm64Instruction {
        let mut insn = Arm64Instruction::new(address, raw);

        let op0 = (raw >> 25) & 0xF;

        match op0 {
            0b0000 | 0b0001 | 0b0010 | 0b0011 => {
                insn = self.decode_unallocated(insn, raw);
            }
            0b1000 | 0b1001 => {
                insn = self.decode_data_processing_imm(insn, raw);
            }
            0b1010 | 0b1011 => {
                insn = self.decode_branch_exception_system(insn, raw);
            }
            0b0100 | 0b0110 | 0b1100 | 0b1110 => {
                insn = self.decode_loads_stores(insn, raw);
            }
            0b0101 | 0b1101 => {
                insn = self.decode_data_processing_reg(insn, raw);
            }
            0b0111 | 0b1111 => {
                insn = self.decode_simd_fp(insn, raw);
            }
            _ => {}
        }

        insn
    }

    pub fn is_valid(&self, raw: u32) -> bool {
        let op0 = (raw >> 25) & 0xF;
        match op0 {
            0b0000 | 0b0001 | 0b0010 | 0b0011 => false,
            _ => true,
        }
    }

    fn decode_unallocated(&self, insn: Arm64Instruction, _raw: u32) -> Arm64Instruction {
        insn.with_opcode(Opcode::Unknown)
    }

    fn decode_data_processing_imm(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let op0 = (raw >> 23) & 0x7;

        match op0 {
            0b000 | 0b001 => {
                insn = self.decode_pc_rel_addressing(insn, raw);
            }
            0b010 => {
                insn = self.decode_add_sub_imm(insn, raw);
            }
            0b011 => {
                insn = self.decode_add_sub_imm_tags(insn, raw);
            }
            0b100 => {
                insn = self.decode_logical_imm(insn, raw);
            }
            0b101 => {
                insn = self.decode_move_wide_imm(insn, raw);
            }
            0b110 => {
                insn = self.decode_bitfield(insn, raw);
            }
            0b111 => {
                insn = self.decode_extract(insn, raw);
            }
            _ => {}
        }

        insn
    }

    fn decode_pc_rel_addressing(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let op = (raw >> 31) & 1;
        let immlo = (raw >> 29) & 0x3;
        let immhi = (raw >> 5) & 0x7FFFF;
        let rd = (raw & 0x1F) as u8;

        let imm = ((immhi << 2) | immlo) as i32;
        let imm = if (imm & 0x100000) != 0 {
            imm | !0x1FFFFF
        } else {
            imm
        };

        if op == 0 {
            insn = insn.with_opcode(Opcode::ADR);
            let target = (insn.address.as_u64() as i64 + imm as i64) as u64;
            insn.operands.push(Operand::register(Register::x(rd)));
            insn.operands.push(Operand::pc_relative(imm));
        } else {
            insn = insn.with_opcode(Opcode::ADRP);
            let imm_page = (imm as i64) << 12;
            let target = ((insn.address.as_u64() as i64 & !0xFFF) + imm_page) as u64;
            insn.operands.push(Operand::register(Register::x(rd)));
            insn.operands.push(Operand::immediate(target as i64));
        }

        insn
    }

    fn decode_add_sub_imm(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let op = (raw >> 30) & 1;
        let s = (raw >> 29) & 1;
        let sh = (raw >> 22) & 1;
        let imm12 = ((raw >> 10) & 0xFFF) as i64;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        let imm = if sh == 1 { imm12 << 12 } else { imm12 };

        let opcode = match (op, s) {
            (0, 0) => Opcode::ADD,
            (0, 1) => Opcode::ADDS,
            (1, 0) => Opcode::SUB,
            (1, 1) => Opcode::SUBS,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rd)));
            insn.operands.push(Operand::register(Register::x(rn)));
        } else {
            insn.operands.push(Operand::register(Register::w(rd)));
            insn.operands.push(Operand::register(Register::w(rn)));
        }
        insn.operands.push(Operand::immediate(imm));

        if rd == 31 && s == 1 {
            insn = insn.with_opcode(if op == 1 { Opcode::CMP } else { Opcode::CMN });
            insn.operands.remove(0);
        }

        if rn == 31 && op == 1 && s == 0 {
            insn = insn.with_opcode(Opcode::NEG);
            insn.operands.remove(1);
        }

        insn
    }

    fn decode_add_sub_imm_tags(&self, insn: Arm64Instruction, _raw: u32) -> Arm64Instruction {
        insn.with_opcode(Opcode::Unknown)
    }

    fn decode_logical_imm(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let opc = (raw >> 29) & 0x3;
        let n = (raw >> 22) & 1;
        let immr = ((raw >> 16) & 0x3F) as u8;
        let imms = ((raw >> 10) & 0x3F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        let opcode = match opc {
            0b00 => Opcode::AND,
            0b01 => Opcode::ORR,
            0b10 => Opcode::EOR,
            0b11 => Opcode::ANDS,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        let imm = self.decode_bitmask_immediate(n as u8, imms, immr, sf == 1);

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rd)));
            insn.operands.push(Operand::register(Register::x(rn)));
        } else {
            insn.operands.push(Operand::register(Register::w(rd)));
            insn.operands.push(Operand::register(Register::w(rn)));
        }
        insn.operands.push(Operand::immediate(imm as i64));

        if opc == 0b01 && rn == 31 {
            insn = insn.with_opcode(Opcode::MOV);
            insn.operands.remove(1);
        }

        if rd == 31 && opc == 0b11 {
            insn = insn.with_opcode(Opcode::TST);
            insn.operands.remove(0);
        }

        insn
    }

    fn decode_bitmask_immediate(&self, n: u8, imms: u8, immr: u8, is_64bit: bool) -> u64 {
        let len = if n == 1 { 6 } else { (imms as u32).leading_zeros() - 26 };
        if len < 1 {
            return 0;
        }

        let levels = (1u32 << len) - 1;
        let s = (imms as u32) & levels;
        let r = (immr as u32) & levels;
        let diff = s.wrapping_sub(r);
        let esize = 1u32 << len;
        let welem = ((1u64 << (s + 1)) - 1) as u64;
        let wmask = welem.rotate_right(r);

        if is_64bit {
            let mut result = 0u64;
            for i in 0..(64 / esize) {
                result |= wmask << (i * esize);
            }
            result
        } else {
            wmask as u64
        }
    }

    fn decode_move_wide_imm(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let opc = (raw >> 29) & 0x3;
        let hw = ((raw >> 21) & 0x3) as u8;
        let imm16 = ((raw >> 5) & 0xFFFF) as i64;
        let rd = (raw & 0x1F) as u8;

        let opcode = match opc {
            0b00 => Opcode::MOVN,
            0b10 => Opcode::MOVZ,
            0b11 => Opcode::MOVK,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rd)));
        } else {
            insn.operands.push(Operand::register(Register::w(rd)));
        }

        let shift = (hw as i64) * 16;
        insn.operands.push(Operand::immediate_shifted(imm16, ShiftType::LSL, shift as u8));

        insn
    }

    fn decode_bitfield(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let opc = (raw >> 29) & 0x3;
        let n = (raw >> 22) & 1;
        let immr = ((raw >> 16) & 0x3F) as u8;
        let imms = ((raw >> 10) & 0x3F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        let opcode = match opc {
            0b00 => Opcode::SBFM,
            0b01 => Opcode::BFM,
            0b10 => Opcode::UBFM,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rd)));
            insn.operands.push(Operand::register(Register::x(rn)));
        } else {
            insn.operands.push(Operand::register(Register::w(rd)));
            insn.operands.push(Operand::register(Register::w(rn)));
        }
        insn.operands.push(Operand::immediate(immr as i64));
        insn.operands.push(Operand::immediate(imms as i64));

        let regsize = if sf == 1 { 64 } else { 32 };

        if opc == 0b10 && imms == regsize - 1 {
            insn = insn.with_opcode(Opcode::LSR);
            insn.operands.truncate(3);
        } else if opc == 0b10 && imms + 1 == immr {
            insn = insn.with_opcode(Opcode::LSL);
            insn.operands.truncate(2);
            insn.operands.push(Operand::immediate((regsize - immr as u8) as i64));
        } else if opc == 0b00 && imms == regsize - 1 {
            insn = insn.with_opcode(Opcode::ASR);
            insn.operands.truncate(3);
        } else if opc == 0b10 && immr == 0 && imms == 7 {
            insn = insn.with_opcode(Opcode::UXTB);
            insn.operands.truncate(2);
        } else if opc == 0b10 && immr == 0 && imms == 15 {
            insn = insn.with_opcode(Opcode::UXTH);
            insn.operands.truncate(2);
        } else if opc == 0b00 && immr == 0 && imms == 7 {
            insn = insn.with_opcode(Opcode::SXTB);
            insn.operands.truncate(2);
        } else if opc == 0b00 && immr == 0 && imms == 15 {
            insn = insn.with_opcode(Opcode::SXTH);
            insn.operands.truncate(2);
        } else if opc == 0b00 && immr == 0 && imms == 31 {
            insn = insn.with_opcode(Opcode::SXTW);
            insn.operands.truncate(2);
        }

        insn
    }

    fn decode_extract(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let n = (raw >> 22) & 1;
        let rm = ((raw >> 16) & 0x1F) as u8;
        let imms = ((raw >> 10) & 0x3F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        insn = insn.with_opcode(Opcode::EXTR);

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rd)));
            insn.operands.push(Operand::register(Register::x(rn)));
            insn.operands.push(Operand::register(Register::x(rm)));
        } else {
            insn.operands.push(Operand::register(Register::w(rd)));
            insn.operands.push(Operand::register(Register::w(rn)));
            insn.operands.push(Operand::register(Register::w(rm)));
        }
        insn.operands.push(Operand::immediate(imms as i64));

        if rn == rm {
            insn = insn.with_opcode(Opcode::ROR);
            insn.operands.remove(2);
        }

        insn
    }

    fn decode_branch_exception_system(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let op0 = (raw >> 29) & 0x7;
        let op1 = (raw >> 22) & 0x7F;

        if op0 == 0b010 && (op1 & 0x40) == 0 {
            return self.decode_conditional_branch(insn, raw);
        }

        if op0 == 0b110 && (op1 & 0x40) == 0 {
            return self.decode_exception_generation(insn, raw);
        }

        if op0 == 0b110 && (op1 & 0x7C) == 0x40 {
            return self.decode_system(insn, raw);
        }

        if op0 == 0b110 && (op1 & 0x7C) == 0x5C {
            return self.decode_unconditional_branch_register(insn, raw);
        }

        if (op0 & 0x3) == 0 {
            return self.decode_unconditional_branch_imm(insn, raw);
        }

        if (op0 & 0x3) == 0x1 {
            return self.decode_compare_and_branch(insn, raw);
        }

        if (op0 & 0x3) == 0x1 {
            return self.decode_test_and_branch(insn, raw);
        }

        insn
    }

    fn decode_conditional_branch(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let imm19 = ((raw >> 5) & 0x7FFFF) as i32;
        let imm = if (imm19 & 0x40000) != 0 {
            (imm19 | !0x7FFFF) << 2
        } else {
            imm19 << 2
        };
        let cond = (raw & 0xF) as u8;

        insn = insn.with_opcode(Opcode::Bcc);
        insn = insn.with_condition(Condition::from_code(cond));
        insn.operands.push(Operand::pc_relative(imm));

        insn
    }

    fn decode_exception_generation(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let opc = (raw >> 21) & 0x7;
        let imm16 = ((raw >> 5) & 0xFFFF) as i64;
        let ll = raw & 0x3;

        let opcode = match (opc, ll) {
            (0b000, 0b01) => Opcode::SVC,
            (0b000, 0b10) => Opcode::HVC,
            (0b000, 0b11) => Opcode::SMC,
            (0b001, 0b00) => Opcode::BRK,
            (0b010, 0b00) => Opcode::HLT,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);
        insn.operands.push(Operand::immediate(imm16));

        insn
    }

    fn decode_system(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let l = (raw >> 21) & 1;
        let op0 = (raw >> 19) & 0x3;
        let op1 = (raw >> 16) & 0x7;
        let crn = (raw >> 12) & 0xF;
        let crm = (raw >> 8) & 0xF;
        let op2 = (raw >> 5) & 0x7;
        let rt = (raw & 0x1F) as u8;

        if op0 == 0 && crn == 3 && op1 == 3 && crm == 2 && op2 == 0 {
            insn = insn.with_opcode(Opcode::NOP);
            return insn;
        }

        if l == 0 {
            insn = insn.with_opcode(Opcode::MSR);
        } else {
            insn = insn.with_opcode(Opcode::MRS);
        }

        let sysreg = (op0 << 14) | (op1 << 11) | (crn << 7) | (crm << 3) | op2;
        insn.operands.push(Operand::system_register(sysreg as u16));
        insn.operands.push(Operand::register(Register::x(rt)));

        insn
    }

    fn decode_unconditional_branch_register(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let opc = (raw >> 21) & 0xF;
        let op2 = (raw >> 16) & 0x1F;
        let op3 = (raw >> 10) & 0x3F;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let op4 = raw & 0x1F;

        let opcode = match (opc, op2, op3, op4) {
            (0b0000, 0b11111, 0b000000, 0b00000) => Opcode::BR,
            (0b0001, 0b11111, 0b000000, 0b00000) => Opcode::BLR,
            (0b0010, 0b11111, 0b000000, 0b00000) => Opcode::RET,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        if opcode != Opcode::RET || rn != 30 {
            insn.operands.push(Operand::register(Register::x(rn)));
        }

        insn
    }

    fn decode_unconditional_branch_imm(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let op = (raw >> 31) & 1;
        let imm26 = (raw & 0x3FFFFFF) as i32;
        let imm = if (imm26 & 0x2000000) != 0 {
            (imm26 | !0x3FFFFFF) << 2
        } else {
            imm26 << 2
        };

        insn = insn.with_opcode(if op == 0 { Opcode::B } else { Opcode::BL });
        insn.operands.push(Operand::pc_relative(imm));

        insn
    }

    fn decode_compare_and_branch(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let op = (raw >> 24) & 1;
        let imm19 = ((raw >> 5) & 0x7FFFF) as i32;
        let imm = if (imm19 & 0x40000) != 0 {
            (imm19 | !0x7FFFF) << 2
        } else {
            imm19 << 2
        };
        let rt = (raw & 0x1F) as u8;

        insn = insn.with_opcode(if op == 0 { Opcode::CBZ } else { Opcode::CBNZ });

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rt)));
        } else {
            insn.operands.push(Operand::register(Register::w(rt)));
        }
        insn.operands.push(Operand::pc_relative(imm));

        insn
    }

    fn decode_test_and_branch(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let b5 = (raw >> 31) & 1;
        let op = (raw >> 24) & 1;
        let b40 = (raw >> 19) & 0x1F;
        let imm14 = ((raw >> 5) & 0x3FFF) as i32;
        let imm = if (imm14 & 0x2000) != 0 {
            (imm14 | !0x3FFF) << 2
        } else {
            imm14 << 2
        };
        let rt = (raw & 0x1F) as u8;

        insn = insn.with_opcode(if op == 0 { Opcode::TBZ } else { Opcode::TBNZ });

        let bit = ((b5 << 5) | b40) as u8;

        if b5 == 1 {
            insn.operands.push(Operand::register(Register::x(rt)));
        } else {
            insn.operands.push(Operand::register(Register::w(rt)));
        }
        insn.operands.push(Operand::immediate(bit as i64));
        insn.operands.push(Operand::pc_relative(imm));

        insn
    }

    fn decode_loads_stores(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let op0 = (raw >> 28) & 0xF;
        let op1 = (raw >> 26) & 1;
        let op2 = (raw >> 23) & 0x3;
        let op3 = (raw >> 16) & 0x3F;
        let op4 = (raw >> 10) & 0x3;

        if (op0 & 0x3) == 0 && op1 == 0 && (op2 & 0x2) == 0 {
            return self.decode_load_store_exclusive(insn, raw);
        }

        if (op0 & 0x3) == 1 {
            return self.decode_load_register_literal(insn, raw);
        }

        if (op0 & 0x3) == 2 && op2 == 0 {
            return self.decode_load_store_no_alloc_pair(insn, raw);
        }

        if (op0 & 0x3) == 2 && (op2 & 0x1) == 1 {
            return self.decode_load_store_pair(insn, raw);
        }

        if (op0 & 0x3) == 3 && op2 == 0 {
            return self.decode_load_store_reg_unscaled(insn, raw);
        }

        if (op0 & 0x3) == 3 && op2 == 1 {
            return self.decode_load_store_reg_post_indexed(insn, raw);
        }

        if (op0 & 0x3) == 3 && op2 == 2 {
            return self.decode_load_store_reg_offset(insn, raw);
        }

        if (op0 & 0x3) == 3 && op2 == 3 {
            return self.decode_load_store_reg_pre_indexed(insn, raw);
        }

        self.decode_load_store_reg_unsigned_imm(insn, raw)
    }

    fn decode_load_store_exclusive(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let size = (raw >> 30) & 0x3;
        let l = (raw >> 22) & 1;
        let rs = ((raw >> 16) & 0x1F) as u8;
        let o0 = (raw >> 15) & 1;
        let rt2 = ((raw >> 10) & 0x1F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rt = (raw & 0x1F) as u8;

        let opcode = match (size, l, o0) {
            (0b10, 0, 0) => Opcode::STXR,
            (0b10, 1, 0) => Opcode::LDXR,
            (0b11, 0, 0) => Opcode::STXR,
            (0b11, 1, 0) => Opcode::LDXR,
            (0b10, 0, 1) => Opcode::STLXR,
            (0b10, 1, 1) => Opcode::LDAXR,
            (0b11, 0, 1) => Opcode::STLXR,
            (0b11, 1, 1) => Opcode::LDAXR,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        if l == 0 {
            insn.operands.push(Operand::register(Register::w(rs)));
        }

        if size >= 2 {
            insn.operands.push(Operand::register(Register::x(rt)));
        } else {
            insn.operands.push(Operand::register(Register::w(rt)));
        }

        insn.operands.push(Operand::memory_base(Register::x(rn)));

        insn
    }

    fn decode_load_register_literal(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let opc = (raw >> 30) & 0x3;
        let v = (raw >> 26) & 1;
        let imm19 = ((raw >> 5) & 0x7FFFF) as i32;
        let imm = if (imm19 & 0x40000) != 0 {
            (imm19 | !0x7FFFF) << 2
        } else {
            imm19 << 2
        };
        let rt = (raw & 0x1F) as u8;

        if v == 0 {
            insn = insn.with_opcode(Opcode::LDR);
            if opc == 0 {
                insn.operands.push(Operand::register(Register::w(rt)));
            } else if opc == 1 {
                insn.operands.push(Operand::register(Register::x(rt)));
            } else if opc == 2 {
                insn = insn.with_opcode(Opcode::LDRSW);
                insn.operands.push(Operand::register(Register::x(rt)));
            }
        } else {
            insn = insn.with_opcode(Opcode::LDR);
            insn.operands.push(Operand::register(Register::v(rt, opc as u8)));
        }

        insn.operands.push(Operand::pc_relative(imm));

        insn
    }

    fn decode_load_store_no_alloc_pair(&self, insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        self.decode_load_store_pair(insn, raw)
    }

    fn decode_load_store_pair(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let opc = (raw >> 30) & 0x3;
        let v = (raw >> 26) & 1;
        let l = (raw >> 22) & 1;
        let imm7 = ((raw >> 15) & 0x7F) as i32;
        let imm = if (imm7 & 0x40) != 0 {
            imm7 | !0x7F
        } else {
            imm7
        };
        let rt2 = ((raw >> 10) & 0x1F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rt = (raw & 0x1F) as u8;

        let scale = if v == 0 { 2 + (opc >> 1) } else { 2 + opc };
        let offset = imm << scale;

        insn = insn.with_opcode(if l == 0 { Opcode::STP } else { Opcode::LDP });

        if v == 0 {
            if (opc & 0x1) == 0 {
                insn.operands.push(Operand::register(Register::w(rt)));
                insn.operands.push(Operand::register(Register::w(rt2)));
            } else {
                insn.operands.push(Operand::register(Register::x(rt)));
                insn.operands.push(Operand::register(Register::x(rt2)));
            }
        } else {
            insn.operands.push(Operand::register(Register::v(rt, opc as u8)));
            insn.operands.push(Operand::register(Register::v(rt2, opc as u8)));
        }

        insn.operands.push(Operand::memory_offset(Register::x(rn), offset as i64));

        insn
    }

    fn decode_load_store_reg_unscaled(&self, insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        self.decode_load_store_reg_common(insn, raw, false)
    }

    fn decode_load_store_reg_post_indexed(&self, insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        self.decode_load_store_reg_common(insn, raw, false)
    }

    fn decode_load_store_reg_pre_indexed(&self, insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        self.decode_load_store_reg_common(insn, raw, false)
    }

    fn decode_load_store_reg_offset(&self, insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        self.decode_load_store_reg_common(insn, raw, true)
    }

    fn decode_load_store_reg_unsigned_imm(&self, insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        self.decode_load_store_reg_common(insn, raw, false)
    }

    fn decode_load_store_reg_common(&self, mut insn: Arm64Instruction, raw: u32, has_reg_offset: bool) -> Arm64Instruction {
        let size = (raw >> 30) & 0x3;
        let v = (raw >> 26) & 1;
        let opc = (raw >> 22) & 0x3;
        let imm12 = ((raw >> 10) & 0xFFF) as i64;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rt = (raw & 0x1F) as u8;

        let is_load = (opc & 0x1) == 1 || opc == 0x2;
        let is_signed = opc >= 2;

        let opcode = if v == 0 {
            match (size, is_load, is_signed) {
                (0, false, _) => Opcode::STRB,
                (0, true, false) => Opcode::LDRB,
                (0, true, true) => Opcode::LDRSB,
                (1, false, _) => Opcode::STRH,
                (1, true, false) => Opcode::LDRH,
                (1, true, true) => Opcode::LDRSH,
                (2, false, _) => Opcode::STR,
                (2, true, false) => Opcode::LDR,
                (2, true, true) => Opcode::LDRSW,
                (3, false, _) => Opcode::STR,
                (3, true, _) => Opcode::LDR,
                _ => Opcode::Unknown,
            }
        } else {
            if is_load { Opcode::LDR } else { Opcode::STR }
        };

        insn = insn.with_opcode(opcode);

        if v == 0 {
            if size == 3 || (size == 2 && is_signed) {
                insn.operands.push(Operand::register(Register::x(rt)));
            } else {
                insn.operands.push(Operand::register(Register::w(rt)));
            }
        } else {
            insn.operands.push(Operand::register(Register::v(rt, size as u8)));
        }

        let scale = if v == 0 { size } else { 2 + size };
        let offset = imm12 << scale;

        insn.operands.push(Operand::memory_offset(Register::x(rn), offset));

        insn
    }

    fn decode_data_processing_reg(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let op0 = (raw >> 30) & 1;
        let op1 = (raw >> 28) & 1;
        let op2 = (raw >> 21) & 0xF;

        if op1 == 0 && (op2 & 0x8) == 0 {
            return self.decode_logical_shifted_reg(insn, raw);
        }

        if op1 == 0 && (op2 & 0x9) == 0x8 {
            return self.decode_add_sub_shifted_reg(insn, raw);
        }

        if op1 == 0 && (op2 & 0x9) == 0x9 {
            return self.decode_add_sub_extended_reg(insn, raw);
        }

        if op1 == 1 && op2 == 0 {
            return self.decode_adc_sbc(insn, raw);
        }

        if op1 == 1 && (op2 & 0xE) == 0x2 {
            return self.decode_conditional_compare(insn, raw);
        }

        if op1 == 1 && (op2 & 0xE) == 0x4 {
            return self.decode_conditional_select(insn, raw);
        }

        if op1 == 1 && (op2 & 0x8) == 0x8 {
            return self.decode_data_processing_3source(insn, raw);
        }

        insn
    }

    fn decode_logical_shifted_reg(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let opc = (raw >> 29) & 0x3;
        let shift = (raw >> 22) & 0x3;
        let n = (raw >> 21) & 1;
        let rm = ((raw >> 16) & 0x1F) as u8;
        let imm6 = ((raw >> 10) & 0x3F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        let opcode = match (opc, n) {
            (0b00, 0) => Opcode::AND,
            (0b00, 1) => Opcode::BIC,
            (0b01, 0) => Opcode::ORR,
            (0b01, 1) => Opcode::ORN,
            (0b10, 0) => Opcode::EOR,
            (0b10, 1) => Opcode::EON,
            (0b11, 0) => Opcode::ANDS,
            (0b11, 1) => Opcode::BICS,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        let shift_type = match shift {
            0 => ShiftType::LSL,
            1 => ShiftType::LSR,
            2 => ShiftType::ASR,
            3 => ShiftType::ROR,
            _ => ShiftType::LSL,
        };

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rd)));
            insn.operands.push(Operand::register(Register::x(rn)));
            insn.operands.push(Operand::register_shifted(Register::x(rm), shift_type, imm6));
        } else {
            insn.operands.push(Operand::register(Register::w(rd)));
            insn.operands.push(Operand::register(Register::w(rn)));
            insn.operands.push(Operand::register_shifted(Register::w(rm), shift_type, imm6));
        }

        if opc == 0b01 && n == 0 && rn == 31 && imm6 == 0 {
            insn = insn.with_opcode(Opcode::MOV);
            insn.operands.remove(1);
            insn.operands[1] = Operand::register(if sf == 1 { Register::x(rm) } else { Register::w(rm) });
        }

        if opc == 0b01 && n == 1 && rn == 31 {
            insn = insn.with_opcode(Opcode::MVN);
            insn.operands.remove(1);
        }

        if rd == 31 && opc == 0b11 && n == 0 {
            insn = insn.with_opcode(Opcode::TST);
            insn.operands.remove(0);
        }

        insn
    }

    fn decode_add_sub_shifted_reg(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let op = (raw >> 30) & 1;
        let s = (raw >> 29) & 1;
        let shift = (raw >> 22) & 0x3;
        let rm = ((raw >> 16) & 0x1F) as u8;
        let imm6 = ((raw >> 10) & 0x3F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        let opcode = match (op, s) {
            (0, 0) => Opcode::ADD,
            (0, 1) => Opcode::ADDS,
            (1, 0) => Opcode::SUB,
            (1, 1) => Opcode::SUBS,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        let shift_type = match shift {
            0 => ShiftType::LSL,
            1 => ShiftType::LSR,
            2 => ShiftType::ASR,
            _ => ShiftType::LSL,
        };

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rd)));
            insn.operands.push(Operand::register(Register::x(rn)));
            insn.operands.push(Operand::register_shifted(Register::x(rm), shift_type, imm6));
        } else {
            insn.operands.push(Operand::register(Register::w(rd)));
            insn.operands.push(Operand::register(Register::w(rn)));
            insn.operands.push(Operand::register_shifted(Register::w(rm), shift_type, imm6));
        }

        if rd == 31 && s == 1 {
            insn = insn.with_opcode(if op == 1 { Opcode::CMP } else { Opcode::CMN });
            insn.operands.remove(0);
        }

        if rn == 31 && op == 1 && s == 0 {
            insn = insn.with_opcode(Opcode::NEG);
            insn.operands.remove(1);
        }

        if rn == 31 && op == 1 && s == 1 {
            insn = insn.with_opcode(Opcode::NEGS);
            insn.operands.remove(1);
        }

        insn
    }

    fn decode_add_sub_extended_reg(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let op = (raw >> 30) & 1;
        let s = (raw >> 29) & 1;
        let rm = ((raw >> 16) & 0x1F) as u8;
        let option = (raw >> 13) & 0x7;
        let imm3 = ((raw >> 10) & 0x7) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        let opcode = match (op, s) {
            (0, 0) => Opcode::ADD,
            (0, 1) => Opcode::ADDS,
            (1, 0) => Opcode::SUB,
            (1, 1) => Opcode::SUBS,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        let extend_type = match option {
            0 => ExtendType::UXTB,
            1 => ExtendType::UXTH,
            2 => ExtendType::UXTW,
            3 => ExtendType::UXTX,
            4 => ExtendType::SXTB,
            5 => ExtendType::SXTH,
            6 => ExtendType::SXTW,
            7 => ExtendType::SXTX,
            _ => ExtendType::UXTX,
        };

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rd)));
            insn.operands.push(Operand::register(Register::x(rn)));
            if option == 3 || option == 7 {
                insn.operands.push(Operand::register_extended(Register::x(rm), extend_type, imm3));
            } else {
                insn.operands.push(Operand::register_extended(Register::w(rm), extend_type, imm3));
            }
        } else {
            insn.operands.push(Operand::register(Register::w(rd)));
            insn.operands.push(Operand::register(Register::w(rn)));
            insn.operands.push(Operand::register_extended(Register::w(rm), extend_type, imm3));
        }

        insn
    }

    fn decode_adc_sbc(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let op = (raw >> 30) & 1;
        let s = (raw >> 29) & 1;
        let rm = ((raw >> 16) & 0x1F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        let opcode = match (op, s) {
            (0, 0) => Opcode::ADC,
            (0, 1) => Opcode::ADCS,
            (1, 0) => Opcode::SBC,
            (1, 1) => Opcode::SBCS,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rd)));
            insn.operands.push(Operand::register(Register::x(rn)));
            insn.operands.push(Operand::register(Register::x(rm)));
        } else {
            insn.operands.push(Operand::register(Register::w(rd)));
            insn.operands.push(Operand::register(Register::w(rn)));
            insn.operands.push(Operand::register(Register::w(rm)));
        }

        insn
    }

    fn decode_conditional_compare(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let op = (raw >> 30) & 1;
        let s = (raw >> 29) & 1;
        let rm = ((raw >> 16) & 0x1F) as u8;
        let cond = ((raw >> 12) & 0xF) as u8;
        let o2 = (raw >> 10) & 1;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let o3 = (raw >> 4) & 1;
        let nzcv = (raw & 0xF) as u8;

        let opcode = if op == 0 { Opcode::CCMN } else { Opcode::CCMP };
        insn = insn.with_opcode(opcode);

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rn)));
            if o2 == 0 {
                insn.operands.push(Operand::register(Register::x(rm)));
            } else {
                insn.operands.push(Operand::immediate(rm as i64));
            }
        } else {
            insn.operands.push(Operand::register(Register::w(rn)));
            if o2 == 0 {
                insn.operands.push(Operand::register(Register::w(rm)));
            } else {
                insn.operands.push(Operand::immediate(rm as i64));
            }
        }
        insn.operands.push(Operand::immediate(nzcv as i64));
        insn = insn.with_condition(Condition::from_code(cond));

        insn
    }

    fn decode_conditional_select(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let op = (raw >> 30) & 1;
        let s = (raw >> 29) & 1;
        let rm = ((raw >> 16) & 0x1F) as u8;
        let cond = ((raw >> 12) & 0xF) as u8;
        let op2 = (raw >> 10) & 0x3;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        let opcode = match (op, op2) {
            (0, 0) => Opcode::CSEL,
            (0, 1) => Opcode::CSINC,
            (1, 0) => Opcode::CSINV,
            (1, 1) => Opcode::CSNEG,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rd)));
            insn.operands.push(Operand::register(Register::x(rn)));
            insn.operands.push(Operand::register(Register::x(rm)));
        } else {
            insn.operands.push(Operand::register(Register::w(rd)));
            insn.operands.push(Operand::register(Register::w(rn)));
            insn.operands.push(Operand::register(Register::w(rm)));
        }
        insn = insn.with_condition(Condition::from_code(cond));

        if op == 0 && op2 == 1 && rn == 31 && rm == 31 {
            insn = insn.with_opcode(Opcode::CSET);
            insn.operands.truncate(1);
            insn = insn.with_condition(Condition::from_code(cond ^ 1));
        }

        if op == 1 && op2 == 0 && rn == 31 && rm == 31 {
            insn = insn.with_opcode(Opcode::CSETM);
            insn.operands.truncate(1);
            insn = insn.with_condition(Condition::from_code(cond ^ 1));
        }

        if op == 0 && op2 == 1 && rn == rm && rn != 31 {
            insn = insn.with_opcode(Opcode::CINC);
            insn.operands.truncate(2);
            insn = insn.with_condition(Condition::from_code(cond ^ 1));
        }

        if op == 1 && op2 == 0 && rn == rm && rn != 31 {
            insn = insn.with_opcode(Opcode::CINV);
            insn.operands.truncate(2);
            insn = insn.with_condition(Condition::from_code(cond ^ 1));
        }

        if op == 1 && op2 == 1 && rn == rm && rn != 31 {
            insn = insn.with_opcode(Opcode::CNEG);
            insn.operands.truncate(2);
            insn = insn.with_condition(Condition::from_code(cond ^ 1));
        }

        insn
    }

    fn decode_data_processing_3source(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        let sf = (raw >> 31) & 1;
        let op54 = (raw >> 29) & 0x3;
        let op31 = (raw >> 21) & 0x7;
        let rm = ((raw >> 16) & 0x1F) as u8;
        let o0 = (raw >> 15) & 1;
        let ra = ((raw >> 10) & 0x1F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rd = (raw & 0x1F) as u8;

        let opcode = match (sf, op54, op31, o0) {
            (_, 0, 0, 0) => Opcode::MADD,
            (_, 0, 0, 1) => Opcode::MSUB,
            (1, 0, 1, 0) => Opcode::SMADDL,
            (1, 0, 1, 1) => Opcode::SMSUBL,
            (1, 0, 2, 0) => Opcode::SMULH,
            (1, 0, 5, 0) => Opcode::UMADDL,
            (1, 0, 5, 1) => Opcode::UMSUBL,
            (1, 0, 6, 0) => Opcode::UMULH,
            _ => Opcode::Unknown,
        };

        insn = insn.with_opcode(opcode);

        if sf == 1 {
            insn.operands.push(Operand::register(Register::x(rd)));
            if op31 == 1 || op31 == 5 {
                insn.operands.push(Operand::register(Register::w(rn)));
                insn.operands.push(Operand::register(Register::w(rm)));
            } else {
                insn.operands.push(Operand::register(Register::x(rn)));
                insn.operands.push(Operand::register(Register::x(rm)));
            }
            insn.operands.push(Operand::register(Register::x(ra)));
        } else {
            insn.operands.push(Operand::register(Register::w(rd)));
            insn.operands.push(Operand::register(Register::w(rn)));
            insn.operands.push(Operand::register(Register::w(rm)));
            insn.operands.push(Operand::register(Register::w(ra)));
        }

        if o0 == 0 && ra == 31 {
            insn = insn.with_opcode(Opcode::MUL);
            insn.operands.truncate(3);
        }

        if o0 == 1 && ra == 31 {
            insn = insn.with_opcode(Opcode::MNEG);
            insn.operands.truncate(3);
        }

        if op31 == 1 && o0 == 0 && ra == 31 {
            insn = insn.with_opcode(Opcode::SMULL);
            insn.operands.truncate(3);
        }

        if op31 == 5 && o0 == 0 && ra == 31 {
            insn = insn.with_opcode(Opcode::UMULL);
            insn.operands.truncate(3);
        }

        insn
    }

    fn decode_simd_fp(&self, mut insn: Arm64Instruction, raw: u32) -> Arm64Instruction {
        insn = insn.with_opcode(Opcode::SIMD(0));
        insn
    }
}

impl Default for Arm64Decoder {
    fn default() -> Self {
        Self::new()
    }
}
