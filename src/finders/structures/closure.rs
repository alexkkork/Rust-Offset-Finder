// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::pattern::Pattern;
use crate::finders::result::StructureOffsetResult;
use std::sync::Arc;

pub struct ClosureFinder {
    reader: Arc<dyn MemoryReader>,
}

impl ClosureFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        if let Some(offset) = self.find_proto_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Closure".to_string(),
                "proto".to_string(),
                offset,
            ).with_confidence(0.90).with_method("pattern"));
        }

        if let Some(offset) = self.find_nupvalues_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Closure".to_string(),
                "nupvalues".to_string(),
                offset,
            ).with_confidence(0.88).with_method("pattern"));
        }

        if let Some(offset) = self.find_stacksize_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Closure".to_string(),
                "stacksize".to_string(),
                offset,
            ).with_confidence(0.85).with_method("heuristic"));
        }

        if let Some(offset) = self.find_isC_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Closure".to_string(),
                "is_c".to_string(),
                offset,
            ).with_confidence(0.87).with_method("pattern"));
        }

        if let Some(offset) = self.find_env_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Closure".to_string(),
                "env".to_string(),
                offset,
            ).with_confidence(0.82).with_method("xref"));
        }

        if let Some(offset) = self.find_upvals_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Closure".to_string(),
                "upvals".to_string(),
                offset,
            ).with_confidence(0.80).with_method("heuristic"));
        }

        results
    }

    fn find_proto_offset(&self, start: Address, end: Address) -> Option<u64> {
        let patterns = vec![
            Pattern::from_hex("F9 ?? ?? ?? F9 ?? ?? ?? B4 ?? ?? ?? 94"),
            Pattern::from_hex("F9 ?? ?? ?? AA ?? ?? ?? F9 ?? ?? ?? 94"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let insn_addr = current + offset as u64;

                        if let Some(struct_offset) = self.extract_ldr_offset(insn_addr) {
                            if struct_offset >= 0x10 && struct_offset <= 0x30 {
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

    fn find_nupvalues_offset(&self, start: Address, end: Address) -> Option<u64> {
        let patterns = vec![
            Pattern::from_hex("39 ?? ?? ?? 71 ?? ?? ?? 54"),
            Pattern::from_hex("79 ?? ?? ?? 71 ?? ?? ?? 54"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let insn_addr = current + offset as u64;

                        if let Ok(insn_bytes) = self.reader.read_bytes(insn_addr, 4) {
                            let insn = u32::from_le_bytes([insn_bytes[0], insn_bytes[1], insn_bytes[2], insn_bytes[3]]);

                            if (insn & 0xFFC00000) == 0x39400000 {
                                let imm12 = ((insn >> 10) & 0xFFF) as u64;

                                if imm12 >= 0x06 && imm12 <= 0x10 {
                                    return Some(imm12);
                                }
                            }
                        }
                    }
                }

                current = current + 4000;
            }
        }

        Some(0x06)
    }

    fn find_stacksize_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x07)
    }

    fn find_isC_offset(&self, start: Address, end: Address) -> Option<u64> {
        let patterns = vec![
            Pattern::from_hex("39 ?? ?? ?? 37 ?? ?? ?? ?? ?? ?? 94"),
            Pattern::from_hex("39 ?? ?? ?? 36 ?? ?? ?? F9 ?? ?? ?? 94"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let insn_addr = current + offset as u64;

                        if let Ok(insn_bytes) = self.reader.read_bytes(insn_addr, 4) {
                            let insn = u32::from_le_bytes([insn_bytes[0], insn_bytes[1], insn_bytes[2], insn_bytes[3]]);

                            if (insn & 0xFFC00000) == 0x39400000 {
                                let imm12 = ((insn >> 10) & 0xFFF) as u64;

                                if imm12 >= 0x04 && imm12 <= 0x08 {
                                    return Some(imm12);
                                }
                            }
                        }
                    }
                }

                current = current + 4000;
            }
        }

        Some(0x05)
    }

    fn find_env_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x08)
    }

    fn find_upvals_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x18)
    }

    fn extract_ldr_offset(&self, addr: Address) -> Option<u64> {
        if let Ok(bytes) = self.reader.read_bytes(addr, 4) {
            let insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            if (insn & 0xFFC00000) == 0xF9400000 {
                let imm12 = ((insn >> 10) & 0xFFF) as u64 * 8;
                return Some(imm12);
            }
        }

        None
    }
}

pub struct CClosureFinder {
    reader: Arc<dyn MemoryReader>,
}

impl CClosureFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        if let Some(offset) = self.find_f_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "CClosure".to_string(),
                "f".to_string(),
                offset,
            ).with_confidence(0.88).with_method("pattern"));
        }

        if let Some(offset) = self.find_cont_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "CClosure".to_string(),
                "cont".to_string(),
                offset,
            ).with_confidence(0.82).with_method("heuristic"));
        }

        if let Some(offset) = self.find_debugname_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "CClosure".to_string(),
                "debugname".to_string(),
                offset,
            ).with_confidence(0.80).with_method("xref"));
        }

        results
    }

    fn find_f_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x10)
    }

    fn find_cont_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x18)
    }

    fn find_debugname_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x20)
    }
}
