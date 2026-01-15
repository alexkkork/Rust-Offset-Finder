// Wed Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use crate::analysis::disasm::{DecodedInstruction, InstructionCategory, Operand, ShiftType};
use std::sync::Arc;

pub struct InstructionIterator {
    reader: Arc<dyn MemoryReader>,
    current: Address,
    end: Option<Address>,
    max_count: usize,
    count: usize,
    stop_on_return: bool,
    stop_on_branch: bool,
}

impl InstructionIterator {
    pub fn new(reader: Arc<dyn MemoryReader>, start: Address, max_count: usize) -> Self {
        Self {
            reader,
            current: start,
            end: None,
            max_count,
            count: 0,
            stop_on_return: false,
            stop_on_branch: false,
        }
    }

    pub fn with_end(mut self, end: Address) -> Self {
        self.end = Some(end);
        self
    }

    pub fn stop_on_return(mut self) -> Self {
        self.stop_on_return = true;
        self
    }

    pub fn stop_on_branch(mut self) -> Self {
        self.stop_on_branch = true;
        self
    }

    pub fn current_address(&self) -> Address {
        self.current
    }

    pub fn instructions_read(&self) -> usize {
        self.count
    }

    fn decode_next(&mut self) -> Result<DecodedInstruction, MemoryError> {
        let bytes = self.reader.read_bytes(self.current, 4)?;
        let raw = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let (mnemonic, operands, operand_str, category) = self.decode_arm64(raw);

        Ok(DecodedInstruction {
            address: self.current,
            bytes: bytes.to_vec(),
            size: 4,
            mnemonic,
            operands,
            operand_str,
            raw,
            category,
        })
    }

    fn decode_arm64(&self, raw: u32) -> (String, Vec<Operand>, String, InstructionCategory) {
        let op0 = (raw >> 25) & 0xF;

        match op0 {
            0b1010 | 0b1011 => self.decode_branch(raw),
            _ => {
                let operand_str = format!("0x{:08X}", raw);
                ("DATA".to_string(), vec![], operand_str, InstructionCategory::Unknown)
            }
        }
    }

    fn decode_branch(&self, raw: u32) -> (String, Vec<Operand>, String, InstructionCategory) {
        let op0 = (raw >> 29) & 0x7;

        match op0 {
            0b000 | 0b100 => {
                let is_link = (raw >> 31) & 1 == 1;
                let imm26 = raw & 0x03FFFFFF;

                let offset = if imm26 & 0x02000000 != 0 {
                    ((imm26 | 0xFC000000) as i32) * 4
                } else {
                    (imm26 as i32) * 4
                };

                let target = (self.current.as_u64() as i64 + offset as i64) as u64;
                let mnemonic = if is_link { "BL" } else { "B" };
                let category = if is_link { InstructionCategory::Call } else { InstructionCategory::Branch };

                let operand_str = format!("0x{:X}", target);
                let operands = vec![Operand::Address(Address::new(target))];

                (mnemonic.to_string(), operands, operand_str, category)
            }
            0b110 => {
                let opc = (raw >> 21) & 0x7;
                let rn = ((raw >> 5) & 0x1F) as u8;

                let (mnemonic, category) = match opc {
                    0b000 => ("BR", InstructionCategory::Branch),
                    0b001 => ("BLR", InstructionCategory::Call),
                    0b010 => ("RET", InstructionCategory::Return),
                    _ => ("UNKNOWN", InstructionCategory::Unknown),
                };

                let operand_str = format!("X{}", rn);
                let operands = vec![Operand::Register(rn)];

                (mnemonic.to_string(), operands, operand_str, category)
            }
            _ => {
                let operand_str = format!("0x{:08X}", raw);
                ("UNKNOWN".to_string(), vec![], operand_str, InstructionCategory::Unknown)
            }
        }
    }
}

impl Iterator for InstructionIterator {
    type Item = Result<DecodedInstruction, MemoryError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.max_count {
            return None;
        }

        if let Some(end) = self.end {
            if self.current >= end {
                return None;
            }
        }

        match self.decode_next() {
            Ok(instr) => {
                self.count += 1;
                self.current = self.current + 4;

                let should_stop = match instr.category {
                    InstructionCategory::Return if self.stop_on_return => true,
                    InstructionCategory::Branch if self.stop_on_branch => true,
                    _ => false,
                };

                if should_stop {
                    self.max_count = self.count;
                }

                Some(Ok(instr))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct LinearIterator {
    reader: Arc<dyn MemoryReader>,
    current: Address,
    end: Address,
}

impl LinearIterator {
    pub fn new(reader: Arc<dyn MemoryReader>, start: Address, end: Address) -> Self {
        Self {
            reader,
            current: start,
            end,
        }
    }

    pub fn remaining(&self) -> u64 {
        if self.current >= self.end {
            0
        } else {
            self.end.as_u64() - self.current.as_u64()
        }
    }
}

impl Iterator for LinearIterator {
    type Item = Result<(Address, u32), MemoryError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }

        match self.reader.read_bytes(self.current, 4) {
            Ok(bytes) => {
                let raw = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                let addr = self.current;
                self.current = self.current + 4;
                Some(Ok((addr, raw)))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct FunctionIterator {
    inner: InstructionIterator,
    finished: bool,
}

impl FunctionIterator {
    pub fn new(reader: Arc<dyn MemoryReader>, entry: Address) -> Self {
        Self {
            inner: InstructionIterator::new(reader, entry, 10000).stop_on_return(),
            finished: false,
        }
    }

    pub fn entry_point(&self) -> Address {
        self.inner.current
    }
}

impl Iterator for FunctionIterator {
    type Item = Result<DecodedInstruction, MemoryError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match self.inner.next() {
            Some(Ok(instr)) => {
                if instr.category == InstructionCategory::Return {
                    self.finished = true;
                }
                Some(Ok(instr))
            }
            other => other,
        }
    }
}
