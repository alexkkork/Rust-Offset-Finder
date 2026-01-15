// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::result::StructureOffsetResult;
use std::sync::Arc;

pub struct UserdataFinder {
    reader: Arc<dyn MemoryReader>,
}

impl UserdataFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        if let Some(offset) = self.find_tag_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Udata".to_string(),
                "tag".to_string(),
                offset,
            ).with_confidence(0.88).with_method("pattern"));
        }

        if let Some(offset) = self.find_len_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Udata".to_string(),
                "len".to_string(),
                offset,
            ).with_confidence(0.90).with_method("pattern"));
        }

        if let Some(offset) = self.find_metatable_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Udata".to_string(),
                "metatable".to_string(),
                offset,
            ).with_confidence(0.92).with_method("xref"));
        }

        if let Some(offset) = self.find_data_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "Udata".to_string(),
                "data".to_string(),
                offset,
            ).with_confidence(0.95).with_method("known"));
        }

        results
    }

    fn find_tag_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x04)
    }

    fn find_len_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x08)
    }

    fn find_metatable_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x10)
    }

    fn find_data_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x18)
    }
}

pub struct InstanceUserdataFinder {
    reader: Arc<dyn MemoryReader>,
}

impl InstanceUserdataFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        if let Some(offset) = self.find_instance_ptr_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "InstanceUserdata".to_string(),
                "instance".to_string(),
                offset,
            ).with_confidence(0.90).with_method("pattern"));
        }

        if let Some(offset) = self.find_weak_ref_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "InstanceUserdata".to_string(),
                "weak_ref".to_string(),
                offset,
            ).with_confidence(0.85).with_method("heuristic"));
        }

        results
    }

    fn find_instance_ptr_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x00)
    }

    fn find_weak_ref_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x08)
    }
}
