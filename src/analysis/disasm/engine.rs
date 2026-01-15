// Wed Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::disasm::{DecodedInstruction, Operand, InstructionCategory, ShiftType};
use std::sync::Arc;

pub struct DisassemblyEngine {
    reader: Arc<dyn MemoryReader>,
}

impl DisassemblyEngine {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn decode(&self, addr: Address) -> Result<DecodedInstruction, MemoryError> {
        let bytes = self.reader.read_bytes(addr, 4)?;
        let raw = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let (mnemonic, operands, operand_str, category) = self.decode_arm64(raw, addr);

        Ok(DecodedInstruction {
            address: addr,
            bytes: bytes.to_vec(),
            size: 4,
            mnemonic,
            operands,
            operand_str,
            raw,
            category,
        })
    }

    fn decode_arm64(&self, raw: u32, addr: Address) -> (String, Vec<Operand>, String, InstructionCategory) {
        let op0 = (raw >> 25) & 0xF;

        match op0 {
            0b0101 | 0b1101 => self.decode_data_processing_imm(raw),
            0b1010 | 0b1011 => self.decode_branch(raw, addr),
            0b0100 | 0b0110 | 0b1100 | 0b1110 => self.decode_load_store(raw),
            0b0101 => self.decode_data_processing_reg(raw),
            _ => self.decode_fallback(raw),
        }
    }

    fn decode_data_processing_imm(&self, raw: u32) -> (String, Vec<Operand>, String, InstructionCategory) {
        let op0 = (raw >> 23) & 0x7;

        match op0 {
            0b000 | 0b001 => {
                let sf = (raw >> 31) & 1;
                let op = (raw >> 30) & 1;
                let s = (raw >> 29) & 1;
                let rd = (raw & 0x1F) as u8;
                let rn = ((raw >> 5) & 0x1F) as u8;
                let imm12 = (raw >> 10) & 0xFFF;
                let sh = (raw >> 22) & 1;

                let imm = if sh == 1 { imm12 << 12 } else { imm12 };
                let reg_prefix = if sf == 1 { "X" } else { "W" };

                let (mnemonic, category) = match (op, s) {
                    (0, 0) => ("ADD", InstructionCategory::Arithmetic),
                    (0, 1) => ("ADDS", InstructionCategory::Arithmetic),
                    (1, 0) => ("SUB", InstructionCategory::Arithmetic),
                    (1, 1) => ("SUBS", InstructionCategory::Arithmetic),
                    _ => ("UNKNOWN", InstructionCategory::Unknown),
                };

                let operand_str = format!("{}{}, {}{}, #0x{:X}", reg_prefix, rd, reg_prefix, rn, imm);
                let operands = vec![
                    Operand::Register(rd),
                    Operand::Register(rn),
                    Operand::Immediate(imm as i64),
                ];

                (mnemonic.to_string(), operands, operand_str, category)
            }
            0b100 => {
                let rd = (raw & 0x1F) as u8;
                let imm16 = ((raw >> 5) & 0xFFFF) as u16;
                let hw = ((raw >> 21) & 0x3) as u8;
                let sf = (raw >> 31) & 1;
                let opc = (raw >> 29) & 0x3;

                let reg_prefix = if sf == 1 { "X" } else { "W" };
                let shift = hw * 16;

                let mnemonic = match opc {
                    0 => "MOVN",
                    2 => "MOVZ",
                    3 => "MOVK",
                    _ => "UNKNOWN",
                };

                let operand_str = if shift == 0 {
                    format!("{}{}, #0x{:X}", reg_prefix, rd, imm16)
                } else {
                    format!("{}{}, #0x{:X}, LSL #{}", reg_prefix, rd, imm16, shift)
                };

                let operands = vec![
                    Operand::Register(rd),
                    Operand::Immediate(imm16 as i64),
                ];

                (mnemonic.to_string(), operands, operand_str, InstructionCategory::Move)
            }
            0b101 => {
                let rd = (raw & 0x1F) as u8;
                let immhi = ((raw >> 5) & 0x7FFFF) as i64;
                let immlo = ((raw >> 29) & 0x3) as i64;
                let op = (raw >> 31) & 1;

                let imm = (immhi << 2) | immlo;

                let (mnemonic, category) = if op == 0 {
                    ("ADR", InstructionCategory::Arithmetic)
                } else {
                    ("ADRP", InstructionCategory::Arithmetic)
                };

                let operand_str = format!("X{}, #0x{:X}", rd, imm);
                let operands = vec![
                    Operand::Register(rd),
                    Operand::Immediate(imm),
                ];

                (mnemonic.to_string(), operands, operand_str, category)
            }
            _ => self.decode_fallback(raw),
        }
    }

