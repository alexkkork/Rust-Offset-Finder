// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use std::collections::HashMap;
use std::sync::Arc;

pub struct VTableAnalyzer {
    reader: Arc<dyn MemoryReader>,
    vtables: HashMap<u64, VTableInfo>,
    min_entries: usize,
    max_entries: usize,
}

#[derive(Debug, Clone)]
pub struct VTableInfo {
    pub address: Address,
    pub entries: Vec<VTableEntry>,
    pub class_name: Option<String>,
    pub type_info_ptr: Option<Address>,
    pub rtti: Option<RTTIInfo>,
    pub size: usize,
    pub xrefs: Vec<Address>,
}

#[derive(Debug, Clone)]
pub struct VTableEntry {
    pub index: usize,
    pub address: Address,
    pub target: Address,
    pub function_name: Option<String>,
    pub is_pure_virtual: bool,
}

#[derive(Debug, Clone)]
pub struct RTTIInfo {
    pub type_name: String,
    pub base_classes: Vec<BaseClassInfo>,
    pub type_descriptor: Address,
}

#[derive(Debug, Clone)]
pub struct BaseClassInfo {
    pub name: String,
    pub offset: i64,
    pub vtable_offset: i64,
}

impl VTableAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            vtables: HashMap::new(),
            min_entries: 2,
            max_entries: 256,
        }
    }

    pub fn with_min_entries(mut self, min: usize) -> Self {
        self.min_entries = min;
        self
    }

    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    pub fn analyze(&mut self, addr: Address) -> Result<Option<VTableInfo>, MemoryError> {
        if let Some(cached) = self.vtables.get(&addr.as_u64()) {
            return Ok(Some(cached.clone()));
        }

        let entries = self.scan_vtable_entries(addr)?;

        if entries.len() < self.min_entries {
            return Ok(None);
        }

        let type_info_ptr = self.find_type_info(addr)?;
        let rtti = if let Some(ti_ptr) = type_info_ptr {
            self.parse_rtti(ti_ptr)?
        } else {
            None
        };

        let class_name = rtti.as_ref().map(|r| r.type_name.clone());

        let info = VTableInfo {
            address: addr,
            entries,
            class_name,
            type_info_ptr,
            rtti,
            size: 0,
            xrefs: Vec::new(),
        };

        self.vtables.insert(addr.as_u64(), info.clone());
        Ok(Some(info))
    }

    fn scan_vtable_entries(&self, addr: Address) -> Result<Vec<VTableEntry>, MemoryError> {
        let mut entries = Vec::new();

        for i in 0..self.max_entries {
            let entry_addr = addr + (i * 8) as u64;
            let target = self.reader.read_u64(entry_addr)?;

            if target == 0 {
                break;
            }

            if !self.is_valid_code_pointer(target) {
                break;
            }

            let is_pure_virtual = self.is_pure_virtual_entry(target);

            entries.push(VTableEntry {
                index: i,
                address: entry_addr,
                target: Address::new(target),
                function_name: None,
                is_pure_virtual,
            });
        }

        Ok(entries)
    }

    fn is_valid_code_pointer(&self, ptr: u64) -> bool {
        if ptr < 0x100000000 || ptr >= 0x800000000000 {
            return false;
        }

        let ptr_addr = Address::new(ptr);
        if let Ok(insn) = self.reader.read_u32(ptr_addr) {
            let op0 = (insn >> 25) & 0xF;
            op0 != 0 && op0 != 1 && op0 != 3
        } else {
            false
        }
    }

    fn is_pure_virtual_entry(&self, target: u64) -> bool {
        let target_addr = Address::new(target);
        if let Ok(insn) = self.reader.read_u32(target_addr) {
            insn == 0xD4200000
        } else {
            false
        }
    }

    fn find_type_info(&self, vtable_addr: Address) -> Result<Option<Address>, MemoryError> {
        let before_vtable = vtable_addr - 8;
        let ptr = self.reader.read_u64(before_vtable)?;

        if ptr >= 0x100000000 && ptr < 0x800000000000 {
            return Ok(Some(Address::new(ptr)));
        }

        Ok(None)
    }

    fn parse_rtti(&self, type_info_ptr: Address) -> Result<Option<RTTIInfo>, MemoryError> {
        let name_ptr = self.reader.read_u64(type_info_ptr + 8)?;

        if name_ptr < 0x100000000 || name_ptr >= 0x800000000000 {
            return Ok(None);
        }

        let name_addr = Address::new(name_ptr);
        let name_bytes = self.reader.read_bytes(name_addr, 256)?;
        let null_pos = name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len());

        let type_name = String::from_utf8_lossy(&name_bytes[..null_pos]).to_string();

        let demangled_name = self.demangle_type_name(&type_name);

        Ok(Some(RTTIInfo {
            type_name: demangled_name,
            base_classes: Vec::new(),
            type_descriptor: type_info_ptr,
        }))
    }

    fn demangle_type_name(&self, name: &str) -> String {
        if name.starts_with("_ZTS") {
            let mangled = &name[4..];
            self.demangle_itanium(mangled)
        } else {
            name.to_string()
        }
    }

    fn demangle_itanium(&self, mangled: &str) -> String {
        let chars: Vec<char> = mangled.chars().collect();
        let mut result = String::new();
        let mut i = 0;

        while i < chars.len() {
            if chars[i].is_ascii_digit() {
                let mut len_str = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    len_str.push(chars[i]);
                    i += 1;
                }
                if let Ok(len) = len_str.parse::<usize>() {
                    let end = (i + len).min(chars.len());
                    if !result.is_empty() {
                        result.push_str("::");
                    }
                    for j in i..end {
                        result.push(chars[j]);
                    }
                    i = end;
                }
            } else {
                i += 1;
            }
        }

        if result.is_empty() {
            mangled.to_string()
        } else {
            result
        }
    }

    pub fn scan_for_vtables(&mut self, start: Address, end: Address) -> Result<Vec<Address>, MemoryError> {
        let mut found = Vec::new();
        let mut current = start;

        while current < end {
            if self.is_likely_vtable_start(current)? {
                if let Ok(Some(_)) = self.analyze(current) {
                    found.push(current);
                }
            }
            current = current + 8;
        }

        Ok(found)
    }

    fn is_likely_vtable_start(&self, addr: Address) -> Result<bool, MemoryError> {
        let mut valid_count = 0;

        for i in 0..3 {
            let entry_addr = addr + (i * 8) as u64;
            if let Ok(ptr) = self.reader.read_u64(entry_addr) {
                if self.is_valid_code_pointer(ptr) {
                    valid_count += 1;
                }
            }
        }

        Ok(valid_count >= 2)
    }

    pub fn get_vtable(&self, addr: Address) -> Option<&VTableInfo> {
        self.vtables.get(&addr.as_u64())
    }

    pub fn get_all_vtables(&self) -> Vec<&VTableInfo> {
        self.vtables.values().collect()
    }

    pub fn get_vtables_for_class(&self, class_name: &str) -> Vec<&VTableInfo> {
        self.vtables
            .values()
            .filter(|v| v.class_name.as_ref().map(|n| n.contains(class_name)).unwrap_or(false))
            .collect()
    }

    pub fn find_virtual_function(&self, vtable_addr: Address, index: usize) -> Option<Address> {
        self.vtables.get(&vtable_addr.as_u64())
            .and_then(|v| v.entries.get(index))
            .map(|e| e.target)
    }

    pub fn add_xref(&mut self, vtable_addr: Address, xref: Address) {
        if let Some(vtable) = self.vtables.get_mut(&vtable_addr.as_u64()) {
            if !vtable.xrefs.contains(&xref) {
                vtable.xrefs.push(xref);
            }
        }
    }

    pub fn clear(&mut self) {
        self.vtables.clear();
    }

    pub fn vtable_count(&self) -> usize {
        self.vtables.len()
    }
}

