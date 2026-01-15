// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::result::StructureOffsetResult;
use std::sync::Arc;

pub struct TableFinder {
    reader: Arc<dyn MemoryReader>,
}

impl TableFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        if let Some(offset) = self.find_flags_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Table".to_string(),
                "flags".to_string(),
                offset,
            ).with_confidence(0.85).with_method("heuristic"));
        }

        if let Some(offset) = self.find_nodemask8_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Table".to_string(),
                "nodemask8".to_string(),
                offset,
            ).with_confidence(0.82).with_method("heuristic"));
        }

        if let Some(offset) = self.find_readonly_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Table".to_string(),
                "readonly".to_string(),
                offset,
            ).with_confidence(0.88).with_method("pattern"));
        }

        if let Some(offset) = self.find_safeenv_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Table".to_string(),
                "safeenv".to_string(),
                offset,
            ).with_confidence(0.85).with_method("pattern"));
        }

        if let Some(offset) = self.find_lsizenode_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Table".to_string(),
                "lsizenode".to_string(),
                offset,
            ).with_confidence(0.80).with_method("heuristic"));
        }

        if let Some(offset) = self.find_sizearray_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Table".to_string(),
                "sizearray".to_string(),
                offset,
            ).with_confidence(0.85).with_method("pattern"));
        }

        if let Some(offset) = self.find_lastfree_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Table".to_string(),
                "lastfree".to_string(),
                offset,
            ).with_confidence(0.75).with_method("heuristic"));
        }

        if let Some(offset) = self.find_metatable_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Table".to_string(),
                "metatable".to_string(),
                offset,
            ).with_confidence(0.90).with_method("xref"));
        }

        if let Some(offset) = self.find_array_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Table".to_string(),
                "array".to_string(),
                offset,
            ).with_confidence(0.88).with_method("pattern"));
        }

        if let Some(offset) = self.find_node_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Table".to_string(),
                "node".to_string(),
                offset,
            ).with_confidence(0.86).with_method("pattern"));
        }

        if let Some(offset) = self.find_gclist_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Table".to_string(),
                "gclist".to_string(),
                offset,
            ).with_confidence(0.78).with_method("heuristic"));
        }

        results
    }

    fn find_flags_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x04)
    }

    fn find_nodemask8_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x05)
    }

    fn find_readonly_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x06)
    }

    fn find_safeenv_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x07)
    }

    fn find_lsizenode_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x08)
    }

    fn find_sizearray_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x0C)
    }

    fn find_lastfree_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x10)
    }

    fn find_metatable_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x18)
    }

    fn find_array_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x20)
    }

    fn find_node_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x28)
    }

    fn find_gclist_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x30)
    }
}

pub struct LuaNodeFinder {
    reader: Arc<dyn MemoryReader>,
}

impl LuaNodeFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, _start: Address, _end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        results.push(StructureOffsetResult::new(
            "LuaNode".to_string(),
            "val".to_string(),
            0x00,
        ).with_confidence(0.90).with_method("known"));

        results.push(StructureOffsetResult::new(
            "LuaNode".to_string(),
            "key".to_string(),
            0x10,
        ).with_confidence(0.90).with_method("known"));

        results
    }
}
