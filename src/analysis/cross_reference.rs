// Tue Jan 13 2026 - Alex

use crate::memory::{Address, MemoryReader, MemoryError, MemoryRegion};
use crate::analysis::disassembler::{Disassembler, DisassembledInstruction};
use std::sync::Arc;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct CrossReferenceAnalyzer {
    reader: Arc<dyn MemoryReader>,
    disassembler: Arc<Disassembler>,
    code_refs: HashMap<u64, Vec<CodeReference>>,
    data_refs: HashMap<u64, Vec<DataReference>>,
    string_refs: HashMap<u64, Vec<StringReference>>,
}

impl CrossReferenceAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>, disassembler: Arc<Disassembler>) -> Self {
        Self {
            reader,
            disassembler,
            code_refs: HashMap::new(),
            data_refs: HashMap::new(),
            string_refs: HashMap::new(),
        }
    }

    pub fn analyze_region(&mut self, region: &MemoryRegion) -> Result<usize, MemoryError> {
        if !region.protection.is_executable() {
            return Ok(0);
        }

        let instructions = self.disassembler.disassemble_region(region)?;
        let mut ref_count = 0;

        for instr in &instructions {
            if instr.is_call() || instr.is_branch() {
                if let Some(target) = instr.branch_target() {
                    self.add_code_ref(target, CodeReference {
                        from: instr.address,
                        to: Address::new(target),
                        ref_type: if instr.is_call() { CodeRefType::Call } else { CodeRefType::Branch },
                    });
                    ref_count += 1;
                }
            }

            if instr.mnemonic == "ADRP" {
                if let Some((next_idx, next_instr)) = self.find_next_instruction(&instructions, instr.address) {
                    if let Some(data_addr) = self.extract_adrp_add_target(instr, next_instr) {
                        self.add_data_ref(data_addr.as_u64(), DataReference {
                            from: instr.address,
                            to: data_addr,
                            ref_type: DataRefType::AddressLoad,
                        });
                        ref_count += 1;
                    }
                }
            }

            if instr.mnemonic == "LDR" && instr.op_str.contains("[PC") {
                if let Some(target) = self.extract_pc_relative_target(instr) {
                    self.add_data_ref(target.as_u64(), DataReference {
                        from: instr.address,
                        to: target,
                        ref_type: DataRefType::PcRelativeLoad,
                    });
                    ref_count += 1;
                }
            }
        }

        Ok(ref_count)
    }

    pub fn build_full_xref_database(&mut self) -> Result<XRefStatistics, MemoryError> {
        let regions = self.reader.get_regions()?;
        let mut stats = XRefStatistics::new();

        for region in &regions {
            let count = self.analyze_region(region)?;
            stats.total_refs += count;

            if region.protection.is_executable() {
                stats.code_regions_analyzed += 1;
            }
        }

        stats.code_refs = self.code_refs.values().map(|v| v.len()).sum();
        stats.data_refs = self.data_refs.values().map(|v| v.len()).sum();
        stats.string_refs = self.string_refs.values().map(|v| v.len()).sum();

        Ok(stats)
    }

    pub fn get_code_refs_to(&self, addr: Address) -> Option<&Vec<CodeReference>> {
        self.code_refs.get(&addr.as_u64())
    }

    pub fn get_data_refs_to(&self, addr: Address) -> Option<&Vec<DataReference>> {
        self.data_refs.get(&addr.as_u64())
    }

    pub fn get_string_refs_to(&self, addr: Address) -> Option<&Vec<StringReference>> {
        self.string_refs.get(&addr.as_u64())
    }

    pub fn get_callers(&self, func_addr: Address) -> Vec<Address> {
        self.code_refs.get(&func_addr.as_u64())
            .map(|refs| {
                refs.iter()
                    .filter(|r| r.ref_type == CodeRefType::Call)
                    .map(|r| r.from)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_callees(&self, func_addr: Address) -> Result<Vec<Address>, MemoryError> {
        let instructions = self.disassembler.disassemble_function(func_addr, 0x10000)?;

        let callees: Vec<Address> = instructions.iter()
            .filter(|i| i.is_call())
            .filter_map(|i| i.branch_target().map(Address::new))
            .collect();

        Ok(callees)
    }

    pub fn find_refs_in_range(&self, start: Address, end: Address) -> Vec<&CodeReference> {
        let mut refs = Vec::new();

        for (_, code_refs) in &self.code_refs {
            for r in code_refs {
                if r.from.as_u64() >= start.as_u64() && r.from.as_u64() < end.as_u64() {
                    refs.push(r);
                }
            }
        }

        refs
    }

    pub fn trace_call_chain(&self, from: Address, to: Address, max_depth: usize) -> Option<Vec<Address>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<u64, u64> = HashMap::new();

        queue.push_back((from, 0));
        visited.insert(from.as_u64());

        while let Some((current, depth)) = queue.pop_front() {
            if depth > max_depth {
                continue;
            }

            if current == to {
                let mut path = Vec::new();
                let mut curr = to.as_u64();

                while curr != from.as_u64() {
                    path.push(Address::new(curr));
                    curr = *parent.get(&curr)?;
                }
                path.push(from);
                path.reverse();

                return Some(path);
            }

            if let Ok(callees) = self.get_callees(current) {
                for callee in callees {
                    if !visited.contains(&callee.as_u64()) {
                        visited.insert(callee.as_u64());
                        parent.insert(callee.as_u64(), current.as_u64());
                        queue.push_back((callee, depth + 1));
                    }
                }
            }
        }

        None
    }

    pub fn find_common_callers(&self, addresses: &[Address]) -> Vec<Address> {
        if addresses.is_empty() {
            return Vec::new();
        }

        let first_callers: HashSet<u64> = self.get_callers(addresses[0])
            .iter()
            .map(|a| a.as_u64())
            .collect();

        let mut common: HashSet<u64> = first_callers;

        for addr in addresses.iter().skip(1) {
            let callers: HashSet<u64> = self.get_callers(*addr)
                .iter()
                .map(|a| a.as_u64())
                .collect();

            common = common.intersection(&callers).cloned().collect();
        }

        common.into_iter().map(Address::new).collect()
    }

    pub fn find_related_functions(&self, addr: Address, depth: usize) -> HashSet<Address> {
        let mut related = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((addr, 0));
        related.insert(addr);

        while let Some((current, current_depth)) = queue.pop_front() {
            if current_depth >= depth {
                continue;
            }

            for caller in self.get_callers(current) {
                if related.insert(caller) {
                    queue.push_back((caller, current_depth + 1));
                }
            }

            if let Ok(callees) = self.get_callees(current) {
                for callee in callees {
                    if related.insert(callee) {
                        queue.push_back((callee, current_depth + 1));
                    }
                }
            }
        }

        related
    }

    pub fn analyze_data_references(&mut self, data_start: Address, data_end: Address) -> Result<Vec<DataReference>, MemoryError> {
        let mut refs = Vec::new();
        let regions = self.reader.get_regions()?;

        for region in &regions {
            if !region.protection.is_executable() {
                continue;
            }

            let instructions = self.disassembler.disassemble_region(region)?;

            for instr in &instructions {
                if instr.mnemonic == "ADRP" {
                    if let Some((_, next_instr)) = self.find_next_instruction(&instructions, instr.address) {
                        if let Some(target) = self.extract_adrp_add_target(instr, next_instr) {
                            if target.as_u64() >= data_start.as_u64() && target.as_u64() < data_end.as_u64() {
                                let data_ref = DataReference {
                                    from: instr.address,
                                    to: target,
                                    ref_type: DataRefType::AddressLoad,
                                };
                                refs.push(data_ref.clone());
                                self.add_data_ref(target.as_u64(), data_ref);
                            }
                        }
                    }
                }
            }
        }

        Ok(refs)
    }

    fn add_code_ref(&mut self, target: u64, reference: CodeReference) {
        self.code_refs.entry(target).or_default().push(reference);
    }

    fn add_data_ref(&mut self, target: u64, reference: DataReference) {
        self.data_refs.entry(target).or_default().push(reference);
    }

    fn find_next_instruction<'a>(&self, instructions: &'a [DisassembledInstruction], addr: Address) -> Option<(usize, &'a DisassembledInstruction)> {
        for (idx, instr) in instructions.iter().enumerate() {
            if instr.address.as_u64() > addr.as_u64() {
                return Some((idx, instr));
            }
        }
        None
    }

    fn extract_adrp_add_target(&self, adrp: &DisassembledInstruction, add: &DisassembledInstruction) -> Option<Address> {
        if add.mnemonic != "ADD" && add.mnemonic != "LDR" {
            return None;
        }

        let page_addr = (adrp.address.as_u64() & !0xFFF) + self.extract_adrp_offset(&adrp.op_str)?;
        let offset = self.extract_add_offset(&add.op_str)?;

        Some(Address::new(page_addr + offset))
    }

    fn extract_adrp_offset(&self, op_str: &str) -> Option<u64> {
        for part in op_str.split(',') {
            let trimmed = part.trim().trim_start_matches('#');
            if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                if let Ok(val) = u64::from_str_radix(&trimmed[2..], 16) {
                    return Some(val << 12);
                }
            }
            if let Ok(val) = trimmed.parse::<i64>() {
                return Some((val as u64) << 12);
            }
        }
        None
    }

    fn extract_add_offset(&self, op_str: &str) -> Option<u64> {
        for part in op_str.split(',') {
            let trimmed = part.trim().trim_start_matches('#');
            if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                if let Ok(val) = u64::from_str_radix(&trimmed[2..], 16) {
                    return Some(val);
                }
            }
            if let Ok(val) = trimmed.parse::<u64>() {
                return Some(val);
            }
        }
        None
    }

    fn extract_pc_relative_target(&self, instr: &DisassembledInstruction) -> Option<Address> {
        for part in instr.op_str.split(',') {
            if part.contains("[PC") || part.contains("[pc") {
                let trimmed = part.trim();
                for num_part in trimmed.split(|c: char| !c.is_ascii_hexdigit() && c != 'x' && c != 'X') {
                    if num_part.starts_with("0x") || num_part.starts_with("0X") {
                        if let Ok(offset) = i64::from_str_radix(&num_part[2..], 16) {
                            let target = (instr.address.as_u64() as i64 + offset) as u64;
                            return Some(Address::new(target));
                        }
                    }
                }
            }
        }
        None
    }

    pub fn stats(&self) -> XRefStatistics {
        XRefStatistics {
            code_refs: self.code_refs.values().map(|v| v.len()).sum(),
            data_refs: self.data_refs.values().map(|v| v.len()).sum(),
            string_refs: self.string_refs.values().map(|v| v.len()).sum(),
            total_refs: 0,
            code_regions_analyzed: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeReference {
    pub from: Address,
    pub to: Address,
    pub ref_type: CodeRefType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeRefType {
    Call,
    Branch,
    IndirectCall,
    IndirectBranch,
}

#[derive(Debug, Clone)]
pub struct DataReference {
    pub from: Address,
    pub to: Address,
    pub ref_type: DataRefType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataRefType {
    AddressLoad,
    PcRelativeLoad,
    GotReference,
    Immediate,
}

#[derive(Debug, Clone)]
pub struct StringReference {
    pub from: Address,
    pub string_addr: Address,
    pub string_content: String,
}

#[derive(Debug, Clone, Default)]
pub struct XRefStatistics {
    pub code_refs: usize,
    pub data_refs: usize,
    pub string_refs: usize,
    pub total_refs: usize,
    pub code_regions_analyzed: usize,
}

impl XRefStatistics {
    pub fn new() -> Self {
        Self::default()
    }
}
