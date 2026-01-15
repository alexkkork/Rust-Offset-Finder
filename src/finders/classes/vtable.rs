// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader};
use crate::finders::result::MethodResult;
use std::sync::Arc;
use std::collections::HashMap;

pub struct VTableAnalyzer {
    reader: Arc<dyn MemoryReader>,
}

impl VTableAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn analyze_vtable(&self, vtable_addr: Address) -> Option<VTableInfo> {
        let mut entries = Vec::new();
        let mut current = vtable_addr;

        for i in 0..256 {
            if let Ok(bytes) = self.reader.read_bytes(current, 8) {
                let func_ptr = u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]);

                if func_ptr < 0x100000000 || func_ptr > 0x7FFFFFFFFFFF {
                    break;
                }

                if !self.is_valid_function_pointer(Address::new(func_ptr)) {
                    break;
                }

                entries.push(VTableEntry {
                    index: i,
                    address: Address::new(func_ptr),
                    name: None,
                });

                current = current + 8;
            } else {
                break;
            }
        }

        if entries.is_empty() {
            return None;
        }

        Some(VTableInfo {
            address: vtable_addr,
            entries,
            rtti_address: None,
            class_name: None,
        })
    }

    fn is_valid_function_pointer(&self, addr: Address) -> bool {
        if let Ok(bytes) = self.reader.read_bytes(addr, 4) {
            let first_insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            let is_prologue = (first_insn & 0x7F800000) == 0x29000000
                || (first_insn & 0x7F800000) == 0x6D000000
                || (first_insn & 0xFFFFFC1F) == 0xD65F0000
                || (first_insn & 0xFC000000) == 0x14000000;

            return is_prologue;
        }

        false
    }

    pub fn find_vtables(&self, start: Address, end: Address) -> Vec<VTableInfo> {
        let mut vtables = Vec::new();
        let mut current = start;

        while current < end {
            if let Ok(bytes) = self.reader.read_bytes(current, 4096) {
                for i in (0..bytes.len() - 16).step_by(8) {
                    let ptr1 = u64::from_le_bytes([
                        bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3],
                        bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7],
                    ]);

                    let ptr2 = u64::from_le_bytes([
                        bytes[i + 8], bytes[i + 9], bytes[i + 10], bytes[i + 11],
                        bytes[i + 12], bytes[i + 13], bytes[i + 14], bytes[i + 15],
                    ]);

                    if ptr1 >= 0x100000000 && ptr1 <= 0x7FFFFFFFFFFF
                        && ptr2 >= 0x100000000 && ptr2 <= 0x7FFFFFFFFFFF
                    {
                        let potential_vtable = current + i as u64;

                        if let Some(vtable_info) = self.analyze_vtable(potential_vtable) {
                            if vtable_info.entries.len() >= 3 {
                                vtables.push(vtable_info);
                            }
                        }
                    }
                }
            }

            current = current + 4000;
        }

        vtables
    }

    pub fn extract_methods(&self, vtable: &VTableInfo, class_name: &str) -> Vec<MethodResult> {
        let mut methods = Vec::new();

        for entry in &vtable.entries {
            let method_name = entry.name.clone()
                .unwrap_or_else(|| format!("vfunc_{}", entry.index));

            methods.push(MethodResult::new(
                class_name.to_string(),
                method_name,
                entry.address,
            ).with_vtable_index(entry.index as u32)
             .set_virtual(true)
             .with_confidence(0.85));
        }

        methods
    }

    pub fn compare_vtables(&self, vtable1: &VTableInfo, vtable2: &VTableInfo) -> VTableComparison {
        let mut shared_entries = Vec::new();
        let mut unique_to_first = Vec::new();
        let mut unique_to_second = Vec::new();

        let entries1: HashMap<_, _> = vtable1.entries.iter()
            .map(|e| (e.index, e.address))
            .collect();

        let entries2: HashMap<_, _> = vtable2.entries.iter()
            .map(|e| (e.index, e.address))
            .collect();

        for (idx, addr) in &entries1 {
            if let Some(addr2) = entries2.get(idx) {
                if addr == addr2 {
                    shared_entries.push(*idx);
                } else {
                    unique_to_first.push(*idx);
                    unique_to_second.push(*idx);
                }
            } else {
                unique_to_first.push(*idx);
            }
        }

        for idx in entries2.keys() {
            if !entries1.contains_key(idx) {
                unique_to_second.push(*idx);
            }
        }

        let shared_count = shared_entries.len();
        let unique_first_count = unique_to_first.len();
        VTableComparison {
            shared_entries,
            unique_to_first,
            unique_to_second,
            likely_inheritance: shared_count > 0 && unique_first_count > 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VTableInfo {
    pub address: Address,
    pub entries: Vec<VTableEntry>,
    pub rtti_address: Option<Address>,
    pub class_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VTableEntry {
    pub index: usize,
    pub address: Address,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VTableComparison {
    pub shared_entries: Vec<usize>,
    pub unique_to_first: Vec<usize>,
    pub unique_to_second: Vec<usize>,
    pub likely_inheritance: bool,
}
