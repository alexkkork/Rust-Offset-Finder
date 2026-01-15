// Tue Jan 15 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Represents a single entry in a virtual table
#[derive(Debug, Clone)]
pub struct VTableEntry {
    /// Index in the vtable (0-based)
    pub index: usize,
    /// Address of the virtual function
    pub function_address: Address,
    /// Inferred name of the function (if available)
    pub function_name: Option<String>,
    /// Whether this entry has been overridden from base class
    pub is_override: bool,
    /// The class that originally declared this virtual function
    pub declaring_class: Option<String>,
}

impl VTableEntry {
    pub fn new(index: usize, function_address: Address) -> Self {
        Self {
            index,
            function_address,
            function_name: None,
            is_override: false,
            declaring_class: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.function_name = Some(name.to_string());
        self
    }

    pub fn with_override(mut self, is_override: bool) -> Self {
        self.is_override = is_override;
        self
    }

    pub fn with_declaring_class(mut self, class: &str) -> Self {
        self.declaring_class = Some(class.to_string());
        self
    }

    pub fn is_valid(&self) -> bool {
        self.function_address.as_u64() != 0 && 
        self.function_address.as_u64() >= 0x100000000
    }
}

impl fmt::Display for VTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.function_name.as_deref().unwrap_or("unknown");
        let override_mark = if self.is_override { " [override]" } else { "" };
        write!(f, "[{}] {:016x} {}{}", self.index, self.function_address.as_u64(), name, override_mark)
    }
}

/// Represents a complete virtual table
#[derive(Debug, Clone)]
pub struct VTable {
    /// Address of the vtable in memory
    pub address: Address,
    /// Name of the class this vtable belongs to
    pub class_name: String,
    /// All entries in the vtable
    pub entries: Vec<VTableEntry>,
    /// RTTI address if present (typically at vtable[-1])
    pub rtti_address: Option<Address>,
    /// Parent vtable address (for inheritance)
    pub parent_vtable: Option<Address>,
    /// Size of the vtable in bytes
    pub size: usize,
}

impl VTable {
    pub fn new(address: Address, class_name: &str) -> Self {
        Self {
            address,
            class_name: class_name.to_string(),
            entries: Vec::new(),
            rtti_address: None,
            parent_vtable: None,
            size: 0,
        }
    }

    pub fn add_entry(&mut self, entry: VTableEntry) {
        self.entries.push(entry);
        self.size = self.entries.len() * 8;
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn get_entry(&self, index: usize) -> Option<&VTableEntry> {
        self.entries.get(index)
    }

    pub fn get_function_address(&self, index: usize) -> Option<Address> {
        self.entries.get(index).map(|e| e.function_address)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&VTableEntry> {
        self.entries.iter().find(|e| {
            e.function_name.as_ref().map(|n| n.contains(name)).unwrap_or(false)
        })
    }

    pub fn override_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_override).count()
    }

    pub fn unique_function_count(&self) -> usize {
        let mut seen = HashSet::new();
        for entry in &self.entries {
            seen.insert(entry.function_address.as_u64());
        }
        seen.len()
    }

    pub fn with_rtti(mut self, rtti: Address) -> Self {
        self.rtti_address = Some(rtti);
        self
    }

    pub fn with_parent(mut self, parent: Address) -> Self {
        self.parent_vtable = Some(parent);
        self
    }
}

impl fmt::Display for VTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "VTable for {} @ {:016x}", self.class_name, self.address.as_u64())?;
        writeln!(f, "  Entries: {}", self.entries.len())?;
        if let Some(rtti) = self.rtti_address {
            writeln!(f, "  RTTI: {:016x}", rtti.as_u64())?;
        }
        for entry in &self.entries {
            writeln!(f, "  {}", entry)?;
        }
        Ok(())
    }
}

/// Analyzer for virtual tables
pub struct VTableAnalyzer {
    reader: Arc<dyn MemoryReader>,
    vtables: HashMap<u64, VTable>,
    function_to_vtable: HashMap<u64, Vec<(u64, usize)>>, // function addr -> [(vtable addr, index)]
}