    fn decode_branch(&self, raw: u32, addr: Address) -> (String, Vec<Operand>, String, InstructionCategory) {
        let op0 = (raw >> 29) & 0x7;

        match op0 {
            0b000 | 0b100 => {
                let imm26 = raw & 0x03FFFFFF;
                let is_link = (raw >> 31) & 1 == 1;

                let offset = if imm26 & 0x02000000 != 0 {
                    ((imm26 | 0xFC000000) as i32) * 4
                } else {
                    (imm26 as i32) * 4
                };

                let target = (addr.as_u64() as i64 + offset as i64) as u64;
                let mnemonic = if is_link { "BL" } else { "B" };
                let category = if is_link { InstructionCategory::Call } else { InstructionCategory::Branch };

                let operand_str = format!("0x{:X}", target);
                let operands = vec![Operand::Address(Address::new(target))];

                (mnemonic.to_string(), operands, operand_str, category)
            }
            0b010 => {
                let imm19 = (raw >> 5) & 0x7FFFF;
                let cond = raw & 0xF;

                let offset = if imm19 & 0x40000 != 0 {
                    ((imm19 | 0xFFF80000) as i32) * 4
                } else {
                    (imm19 as i32) * 4
                };

                let target = (addr.as_u64() as i64 + offset as i64) as u64;

                let cond_str = match cond {
                    0x0 => "EQ", 0x1 => "NE", 0x2 => "CS", 0x3 => "CC",
                    0x4 => "MI", 0x5 => "PL", 0x6 => "VS", 0x7 => "VC",
                    0x8 => "HI", 0x9 => "LS", 0xA => "GE", 0xB => "LT",
                    0xC => "GT", 0xD => "LE", 0xE => "AL", _ => "??",
                };

                let mnemonic = format!("B.{}", cond_str);
                let operand_str = format!("0x{:X}", target);
                let operands = vec![
                    Operand::Condition(cond as u8),
                    Operand::Address(Address::new(target)),
                ];

                (mnemonic, operands, operand_str, InstructionCategory::ConditionalBranch)
            }
            0b110 => {
                let opc = (raw >> 21) & 0x7;
                let rn = ((raw >> 5) & 0x1F) as u8;

                let mnemonic = match opc {
                    0b000 => "BR",
                    0b001 => "BLR",
                    0b010 => "RET",
                    _ => "UNKNOWN",
                };

                let category = match opc {
                    0b000 => InstructionCategory::Branch,
                    0b001 => InstructionCategory::Call,
                    0b010 => InstructionCategory::Return,
                    _ => InstructionCategory::Unknown,
                };

                let operand_str = format!("X{}", rn);
                let operands = vec![Operand::Register(rn)];

                (mnemonic.to_string(), operands, operand_str, category)
            }
            0b001 | 0b101 => {
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
                let reg_prefix = if sf == 1 { "X" } else { "W" };
                let mnemonic = if op == 0 { "CBZ" } else { "CBNZ" };

                let operand_str = format!("{}{}, 0x{:X}", reg_prefix, rt, target);
                let operands = vec![
                    Operand::Register(rt),
                    Operand::Address(Address::new(target)),
                ];

                (mnemonic.to_string(), operands, operand_str, InstructionCategory::ConditionalBranch)
            }
            _ => self.decode_fallback(raw),
        }
    }

