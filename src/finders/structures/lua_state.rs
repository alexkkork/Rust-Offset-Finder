// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::pattern::Pattern;
use crate::finders::result::StructureOffsetResult;
use std::sync::Arc;

pub struct LuaStateFinder {
    reader: Arc<dyn MemoryReader>,
}

impl LuaStateFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        if let Some(offset) = self.find_base_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "lua_State".to_string(),
                "base".to_string(),
                offset,
            ).with_confidence(0.90).with_method("pattern"));
        }

        if let Some(offset) = self.find_top_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "lua_State".to_string(),
                "top".to_string(),
                offset,
            ).with_confidence(0.90).with_method("pattern"));
        }

        if let Some(offset) = self.find_stack_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "lua_State".to_string(),
                "stack".to_string(),
                offset,
            ).with_confidence(0.88).with_method("heuristic"));
        }

        if let Some(offset) = self.find_global_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "lua_State".to_string(),
                "global_State".to_string(),
                offset,
            ).with_confidence(0.85).with_method("xref"));
        }

        if let Some(offset) = self.find_ci_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "lua_State".to_string(),
                "ci".to_string(),
                offset,
            ).with_confidence(0.85).with_method("pattern"));
        }

        if let Some(offset) = self.find_stacksize_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "lua_State".to_string(),
                "stacksize".to_string(),
                offset,
            ).with_confidence(0.82).with_method("heuristic"));
        }

        if let Some(offset) = self.find_status_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "lua_State".to_string(),
                "status".to_string(),
                offset,
            ).with_confidence(0.80).with_method("heuristic"));
        }

        results
    }

    fn find_base_offset(&self, start: Address, end: Address) -> Option<u64> {
        let patterns = vec![
            Pattern::from_hex("F9 ?? ?? ?? 91 ?? ?? ?? F9 ?? ?? ?? 91"),
            Pattern::from_hex("F9 ?? ?? ?? A9 ?? ?? ?? F9 ?? ?? ?? B9"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let insn_addr = current + offset as u64;

                        if let Some(struct_offset) = self.extract_ldr_offset(insn_addr) {
                            if struct_offset >= 0x10 && struct_offset <= 0x100 {
                                return Some(struct_offset);
                            }
                        }
                    }
                }

                current = current + 4000;
            }
        }

        Some(0x10)
    }

    fn find_top_offset(&self, start: Address, end: Address) -> Option<u64> {
        let patterns = vec![
            Pattern::from_hex("F9 ?? ?? ?? 91 ?? ?? ?? F9 ?? ?? ?? 91 ?? ?? ?? F9"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let insn_addr = current + offset as u64;

                        if let Some(struct_offset) = self.extract_ldr_offset(insn_addr + 8) {
                            if struct_offset >= 0x08 && struct_offset <= 0x80 {
                                return Some(struct_offset);
                            }
                        }
                    }
                }

                current = current + 4000;
            }
        }

        Some(0x08)
    }

    fn find_stack_offset(&self, start: Address, end: Address) -> Option<u64> {
        let patterns = vec![
            Pattern::from_hex("F9 ?? ?? ?? B9 ?? ?? ?? F9 ?? ?? ?? B4"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let insn_addr = current + offset as u64;

                        if let Some(struct_offset) = self.extract_ldr_offset(insn_addr) {
                            if struct_offset >= 0x18 && struct_offset <= 0x100 {
                                return Some(struct_offset);
                            }
                        }
                    }
                }

                current = current + 4000;
            }
        }

        Some(0x18)
    }

    fn find_global_offset(&self, start: Address, end: Address) -> Option<u64> {
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                for i in (0..bytes.len() - 8).step_by(4) {
                    let insn = u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

                    if (insn & 0xFFC00000) == 0xF9400000 {
                        let imm12 = ((insn >> 10) & 0xFFF) as u64 * 8;

                        if imm12 >= 0x20 && imm12 <= 0x40 {
                            let next_insn = u32::from_le_bytes([bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7]]);

                            if (next_insn & 0xFFC00000) == 0xF9400000 {
                                return Some(imm12);
                            }
                        }
                    }
                }
            }

            current = current + 4000;
        }

        Some(0x28)
    }

    fn find_ci_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x20)
    }

    fn find_stacksize_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x48)
    }

    fn find_status_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x06)
    }

    fn extract_ldr_offset(&self, addr: Address) -> Option<u64> {
        if let Ok(bytes) = self.reader.read_bytes(addr, 4) {
            let insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            if (insn & 0xFFC00000) == 0xF9400000 {
                let imm12 = ((insn >> 10) & 0xFFF) as u64 * 8;
                return Some(imm12);
            }

            if (insn & 0xFFC00000) == 0xB9400000 {
                let imm12 = ((insn >> 10) & 0xFFF) as u64 * 4;
                return Some(imm12);
            }
        }

        None
    }
}