impl VTableAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            vtables: HashMap::new(),
            function_to_vtable: HashMap::new(),
        }
    }

    /// Analyze a potential vtable at the given address
    pub fn analyze_vtable(&mut self, address: Address, class_name: &str) -> Result<VTable, MemoryError> {
        let mut vtable = VTable::new(address, class_name);

        // Try to read RTTI from vtable[-1] (common in MSVC and some ARM ABIs)
        if address.as_u64() >= 8 {
            let rtti_addr = address - 8u64;
            if let Ok(rtti) = self.reader.read_u64(rtti_addr) {
                if rtti != 0 && rtti >= 0x100000000 && rtti < 0x800000000000 {
                    vtable.rtti_address = Some(Address::new(rtti));
                }
            }
        }

        // Read vtable entries
        let mut index = 0;
        let max_entries = 500; // Safety limit
        
        while index < max_entries {
            let entry_addr = address + (index * 8) as u64;
            let func_addr = self.reader.read_u64(entry_addr)?;

            // Check if this looks like a valid function pointer
            if !self.is_valid_function_pointer(func_addr) {
                break;
            }

            let entry = VTableEntry::new(index, Address::new(func_addr));
            vtable.add_entry(entry);

            // Track function to vtable mapping
            self.function_to_vtable
                .entry(func_addr)
                .or_default()
                .push((address.as_u64(), index));

            index += 1;
        }

        self.vtables.insert(address.as_u64(), vtable.clone());
        Ok(vtable)
    }

    /// Check if a value looks like a valid ARM64 function pointer
    fn is_valid_function_pointer(&self, addr: u64) -> bool {
        // Must be in valid address range
        if addr < 0x100000000 || addr > 0x800000000000 {
            return false;
        }

        // Must be 4-byte aligned (ARM64 instruction alignment)
        if addr % 4 != 0 {
            return false;
        }

        // Try to read the first instruction
        if let Ok(bytes) = self.reader.read_bytes(Address::new(addr), 4) {
            let insn = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            
            // Check for common function prologue patterns
            // STP x29, x30, [sp, #-N]!
            if (insn & 0xFFC003E0) == 0xA9800000 {
                return true;
            }
            // SUB sp, sp, #N
            if (insn & 0xFF8003FF) == 0xD10003FF {
                return true;
            }
            // PACIBSP (pointer authentication)
            if insn == 0xD503237F {
                return true;
            }
            // MOV x29, sp or other common starts
            if (insn & 0xFFFFFC00) == 0x910003E0 {
                return true;
            }
            // STR/STP variations
            if (insn & 0x7F800000) == 0x29000000 || (insn & 0x7F800000) == 0x6D000000 {
                return true;
            }
            
            // Allow if it's any valid ARM64 instruction (not all zeros or invalid)
            if insn != 0 && insn != 0xFFFFFFFF {
                return true;
            }
        }

        false
    }

    /// Compare two vtables to find differences
    pub fn compare_vtables(&self, vtable1: &VTable, vtable2: &VTable) -> VTableComparison {
        let mut comparison = VTableComparison::new(&vtable1.class_name, &vtable2.class_name);

        let max_len = vtable1.entries.len().max(vtable2.entries.len());
        
        for i in 0..max_len {
            let entry1 = vtable1.entries.get(i);
            let entry2 = vtable2.entries.get(i);

            match (entry1, entry2) {
                (Some(e1), Some(e2)) => {
                    if e1.function_address != e2.function_address {
                        comparison.add_difference(VTableDifference::Modified {
                            index: i,
                            old_addr: e1.function_address,
                            new_addr: e2.function_address,
                        });
                    } else {
                        comparison.add_match(i);
                    }
                }
                (Some(e1), None) => {
                    comparison.add_difference(VTableDifference::Removed {
                        index: i,
                        addr: e1.function_address,
                    });
                }
                (None, Some(e2)) => {
                    comparison.add_difference(VTableDifference::Added {
                        index: i,
                        addr: e2.function_address,
                    });
                }
                (None, None) => unreachable!(),
            }
        }

        comparison
    }

    /// Find potential inheritance by comparing vtable prefixes
    pub fn detect_inheritance(&self, child: &VTable, parent: &VTable) -> InheritanceInfo {
        let mut info = InheritanceInfo::new(&child.class_name, &parent.class_name);

        // Check if child vtable starts with parent vtable entries
        let mut matching_prefix = 0;
        for (i, parent_entry) in parent.entries.iter().enumerate() {
            if let Some(child_entry) = child.entries.get(i) {
                if child_entry.function_address == parent_entry.function_address {
                    matching_prefix += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        info.matching_entries = matching_prefix;
        info.is_likely_derived = matching_prefix > 0 && 
            matching_prefix as f64 / parent.entries.len() as f64 > 0.3;

        // Count overrides (different addresses for same index)
        for (i, parent_entry) in parent.entries.iter().enumerate() {
            if let Some(child_entry) = child.entries.get(i) {
                if child_entry.function_address != parent_entry.function_address {
                    info.overridden_methods.push(i);
                }
            }
        }

        // Count new virtuals (entries beyond parent size)
        for i in parent.entries.len()..child.entries.len() {
            info.new_virtuals.push(i);
        }

        info
    }

    /// Get all analyzed vtables
    pub fn get_vtables(&self) -> Vec<&VTable> {
        self.vtables.values().collect()
    }

    /// Find which vtables contain a given function
    pub fn find_vtables_with_function(&self, func_addr: Address) -> Vec<(Address, usize)> {
        self.function_to_vtable
            .get(&func_addr.as_u64())
            .map(|v| v.iter().map(|(vt, idx)| (Address::new(*vt), *idx)).collect())
            .unwrap_or_default()
    }

    /// Scan memory range for potential vtables
    pub fn scan_for_vtables(&mut self, start: Address, end: Address) -> Result<Vec<Address>, MemoryError> {
        let mut vtable_candidates = Vec::new();
        let mut current = start;
        let step = 8u64; // Vtables are 8-byte aligned

        while current < end {
            // Read potential vtable first entry
            if let Ok(first_entry) = self.reader.read_u64(current) {
                if self.is_valid_function_pointer(first_entry) {
                    // Check if this looks like a vtable (multiple consecutive function pointers)
                    let mut consecutive_valid = 0;
                    for i in 0..16 {
                        let entry_addr = current + (i * 8) as u64;
                        if let Ok(entry) = self.reader.read_u64(entry_addr) {
                            if self.is_valid_function_pointer(entry) {
                                consecutive_valid += 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    // At least 3 consecutive function pointers suggests a vtable
                    if consecutive_valid >= 3 {
                        vtable_candidates.push(current);
                    }
                }
            }

            current = current + step;
        }

        Ok(vtable_candidates)
    }

    pub fn clear(&mut self) {
        self.vtables.clear();
        self.function_to_vtable.clear();
    }
}

/// Comparison result between two vtables
#[derive(Debug, Clone)]
pub struct VTableComparison {
    pub class1: String,
    pub class2: String,
    pub differences: Vec<VTableDifference>,
    pub matching_indices: Vec<usize>,
}

impl VTableComparison {
    pub fn new(class1: &str, class2: &str) -> Self {
        Self {
            class1: class1.to_string(),
            class2: class2.to_string(),
            differences: Vec::new(),
            matching_indices: Vec::new(),
        }
    }

    pub fn add_difference(&mut self, diff: VTableDifference) {
        self.differences.push(diff);
    }

    pub fn add_match(&mut self, index: usize) {
        self.matching_indices.push(index);
    }

    pub fn match_percentage(&self) -> f64 {
        let total = self.differences.len() + self.matching_indices.len();
        if total == 0 {
            return 100.0;
        }
        (self.matching_indices.len() as f64 / total as f64) * 100.0
    }

    pub fn has_differences(&self) -> bool {
        !self.differences.is_empty()
    }
}

/// A single difference between vtable entries
#[derive(Debug, Clone)]
pub enum VTableDifference {
    Added { index: usize, addr: Address },
    Removed { index: usize, addr: Address },
    Modified { index: usize, old_addr: Address, new_addr: Address },
}

impl fmt::Display for VTableDifference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VTableDifference::Added { index, addr } => {
                write!(f, "[{}] Added: {:016x}", index, addr.as_u64())
            }
            VTableDifference::Removed { index, addr } => {
                write!(f, "[{}] Removed: {:016x}", index, addr.as_u64())
            }
            VTableDifference::Modified { index, old_addr, new_addr } => {
                write!(f, "[{}] Modified: {:016x} -> {:016x}", index, old_addr.as_u64(), new_addr.as_u64())
            }
        }
    }
}

/// Information about class inheritance relationships
#[derive(Debug, Clone)]
pub struct InheritanceInfo {
    pub child_class: String,
    pub parent_class: String,
    pub matching_entries: usize,
    pub is_likely_derived: bool,
    pub overridden_methods: Vec<usize>,
    pub new_virtuals: Vec<usize>,
}

impl InheritanceInfo {
    pub fn new(child: &str, parent: &str) -> Self {
        Self {
            child_class: child.to_string(),
            parent_class: parent.to_string(),
            matching_entries: 0,
            is_likely_derived: false,
            overridden_methods: Vec::new(),
            new_virtuals: Vec::new(),
        }
    }

    pub fn confidence(&self) -> f64 {
        if self.is_likely_derived {
            0.5 + (self.matching_entries as f64 * 0.05).min(0.4)
        } else {
            0.0
        }
    }
}

impl fmt::Display for InheritanceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} : {} {{", self.child_class, self.parent_class)?;
        writeln!(f, "  Matching vtable entries: {}", self.matching_entries)?;
        writeln!(f, "  Is likely derived: {}", self.is_likely_derived)?;
        writeln!(f, "  Overridden methods: {:?}", self.overridden_methods)?;
        writeln!(f, "  New virtuals: {:?}", self.new_virtuals)?;
        writeln!(f, "  Confidence: {:.1}%", self.confidence() * 100.0)?;
        write!(f, "}}")
    }
}

/// Builder for constructing vtables programmatically
pub struct VTableBuilder {
    address: Address,
    class_name: String,
    entries: Vec<VTableEntry>,
    rtti: Option<Address>,
}

impl VTableBuilder {
    pub fn new(address: Address, class_name: &str) -> Self {
        Self {
            address,
            class_name: class_name.to_string(),
            entries: Vec::new(),
            rtti: None,
        }
    }

    pub fn with_rtti(mut self, rtti: Address) -> Self {
        self.rtti = Some(rtti);
        self
    }

    pub fn add_function(mut self, func_addr: Address) -> Self {
        let index = self.entries.len();
        self.entries.push(VTableEntry::new(index, func_addr));
        self
    }

    pub fn add_named_function(mut self, func_addr: Address, name: &str) -> Self {
        let index = self.entries.len();
        let entry = VTableEntry::new(index, func_addr).with_name(name);
        self.entries.push(entry);
        self
    }

    pub fn add_override(mut self, func_addr: Address, name: &str, declaring_class: &str) -> Self {
        let index = self.entries.len();
        let entry = VTableEntry::new(index, func_addr)
            .with_name(name)
            .with_override(true)
            .with_declaring_class(declaring_class);
        self.entries.push(entry);
        self
    }

    pub fn build(self) -> VTable {
        let mut vtable = VTable::new(self.address, &self.class_name);
        vtable.entries = self.entries;
        vtable.size = vtable.entries.len() * 8;
        if let Some(rtti) = self.rtti {
            vtable.rtti_address = Some(rtti);
        }
        vtable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vtable_entry_creation() {
        let entry = VTableEntry::new(0, Address::new(0x100000000))
            .with_name("test_func")
            .with_override(true)
            .with_declaring_class("BaseClass");

        assert_eq!(entry.index, 0);
        assert_eq!(entry.function_name, Some("test_func".to_string()));
        assert!(entry.is_override);
        assert_eq!(entry.declaring_class, Some("BaseClass".to_string()));
    }

    #[test]
    fn test_vtable_builder() {
        let vtable = VTableBuilder::new(Address::new(0x200000000), "TestClass")
            .with_rtti(Address::new(0x100000000))
            .add_named_function(Address::new(0x300000000), "func1")
            .add_named_function(Address::new(0x300000100), "func2")
            .build();

        assert_eq!(vtable.class_name, "TestClass");
        assert_eq!(vtable.entry_count(), 2);
        assert!(vtable.rtti_address.is_some());
    }
}