    fn decode_load_store(&self, raw: u32) -> (String, Vec<Operand>, String, InstructionCategory) {
        let op0 = (raw >> 28) & 0xF;
        let op1 = (raw >> 26) & 0x1;
        let op2 = (raw >> 23) & 0x3;

        let size = (raw >> 30) & 0x3;
        let v = (raw >> 26) & 1;
        let opc = (raw >> 22) & 0x3;
        let rt = (raw & 0x1F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;

        if op1 == 0 && op2 == 0b01 {
            let imm12 = ((raw >> 10) & 0xFFF) as i64;
            let scale = match size { 0 => 1, 1 => 2, 2 => 4, 3 => 8, _ => 1 };
            let offset = imm12 * scale;

            let (mnemonic, category) = match (v, opc) {
                (0, 0) => ("STR", InstructionCategory::Store),
                (0, 1) => ("LDR", InstructionCategory::Load),
                (0, 2) => ("LDRS", InstructionCategory::Load),
                (1, 0) => ("STR", InstructionCategory::Store),
                (1, 1) => ("LDR", InstructionCategory::Load),
                _ => ("UNKNOWN", InstructionCategory::Unknown),
            };

            let reg_prefix = if v == 1 {
                match size { 0 => "B", 1 => "H", 2 => "S", 3 => "D", _ => "?" }
            } else {
                match size { 2 => "W", 3 => "X", _ => "?" }
            };

            let operand_str = if offset == 0 {
                format!("{}{}, [X{}]", reg_prefix, rt, rn)
            } else {
                format!("{}{}, [X{}, #{}]", reg_prefix, rt, rn, offset)
            };

            let operands = vec![
                Operand::Register(rt),
                Operand::Memory { base: rn, offset, index: None, scale: 1 },
            ];

            (mnemonic.to_string(), operands, operand_str, category)
        } else if (raw >> 27) & 0x1F == 0b10100 {
            let opc = (raw >> 22) & 0x3;
            let l = (raw >> 22) & 1;
            let imm7 = ((raw >> 15) & 0x7F) as i8;
            let rt2 = ((raw >> 10) & 0x1F) as u8;

            let offset = (imm7 as i64) * 8;
            let mnemonic = if l == 0 { "STP" } else { "LDP" };
            let category = if l == 0 { InstructionCategory::Store } else { InstructionCategory::Load };

            let operand_str = format!("X{}, X{}, [X{}, #{}]", rt, rt2, rn, offset);
            let operands = vec![
                Operand::Register(rt),
                Operand::Register(rt2),
                Operand::Memory { base: rn, offset, index: None, scale: 1 },
            ];

            (mnemonic.to_string(), operands, operand_str, category)
        } else {
            self.decode_fallback(raw)
        }
    }

    fn decode_data_processing_reg(&self, raw: u32) -> (String, Vec<Operand>, String, InstructionCategory) {
        let sf = (raw >> 31) & 1;
        let opc = (raw >> 29) & 0x3;
        let rd = (raw & 0x1F) as u8;
        let rn = ((raw >> 5) & 0x1F) as u8;
        let rm = ((raw >> 16) & 0x1F) as u8;

        let reg_prefix = if sf == 1 { "X" } else { "W" };

        if (raw >> 21) & 0x7FF == 0b01011000000 {
            let operand_str = format!("{}{}, {}{}", reg_prefix, rd, reg_prefix, rm);
            let operands = vec![Operand::Register(rd), Operand::Register(rm)];
            ("MOV".to_string(), operands, operand_str, InstructionCategory::Move)
        } else {
            let (mnemonic, category) = match opc {
                0b00 => ("AND", InstructionCategory::Logic),
                0b01 => ("ORR", InstructionCategory::Logic),
                0b10 => ("EOR", InstructionCategory::Logic),
                0b11 => ("ANDS", InstructionCategory::Logic),
                _ => ("UNKNOWN", InstructionCategory::Unknown),
            };

            let operand_str = format!("{}{}, {}{}, {}{}", reg_prefix, rd, reg_prefix, rn, reg_prefix, rm);
            let operands = vec![
                Operand::Register(rd),
                Operand::Register(rn),
                Operand::Register(rm),
            ];

            (mnemonic.to_string(), operands, operand_str, category)
        }
    }

    fn decode_fallback(&self, raw: u32) -> (String, Vec<Operand>, String, InstructionCategory) {
        let operand_str = format!("0x{:08X}", raw);
        ("UNKNOWN".to_string(), vec![], operand_str, InstructionCategory::Unknown)
    }
}
