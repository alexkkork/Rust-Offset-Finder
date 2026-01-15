// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use std::sync::Arc;

pub struct MethodAnalyzer {
    reader: Arc<dyn MemoryReader>,
}

impl MethodAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn analyze(&self, addr: Address) -> Option<MethodInfo> {
        let bytes = self.reader.read_bytes(addr, 256).ok()?;

        let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        if (first_insn & 0x7F800000) != 0x29000000 && (first_insn & 0x7F800000) != 0x6D000000 {
            return None;
        }

        let mut info = MethodInfo {
            address: addr,
            estimated_size: 0,
            argument_count: 0,
            return_type: ReturnType::Unknown,
            is_virtual: false,
            is_static: false,
            calls_count: 0,
            has_try_catch: false,
            stack_size: 0,
        };

        let mut i = 0;
        while i < bytes.len() - 4 {
            let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

            if (insn & 0xFFFFFC1F) == 0xD65F0000 {
                info.estimated_size = i + 4;
                break;
            }

            if (insn & 0xFC000000) == 0x94000000 {
                info.calls_count += 1;
            }

            if (insn & 0xFFC00000) == 0xF9400000 {
                let imm = ((insn >> 10) & 0xFFF) as usize * 8;
                if imm > info.stack_size {
                    info.stack_size = imm;
                }
            }

            i += 4;
        }

        if info.estimated_size == 0 {
            info.estimated_size = bytes.len();
        }

        info.argument_count = self.estimate_argument_count(&bytes);
        info.return_type = self.analyze_return_type(&bytes);

        Some(info)
    }

    fn estimate_argument_count(&self, bytes: &[u8]) -> usize {
        let mut max_arg_reg = 0;

        for i in (0..bytes.len().min(64) - 4).step_by(4) {
            let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

            if (insn & 0xFFC00000) == 0xF9000000 || (insn & 0xFFC00000) == 0xB9000000 {
                let rn = ((insn >> 5) & 0x1F) as usize;
                if rn <= 7 && rn > max_arg_reg {
                    max_arg_reg = rn;
                }
            }

            if (insn & 0x7FE00000) == 0x2A000000 || (insn & 0x7FE00000) == 0xAA000000 {
                let rm = ((insn >> 16) & 0x1F) as usize;
                if rm <= 7 && rm > max_arg_reg {
                    max_arg_reg = rm;
                }
            }
        }

        max_arg_reg + 1
    }

    fn analyze_return_type(&self, bytes: &[u8]) -> ReturnType {
        for i in (bytes.len().saturating_sub(32)..bytes.len() - 4).step_by(4) {
            let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

            if (insn & 0xFFFFFC1F) == 0xD65F0000 {
                if i >= 4 {
                    let prev_insn = u32::from_le_bytes([
                        bytes[i - 4], bytes[i - 3], bytes[i - 2], bytes[i - 1]
                    ]);

                    let rd = (prev_insn & 0x1F) as usize;
                    if rd == 0 {
                        if (prev_insn & 0xFFC00000) == 0xF9400000 {
                            return ReturnType::Pointer;
                        }
                        if (prev_insn & 0xFFC00000) == 0xB9400000 {
                            return ReturnType::Int32;
                        }
                        if (prev_insn & 0x7F000000) == 0x52000000 {
                            return ReturnType::Int32;
                        }
                        if (prev_insn & 0x7FE00000) == 0x2A000000 {
                            return ReturnType::Int32;
                        }
                        if (prev_insn & 0x7FE00000) == 0xAA000000 {
                            return ReturnType::Int64;
                        }
                    }
                }

                break;
            }
        }

        ReturnType::Unknown
    }

    pub fn find_calls_to(&self, target: Address, start: Address, end: Address) -> Vec<Address> {
        let mut callers = Vec::new();
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                for i in (0..bytes.len() - 4).step_by(4) {
                    let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

                    if (insn & 0xFC000000) == 0x94000000 {
                        let offset = ((insn & 0x03FFFFFF) as i32) << 6 >> 6;
                        let call_addr = current + i as u64;
                        let dest = (call_addr.as_u64() as i64 + (offset as i64 * 4)) as u64;

                        if dest == target.as_u64() {
                            callers.push(call_addr);
                        }
                    }
                }
            }

            current = current + 4000;
        }

        callers
    }
}

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub address: Address,
    pub estimated_size: usize,
    pub argument_count: usize,
    pub return_type: ReturnType,
    pub is_virtual: bool,
    pub is_static: bool,
    pub calls_count: usize,
    pub has_try_catch: bool,
    pub stack_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnType {
    Void,
    Int32,
    Int64,
    Float,
    Double,
    Pointer,
    Unknown,
}