impl VTableInfo {
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn get_entry(&self, index: usize) -> Option<&VTableEntry> {
        self.entries.get(index)
    }

    pub fn has_pure_virtual(&self) -> bool {
        self.entries.iter().any(|e| e.is_pure_virtual)
    }

    pub fn pure_virtual_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_pure_virtual).count()
    }

    pub fn get_function_addresses(&self) -> Vec<Address> {
        self.entries.iter().map(|e| e.target).collect()
    }
}

pub fn find_vtable_references(reader: &dyn MemoryReader, vtable_addr: Address, start: Address, end: Address) -> Result<Vec<Address>, MemoryError> {
    let mut refs = Vec::new();
    let target = vtable_addr.as_u64();
    let target_bytes = target.to_le_bytes();
    let scan_size = (end.as_u64() - start.as_u64()) as usize;
    let data = reader.read_bytes(start, scan_size)?;

    for i in 0..data.len().saturating_sub(8) {
        if &data[i..i + 8] == &target_bytes {
            refs.push(start + i as u64);
        }
    }

    Ok(refs)
}

pub fn estimate_class_size(reader: &dyn MemoryReader, instance_addr: Address, max_size: usize) -> Result<usize, MemoryError> {
    let data = reader.read_bytes(instance_addr, max_size)?;

    let mut last_nonzero = 0;
    for (i, chunk) in data.chunks(8).enumerate() {
        let val = u64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
        if val != 0 {
            last_nonzero = (i + 1) * 8;
        }
    }

    Ok(((last_nonzero + 7) / 8) * 8)
}
