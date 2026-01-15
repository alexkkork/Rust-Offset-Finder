// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use std::sync::Arc;
use std::collections::HashMap;

pub struct Disassembler {
    reader: Arc<dyn MemoryReader>,
    cache: HashMap<u64, DisassembledInstruction>,
}

impl Disassembler {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            cache: HashMap::new(),
        }
    }

    pub fn disassemble(&self, addr: Address) -> Result<DisassembledInstruction, MemoryError> {
        let bytes = self.reader.read_bytes(addr, 4)?;
        let raw = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let (mnemonic, operands) = self.decode_instruction(raw);

        Ok(DisassembledInstruction {
            address: addr,
            bytes: bytes.to_vec(),
            mnemonic,
            operands,
            raw,
            size: 4,
        })
    }

    pub fn disassemble_function(&self, start: Address, max_bytes: usize) -> Result<Vec<DisassembledInstruction>, MemoryError> {
        let mut instructions = Vec::new();
        let mut offset = 0u64;

        while offset < max_bytes as u64 {
            let addr = start + offset;
            let instr = self.disassemble(addr)?;

            let is_ret = instr.mnemonic == "RET";
            instructions.push(instr);

            if is_ret {
                break;
            }

            offset += 4;
        }

        Ok(instructions)
    }

    pub fn disassemble_range(&self, start: Address, end: Address) -> Result<Vec<DisassembledInstruction>, MemoryError> {
        let mut instructions = Vec::new();
        let mut current = start;

        while current < end {
            let instr = self.disassemble(current)?;
            instructions.push(instr);
            current = current + 4;
        }

        Ok(instructions)
    }

    fn decode_instruction(&self, raw: u32) -> (String, String) {
        let op = raw >> 24;

        match op {
            0x94 => {
                let imm26 = raw & 0x03FFFFFF;
                ("BL".to_string(), format!("#{}", imm26))
            }
            0x14 => {
                let imm26 = raw & 0x03FFFFFF;
                ("B".to_string(), format!("#{}", imm26))
            }
            0xD6 => {
                if (raw & 0xFFFFFC1F) == 0xD65F0000 {
                    let rn = (raw >> 5) & 0x1F;
                    ("RET".to_string(), format!("X{}", rn))
                } else if (raw & 0xFFFFFC1F) == 0xD63F0000 {
                    let rn = (raw >> 5) & 0x1F;
                    ("BLR".to_string(), format!("X{}", rn))
                } else if (raw & 0xFFFFFC1F) == 0xD61F0000 {
                    let rn = (raw >> 5) & 0x1F;
                    ("BR".to_string(), format!("X{}", rn))
                } else {
                    ("UNKNOWN".to_string(), format!("0x{:08X}", raw))
                }
            }
            0xA9 => {
                let rt = raw & 0x1F;
                let rn = (raw >> 5) & 0x1F;
                let rt2 = (raw >> 10) & 0x1F;
                let imm7 = ((raw >> 15) & 0x7F) as i8;
                ("STP".to_string(), format!("X{}, X{}, [X{}, #{}]", rt, rt2, rn, imm7 * 8))
            }
            0xA8 => {
                let rt = raw & 0x1F;
                let rn = (raw >> 5) & 0x1F;
                let rt2 = (raw >> 10) & 0x1F;
                let imm7 = ((raw >> 15) & 0x7F) as i8;
                ("LDP".to_string(), format!("X{}, X{}, [X{}, #{}]", rt, rt2, rn, imm7 * 8))
            }
            0xF9 => {
                let rt = raw & 0x1F;
                let rn = (raw >> 5) & 0x1F;
                let imm12 = ((raw >> 10) & 0xFFF) * 8;
                if (raw >> 22) & 0x3 == 1 {
                    ("LDR".to_string(), format!("X{}, [X{}, #{}]", rt, rn, imm12))
                } else {
                    ("STR".to_string(), format!("X{}, [X{}, #{}]", rt, rn, imm12))
                }
            }
            0xD1 => {
                let rd = raw & 0x1F;
                let rn = (raw >> 5) & 0x1F;
                let imm12 = (raw >> 10) & 0xFFF;
                ("SUB".to_string(), format!("X{}, X{}, #{}", rd, rn, imm12))
            }
            0x91 => {
                let rd = raw & 0x1F;
                let rn = (raw >> 5) & 0x1F;
                let imm12 = (raw >> 10) & 0xFFF;
                ("ADD".to_string(), format!("X{}, X{}, #{}", rd, rn, imm12))
            }
            0x90 => {
                let rd = raw & 0x1F;
                ("ADRP".to_string(), format!("X{}, #page", rd))
            }
            0xAA => {
                let rd = raw & 0x1F;
                let rm = (raw >> 16) & 0x1F;
                ("MOV".to_string(), format!("X{}, X{}", rd, rm))
            }
            0xB4 => {
                let rt = raw & 0x1F;
                let imm19 = (raw >> 5) & 0x7FFFF;
                ("CBZ".to_string(), format!("X{}, #{}", rt, imm19))
            }
            0xB5 => {
                let rt = raw & 0x1F;
                let imm19 = (raw >> 5) & 0x7FFFF;
                ("CBNZ".to_string(), format!("X{}, #{}", rt, imm19))
            }
            0xEB => {
                let rd = raw & 0x1F;
                let rn = (raw >> 5) & 0x1F;
                let rm = (raw >> 16) & 0x1F;
                ("SUBS".to_string(), format!("X{}, X{}, X{}", rd, rn, rm))
            }
            0xF1 => {
                let rn = (raw >> 5) & 0x1F;
                let imm12 = (raw >> 10) & 0xFFF;
                ("CMP".to_string(), format!("X{}, #{}", rn, imm12))
            }
            0x54 => {
                let cond = raw & 0xF;
                let imm19 = (raw >> 5) & 0x7FFFF;
                let cond_str = match cond {
                    0x0 => "EQ",
                    0x1 => "NE",
                    0x2 => "CS",
                    0x3 => "CC",
                    0x4 => "MI",
                    0x5 => "PL",
                    0x8 => "HI",
                    0x9 => "LS",
                    0xA => "GE",
                    0xB => "LT",
                    0xC => "GT",
                    0xD => "LE",
                    _ => "??",
                };
                (format!("B.{}", cond_str), format!("#{}", imm19))
            }
            _ => ("UNKNOWN".to_string(), format!("0x{:08X}", raw)),
        }
    }

    pub fn is_call_instruction(&self, instr: &DisassembledInstruction) -> bool {
        instr.mnemonic == "BL" || instr.mnemonic == "BLR"
    }

    pub fn is_branch_instruction(&self, instr: &DisassembledInstruction) -> bool {
        instr.mnemonic.starts_with("B")
    }

    pub fn is_return_instruction(&self, instr: &DisassembledInstruction) -> bool {
        instr.mnemonic == "RET"
    }

    pub fn get_call_target(&self, instr: &DisassembledInstruction) -> Option<Address> {
        if instr.mnemonic == "BL" {
            let imm26 = instr.raw & 0x03FFFFFF;
            let offset = if imm26 & 0x02000000 != 0 {
                ((imm26 | 0xFC000000) as i32) * 4
            } else {
                (imm26 as i32) * 4
            };
            let target = (instr.address.as_u64() as i64 + offset as i64) as u64;
            Some(Address::new(target))
        } else {
            None
        }
    }

    pub fn get_branch_target(&self, instr: &DisassembledInstruction) -> Option<Address> {
        if instr.mnemonic == "B" || instr.mnemonic.starts_with("B.") {
            let imm = if instr.mnemonic == "B" {
                instr.raw & 0x03FFFFFF
            } else {
                (instr.raw >> 5) & 0x7FFFF
            };
            let offset = if instr.mnemonic == "B" {
                if imm & 0x02000000 != 0 {
                    ((imm | 0xFC000000) as i32) * 4
                } else {
                    (imm as i32) * 4
                }
            } else {
                if imm & 0x40000 != 0 {
                    ((imm | 0xFFF80000) as i32) * 4
                } else {
                    (imm as i32) * 4
                }
            };
            let target = (instr.address.as_u64() as i64 + offset as i64) as u64;
            Some(Address::new(target))
        } else if instr.mnemonic == "CBZ" || instr.mnemonic == "CBNZ" {
            let imm19 = (instr.raw >> 5) & 0x7FFFF;
            let offset = if imm19 & 0x40000 != 0 {
                ((imm19 | 0xFFF80000) as i32) * 4
            } else {
                (imm19 as i32) * 4
            };
            let target = (instr.address.as_u64() as i64 + offset as i64) as u64;
            Some(Address::new(target))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct DisassembledInstruction {
    pub address: Address,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub operands: String,
    pub raw: u32,
    pub size: usize,
}

impl DisassembledInstruction {
    pub fn to_string(&self) -> String {
        format!("{:016X}: {} {}", self.address.as_u64(), self.mnemonic, self.operands)
    }

    pub fn is_nop(&self) -> bool {
        self.raw == 0xD503201F
    }
}
