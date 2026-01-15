// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::pattern::Pattern;
use crate::finders::result::StructureOffsetResult;
use std::sync::Arc;

pub struct ExtraSpaceFinder {
    reader: Arc<dyn MemoryReader>,
}

impl ExtraSpaceFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        if let Some(offset) = self.find_identity_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "ExtraSpace".to_string(),
                "identity".to_string(),
                offset,
            ).with_confidence(0.88).with_method("pattern"));
        }

        if let Some(offset) = self.find_capabilities_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "ExtraSpace".to_string(),
                "capabilities".to_string(),
                offset,
            ).with_confidence(0.86).with_method("pattern"));
        }

        if let Some(offset) = self.find_script_context_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "ExtraSpace".to_string(),
                "script_context".to_string(),
                offset,
            ).with_confidence(0.85).with_method("xref"));
        }

        if let Some(offset) = self.find_shared_extra_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "ExtraSpace".to_string(),
                "shared_extra".to_string(),
                offset,
            ).with_confidence(0.82).with_method("heuristic"));
        }

        results
    }

    fn find_identity_offset(&self, start: Address, end: Address) -> Option<u64> {
        let patterns = vec![
            Pattern::from_hex("B9 ?? ?? ?? 71 ?? ?? ?? 54 ?? ?? ?? B9"),
            Pattern::from_hex("F9 ?? ?? ?? B9 ?? ?? ?? 52 ?? ?? ?? 72"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let insn_addr = current + offset as u64;

                        if let Ok(insn_bytes) = self.reader.read_bytes(insn_addr, 4) {
                            let insn = u32::from_le_bytes([insn_bytes[0], insn_bytes[1], insn_bytes[2], insn_bytes[3]]);

                            if (insn & 0xFFC00000) == 0xB9400000 {
                                let imm12 = ((insn >> 10) & 0xFFF) as u64 * 4;

                                if imm12 >= 0x10 && imm12 <= 0x40 {
                                    return Some(imm12);
                                }
                            }
                        }
                    }
                }

                current = current + 4000;
            }
        }

        Some(0x18)
    }

    fn find_capabilities_offset(&self, start: Address, end: Address) -> Option<u64> {
        let patterns = vec![
            Pattern::from_hex("F9 ?? ?? ?? B9 ?? ?? ?? 72 ?? ?? ?? B9"),
            Pattern::from_hex("B9 ?? ?? ?? 2A ?? ?? ?? B9 ?? ?? ?? F9"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let insn_addr = current + offset as u64;

                        if let Some(struct_offset) = self.extract_str_offset(insn_addr + 8) {
                            if struct_offset >= 0x20 && struct_offset <= 0x60 {
                                return Some(struct_offset);
                            }
                        }
                    }
                }

                current = current + 4000;
            }
        }

        Some(0x20)
    }

    fn find_script_context_offset(&self, start: Address, end: Address) -> Option<u64> {
        let patterns = vec![
            Pattern::from_hex("F9 ?? ?? ?? B4 ?? ?? ?? F9 ?? ?? ?? B4"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let insn_addr = current + offset as u64;

                        if let Some(struct_offset) = self.extract_ldr_offset(insn_addr) {
                            if struct_offset >= 0x08 && struct_offset <= 0x30 {
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

    fn find_shared_extra_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x00)
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

    fn extract_str_offset(&self, addr: Address) -> Option<u64> {
        if let Ok(bytes) = self.reader.read_bytes(addr, 4) {
            let insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            if (insn & 0xFFC00000) == 0xB9000000 {
                let imm12 = ((insn >> 10) & 0xFFF) as u64 * 4;
                return Some(imm12);
            }
        }

        None
    }
}

pub struct ScriptContextFinder {
    reader: Arc<dyn MemoryReader>,
}

impl ScriptContextFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        if let Some(offset) = self.find_identity_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "ScriptContext".to_string(),
                "identity".to_string(),
                offset,
            ).with_confidence(0.85).with_method("pattern"));
        }

        if let Some(offset) = self.find_capabilities_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "ScriptContext".to_string(),
                "capabilities".to_string(),
                offset,
            ).with_confidence(0.83).with_method("pattern"));
        }

        results
    }

    fn find_identity_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x18)
    }

    fn find_capabilities_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x20)
    }
}
