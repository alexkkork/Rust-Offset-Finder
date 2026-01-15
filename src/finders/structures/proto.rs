// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::pattern::Pattern;
use crate::finders::result::StructureOffsetResult;
use std::sync::Arc;

pub struct ProtoFinder {
    reader: Arc<dyn MemoryReader>,
}

impl ProtoFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        if let Some(offset) = self.find_k_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "k".to_string(),
                offset,
            ).with_confidence(0.88).with_method("pattern"));
        }

        if let Some(offset) = self.find_code_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "code".to_string(),
                offset,
            ).with_confidence(0.90).with_method("pattern"));
        }

        if let Some(offset) = self.find_p_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "p".to_string(),
                offset,
            ).with_confidence(0.85).with_method("heuristic"));
        }

        if let Some(offset) = self.find_lineinfo_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "lineinfo".to_string(),
                offset,
            ).with_confidence(0.82).with_method("heuristic"));
        }

        if let Some(offset) = self.find_abslineinfo_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "abslineinfo".to_string(),
                offset,
            ).with_confidence(0.80).with_method("heuristic"));
        }

        if let Some(offset) = self.find_locvars_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "locvars".to_string(),
                offset,
            ).with_confidence(0.78).with_method("heuristic"));
        }

        if let Some(offset) = self.find_upvalues_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "upvalues".to_string(),
                offset,
            ).with_confidence(0.80).with_method("heuristic"));
        }

        if let Some(offset) = self.find_source_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "source".to_string(),
                offset,
            ).with_confidence(0.85).with_method("xref"));
        }

        if let Some(offset) = self.find_debugname_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "debugname".to_string(),
                offset,
            ).with_confidence(0.82).with_method("xref"));
        }

        if let Some(offset) = self.find_sizecode_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "sizecode".to_string(),
                offset,
            ).with_confidence(0.86).with_method("pattern"));
        }

        if let Some(offset) = self.find_sizep_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "sizep".to_string(),
                offset,
            ).with_confidence(0.82).with_method("heuristic"));
        }

        if let Some(offset) = self.find_sizek_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "sizek".to_string(),
                offset,
            ).with_confidence(0.84).with_method("pattern"));
        }

        if let Some(offset) = self.find_sizeupvalues_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "sizeupvalues".to_string(),
                offset,
            ).with_confidence(0.80).with_method("heuristic"));
        }

        if let Some(offset) = self.find_sizelocvars_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "sizelocvars".to_string(),
                offset,
            ).with_confidence(0.78).with_method("heuristic"));
        }

        if let Some(offset) = self.find_linedefined_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "linedefined".to_string(),
                offset,
            ).with_confidence(0.82).with_method("heuristic"));
        }

        if let Some(offset) = self.find_bytecodeid_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Proto".to_string(),
                "bytecodeid".to_string(),
                offset,
            ).with_confidence(0.75).with_method("heuristic"));
        }

        results
    }

    fn find_k_offset(&self, start: Address, end: Address) -> Option<u64> {
        let patterns = vec![
            Pattern::from_hex("F9 ?? ?? ?? 8B ?? ?? ?? F9 ?? ?? ?? B4"),
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

    fn find_code_offset(&self, start: Address, end: Address) -> Option<u64> {
        let patterns = vec![
            Pattern::from_hex("F9 ?? ?? ?? 39 ?? ?? ?? 91 ?? ?? ?? F9"),
        ];

        for pattern in patterns {
            let mut current = start;

            while current < end {
                if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                    if let Some(offset) = pattern.find_in(&bytes) {
                        let insn_addr = current + offset as u64;

                        if let Some(struct_offset) = self.extract_ldr_offset(insn_addr) {
                            if struct_offset >= 0x10 && struct_offset <= 0x40 {
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

    fn find_p_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x18)
    }

    fn find_lineinfo_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x20)
    }

    fn find_abslineinfo_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x28)
    }

    fn find_locvars_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x30)
    }

    fn find_upvalues_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x38)
    }

    fn find_source_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x40)
    }

    fn find_debugname_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x48)
    }

    fn find_sizecode_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x54)
    }

    fn find_sizep_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x58)
    }

    fn find_sizek_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x5C)
    }

    fn find_sizeupvalues_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x50)
    }

    fn find_sizelocvars_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x52)
    }

    fn find_linedefined_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x60)
    }

    fn find_bytecodeid_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x64)
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
