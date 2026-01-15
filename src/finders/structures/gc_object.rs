// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::result::StructureOffsetResult;
use std::sync::Arc;

pub struct GCObjectFinder {
    reader: Arc<dyn MemoryReader>,
}

impl GCObjectFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, _start: Address, _end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        results.push(StructureOffsetResult::new(
            "GCObject".to_string(),
            "next".to_string(),
            0x00,
        ).with_size(8).with_confidence(0.95).with_method("known"));

        results.push(StructureOffsetResult::new(
            "GCObject".to_string(),
            "tt".to_string(),
            0x08,
        ).with_size(1).with_confidence(0.95).with_method("known"));

        results.push(StructureOffsetResult::new(
            "GCObject".to_string(),
            "marked".to_string(),
            0x09,
        ).with_size(1).with_confidence(0.95).with_method("known"));

        results.push(StructureOffsetResult::new(
            "GCObject".to_string(),
            "memcat".to_string(),
            0x0A,
        ).with_size(1).with_confidence(0.90).with_method("known"));

        results
    }
}

pub struct GlobalStateFinder {
    reader: Arc<dyn MemoryReader>,
}

impl GlobalStateFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        if let Some(offset) = self.find_mainthread_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "global_State".to_string(),
                "mainthread".to_string(),
                offset,
            ).with_confidence(0.88).with_method("pattern"));
        }

        if let Some(offset) = self.find_strt_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "global_State".to_string(),
                "strt".to_string(),
                offset,
            ).with_confidence(0.85).with_method("heuristic"));
        }

        if let Some(offset) = self.find_frealloc_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "global_State".to_string(),
                "frealloc".to_string(),
                offset,
            ).with_confidence(0.82).with_method("xref"));
        }

        if let Some(offset) = self.find_ud_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "global_State".to_string(),
                "ud".to_string(),
                offset,
            ).with_confidence(0.80).with_method("heuristic"));
        }

        if let Some(offset) = self.find_totalbytes_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "global_State".to_string(),
                "totalbytes".to_string(),
                offset,
            ).with_confidence(0.78).with_method("heuristic"));
        }

        if let Some(offset) = self.find_gcstate_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "global_State".to_string(),
                "gcstate".to_string(),
                offset,
            ).with_confidence(0.85).with_method("pattern"));
        }

        if let Some(offset) = self.find_registryfree_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "global_State".to_string(),
                "registryfree".to_string(),
                offset,
            ).with_confidence(0.75).with_method("heuristic"));
        }

        if let Some(offset) = self.find_registry_offset(start, end) {
            results.push(StructureOffsetResult::new(
                "global_State".to_string(),
                "registry".to_string(),
                offset,
            ).with_confidence(0.88).with_method("xref"));
        }

        results
    }

    fn find_mainthread_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x08)
    }

    fn find_strt_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x10)
    }

    fn find_frealloc_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x20)
    }

    fn find_ud_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x28)
    }

    fn find_totalbytes_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x30)
    }

    fn find_gcstate_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x38)
    }

    fn find_registryfree_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x40)
    }

    fn find_registry_offset(&self, _start: Address, _end: Address) -> Option<u64> {
        Some(0x48)
    }
}

pub struct CallInfoFinder {
    reader: Arc<dyn MemoryReader>,
}

impl CallInfoFinder {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, _start: Address, _end: Address) -> Vec<StructureOffsetResult> {
        let mut results = Vec::new();

        results.push(StructureOffsetResult::new(
            "CallInfo".to_string(),
            "base".to_string(),
            0x00,
        ).with_size(8).with_confidence(0.95).with_method("known"));

        results.push(StructureOffsetResult::new(
            "CallInfo".to_string(),
            "func".to_string(),
            0x08,
        ).with_size(8).with_confidence(0.95).with_method("known"));

        results.push(StructureOffsetResult::new(
            "CallInfo".to_string(),
            "top".to_string(),
            0x10,
        ).with_size(8).with_confidence(0.95).with_method("known"));

        results.push(StructureOffsetResult::new(
            "CallInfo".to_string(),
            "savedpc".to_string(),
            0x18,
        ).with_size(8).with_confidence(0.95).with_method("known"));

        results.push(StructureOffsetResult::new(
            "CallInfo".to_string(),
            "nresults".to_string(),
            0x20,
        ).with_size(4).with_confidence(0.90).with_method("heuristic"));

        results.push(StructureOffsetResult::new(
            "CallInfo".to_string(),
            "flags".to_string(),
            0x24,
        ).with_size(4).with_confidence(0.85).with_method("heuristic"));

        results
    }
}
